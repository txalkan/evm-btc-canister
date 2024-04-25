use serde::{de::DeserializeOwned, Serialize};
use candid::{CandidType, Deserialize};
use thiserror::Error;
use std::marker::PhantomData;
use ic_cdk::api::management_canister::http_request::{CanisterHttpRequestArgument, HttpResponse};
use ic_canister_log::log;
use async_trait::async_trait;
use futures::future::join_all;
use std::fmt::Debug;

use crate::*;

use cketh_common::{
    state::State,
    eth_rpc::{
        self,
        Hash, Block,
        SendRawTransactionResult,
        FeeHistory, FeeHistoryParams,
        GetLogsParam,
        LogEntry,
        HttpResponsePayload, HttpOutcallError,
        ResponseSizeEstimate,
        BlockSpec,
    },
    eth_rpc_client::{
        requests::GetTransactionCountParams,
        MultiCallResults, MultiCallError,
        responses::TransactionReceipt,
    },
    numeric::TransactionCount,
    logs::{INFO, DEBUG}
};

// This constant is our approximation of the expected header size.
// The HTTP standard doesn't define any limit, and many implementations limit
// the headers size to 8 KiB. We chose a lower limit because headers observed on most providers
// fit in the constant defined below, and if there is spike, then the payload size adjustment
// should take care of that.
const HEADER_SIZE_LIMIT: u64 = 2 * 1024;

pub(crate) const MAINNET_PROVIDERS: &[RpcService] = &[
    RpcService::EthMainnet(EthMainnetService::Alchemy),
    RpcService::EthMainnet(EthMainnetService::Ankr),
    RpcService::EthMainnet(EthMainnetService::PublicNode),
    RpcService::EthMainnet(EthMainnetService::Cloudflare),
];

pub(crate) const SEPOLIA_PROVIDERS: &[RpcService] = &[
    RpcService::EthSepolia(EthSepoliaService::Alchemy),
    RpcService::EthSepolia(EthSepoliaService::Ankr),
    RpcService::EthSepolia(EthSepoliaService::BlockPi),
    RpcService::EthSepolia(EthSepoliaService::PublicNode),
];

// Default RPC services for unknown EVM network
pub(crate) const UNKNOWN_PROVIDERS: &[RpcService] = &[];

#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub enum RpcError {
    // #[error("RPC provider error")]
    ProviderError(/* #[source] */ ProviderError),
    // #[error("HTTPS outcall error")]
    HttpOutcallError(/* #[source] */ HttpOutcallError),
    // #[error("JSON-RPC error")]
    JsonRpcError(/* #[source] */ JsonRpcError),
    // #[error("data format error")]
    ValidationError(/* #[source] */ ValidationError),
}

impl From<ProviderError> for RpcError {
    fn from(err: ProviderError) -> Self {
        RpcError::ProviderError(err)
    }
}

impl From<HttpOutcallError> for RpcError {
    fn from(err: HttpOutcallError) -> Self {
        RpcError::HttpOutcallError(err)
    }
}

impl From<JsonRpcError> for RpcError {
    fn from(err: JsonRpcError) -> Self {
        RpcError::JsonRpcError(err)
    }
}

impl From<ValidationError> for RpcError {
    fn from(err: ValidationError) -> Self {
        RpcError::ValidationError(err)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub enum ProviderError {
    // #[error("no permission")]
    NoPermission,
    // #[error("too few cycles (expected {expected}, received {received})")]
    TooFewCycles { expected: u128, received: u128 },
    // #[error("provider not found")]
    ProviderNotFound,
    // #[error("missing required provider")]
    MissingRequiredProvider,
}

#[derive(Clone, Hash, Debug, PartialEq, Eq, PartialOrd, Ord, CandidType, Deserialize)]
pub enum ValidationError {
    // #[error("{0}")]
    Custom(String),
    // #[error("invalid hex data: {0}")]
    InvalidHex(String),
    // #[error("URL parse error: {0}")]
    UrlParseError(String),
    // #[error("hostname not allowed: {0}")]
    HostNotAllowed(String),
    // #[error("credential path not allowed")]
    CredentialPathNotAllowed,
    // #[error("credential header not allowed")]
    CredentialHeaderNotAllowed,
}

#[derive(
    Clone, Hash, Debug, PartialEq, Eq, PartialOrd, Ord, CandidType, Serialize, Deserialize, Error,
)]
#[error("code {code}: {message}")]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, CandidType, Deserialize)]
pub struct RpcConfig {
    #[serde(rename = "responseSizeEstimate")]
    pub response_size_estimate: Option<u64>,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait RpcTransport: Debug {
    fn resolve_api(provider: &RpcService) -> Result<RpcApi, ProviderError>;

    async fn http_request(
        provider: &RpcService,
        method: &str,
        request: CanisterHttpRequestArgument,
        effective_size_estimate: u64,
    ) -> Result<HttpResponse, RpcError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EthRpcClient<T: RpcTransport> {
    chain: EthereumNetwork,
    providers: Option<Vec<RpcService>>,
    config: RpcConfig,
    phantom: PhantomData<T>,
}

impl<T: RpcTransport> EthRpcClient<T> {
    pub const fn new(
        chain: EthereumNetwork,
        providers: Option<Vec<RpcService>>,
        config: RpcConfig,
    ) -> Self {
        Self {
            chain,
            providers,
            config,
            phantom: PhantomData,
        }
    }

    pub fn from_state(state: &State) -> Self {
        Self::new(state.ethereum_network(), None, RpcConfig::default())
    }

    fn providers(&self) -> &[RpcService] {
        match self.providers {
            Some(ref providers) => providers,
            None => match self.chain {
                EthereumNetwork::MAINNET => MAINNET_PROVIDERS,
                EthereumNetwork::SEPOLIA => SEPOLIA_PROVIDERS,
                _ => UNKNOWN_PROVIDERS,
            },
        }
    }

    fn response_size_estimate(&self, estimate: u64) -> ResponseSizeEstimate {
        ResponseSizeEstimate::new(self.config.response_size_estimate.unwrap_or(estimate))
    }

    /// Query all providers in sequence until one returns an ok result
    /// (which could still be a JsonRpcResult::Error).
    /// If none of the providers return an ok result, return the last error.
    /// This method is useful in case a provider is temporarily down but should only be for
    /// querying data that is **not** critical since the returned value comes from a single provider.
    async fn sequential_call_until_ok<I, O>(
        &self,
        method: impl Into<String> + Clone,
        params: I,
        response_size_estimate: ResponseSizeEstimate,
    ) -> Result<O, RpcError>
    where
        I: Serialize + Clone,
        O: DeserializeOwned + HttpResponsePayload + Debug,
    {
        let mut last_result: Option<Result<O, RpcError>> = None;
        for provider in self.providers() {
            log!(
                DEBUG,
                "[sequential_call_until_ok]: calling provider: {:?}",
                provider
            );
            let result = eth_rpc::call::<T, _, _>(
                provider,
                method.clone(),
                params.clone(),
                response_size_estimate,
            )
            .await;
            match result {
                Ok(value) => return Ok(value),
                Err(RpcError::JsonRpcError(json_rpc_error @ JsonRpcError { .. })) => {
                    log!(
                        INFO,
                        "{provider:?} returned JSON-RPC error {json_rpc_error:?}",
                    );
                    last_result = Some(Err(json_rpc_error.into()));
                }
                Err(e) => {
                    log!(INFO, "Querying {provider:?} returned error {e:?}");
                    last_result = Some(Err(e));
                }
            };
        }
        last_result.unwrap_or_else(|| panic!("BUG: No providers in RPC client {:?}", self))
    }

    /// Query all providers in parallel and return all results.
    /// It's up to the caller to decide how to handle the results, which could be inconsistent among one another,
    /// (e.g., if different providers gave different responses).
    /// This method is useful for querying data that is critical for the system to ensure that there is no single point of failure,
    /// e.g., ethereum logs upon which ckETH will be minted.
    async fn parallel_call<I, O>(
        &self,
        method: impl Into<String> + Clone,
        params: I,
        response_size_estimate: ResponseSizeEstimate,
    ) -> MultiCallResults<O>
    where
        I: Serialize + Clone,
        O: DeserializeOwned + HttpResponsePayload,
    {
        let providers = self.providers();
        let results = {
            let mut fut = Vec::with_capacity(providers.len());
            for provider in providers {
                log!(DEBUG, "[parallel_call]: will call provider: {:?}", provider);
                fut.push(async {
                    eth_rpc::call::<T, _, _>(
                        provider,
                        method.clone(),
                        params.clone(),
                        response_size_estimate,
                    )
                    .await
                });
            }
            join_all(fut).await
        };
        MultiCallResults::from_non_empty_iter(providers.iter().cloned().zip(results.into_iter()))
    }

    pub async fn eth_get_logs(
        &self,
        params: GetLogsParam,
    ) -> Result<Vec<LogEntry>, MultiCallError<Vec<LogEntry>>> {
        let results: MultiCallResults<Vec<LogEntry>> = self
            .parallel_call(
                "eth_getLogs",
                vec![params],
                self.response_size_estimate(1024 + HEADER_SIZE_LIMIT),
            )
            .await;
        results.reduce_with_equality()
    }

    pub async fn eth_get_block_by_number(
        &self,
        block: BlockSpec,
    ) -> Result<Block, MultiCallError<Block>> {
        use cketh_common::eth_rpc::GetBlockByNumberParams;

        let expected_block_size = match self.chain {
            EthereumNetwork::SEPOLIA => 12 * 1024,
            EthereumNetwork::MAINNET => 24 * 1024,
            _ => 24 * 1024, // Default for unknown networks
        };

        let results: MultiCallResults<Block> = self
            .parallel_call(
                "eth_getBlockByNumber",
                GetBlockByNumberParams {
                    block,
                    include_full_transactions: false,
                },
                self.response_size_estimate(expected_block_size + HEADER_SIZE_LIMIT),
            )
            .await;
        results.reduce_with_equality()
    }

    pub async fn eth_get_transaction_receipt(
        &self,
        tx_hash: Hash,
    ) -> Result<Option<TransactionReceipt>, MultiCallError<Option<TransactionReceipt>>> {
        let results: MultiCallResults<Option<TransactionReceipt>> = self
            .parallel_call(
                "eth_getTransactionReceipt",
                vec![tx_hash],
                self.response_size_estimate(700 + HEADER_SIZE_LIMIT),
            )
            .await;
        results.reduce_with_equality()
    }

    pub async fn eth_fee_history(
        &self,
        params: FeeHistoryParams,
    ) -> Result<FeeHistory, MultiCallError<FeeHistory>> {
        // A typical response is slightly above 300 bytes.
        let results: MultiCallResults<FeeHistory> = self
            .parallel_call(
                "eth_feeHistory",
                params,
                self.response_size_estimate(512 + HEADER_SIZE_LIMIT),
            )
            .await;
        results.reduce_with_strict_majority_by_key(|fee_history| fee_history.oldest_block)
    }

    pub async fn eth_send_raw_transaction(
        &self,
        raw_signed_transaction_hex: String,
    ) -> Result<SendRawTransactionResult, RpcError> {
        // A successful reply is under 256 bytes, but we expect most calls to end with an error
        // since we submit the same transaction from multiple nodes.
        self.sequential_call_until_ok(
            "eth_sendRawTransaction",
            vec![raw_signed_transaction_hex],
            self.response_size_estimate(256 + HEADER_SIZE_LIMIT),
        )
        .await
    }

    pub async fn multi_eth_send_raw_transaction(
        &self,
        raw_signed_transaction_hex: String,
    ) -> Result<SendRawTransactionResult, MultiCallError<SendRawTransactionResult>> {
        self.parallel_call(
            "eth_sendRawTransaction",
            vec![raw_signed_transaction_hex],
            self.response_size_estimate(256 + HEADER_SIZE_LIMIT),
        )
        .await
        .reduce_with_equality()
    }

    pub async fn eth_get_transaction_count(
        &self,
        params: GetTransactionCountParams,
    ) -> MultiCallResults<TransactionCount> {
        self.parallel_call(
            "eth_getTransactionCount",
            params,
            self.response_size_estimate(50 + HEADER_SIZE_LIMIT),
        )
        .await
    }
}

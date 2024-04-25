use candid::CandidType;
use cketh_common::{
    logs::INFO,
};
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::HttpHeader;
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize, CandidType)]
pub struct RpcApi {
    pub url: String,
    pub headers: Option<Vec<HttpHeader>>,
}

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize, CandidType)]
pub enum RpcService {
    EthMainnet(EthMainnetService),
    EthSepolia(EthSepoliaService),
    Chain(u64),
    Provider(u64),
    Custom(RpcApi),
}

impl std::fmt::Debug for RpcService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcService::EthMainnet(service) => write!(f, "{:?}", service),
            RpcService::EthSepolia(service) => write!(f, "{:?}", service),
            RpcService::Chain(chain_id) => write!(f, "Chain({})", chain_id),
            RpcService::Provider(provider_id) => write!(f, "Provider({})", provider_id),
            RpcService::Custom(_) => write!(f, "Custom(..)"), // Redact credentials
        }
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize, CandidType,
)]
pub enum EthMainnetService {
    Alchemy,
    Ankr,
    BlockPi,
    PublicNode,
    Cloudflare,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize, CandidType,
)]
pub enum EthSepoliaService {
    Alchemy,
    Ankr,
    BlockPi,
    PublicNode,
}

pub const ANKR_HOSTNAME: &str = "rpc.ankr.com";
pub const ALCHEMY_ETH_MAINNET_HOSTNAME: &str = "eth-mainnet.g.alchemy.com";
pub const ALCHEMY_ETH_SEPOLIA_HOSTNAME: &str = "eth-sepolia.g.alchemy.com";
pub const CLOUDFLARE_HOSTNAME: &str = "cloudflare-eth.com";
pub const BLOCKPI_ETH_MAINNET_HOSTNAME: &str = "ethereum.blockpi.network";
pub const BLOCKPI_ETH_SEPOLIA_HOSTNAME: &str = "ethereum-sepolia.blockpi.network";
pub const PUBLICNODE_ETH_MAINNET_HOSTNAME: &str = "ethereum-rpc.publicnode.com";
pub const PUBLICNODE_ETH_SEPOLIA_HOSTNAME: &str = "ethereum-sepolia-rpc.publicnode.com";
pub const ETH_SEPOLIA_HOSTNAME: &str = "rpc.sepolia.org";

// Limited API credentials for local testing.
// Use `dfx canister call evm_rpc updateProvider ...` to pass your own keys.
pub const ALCHEMY_ETH_MAINNET_CREDENTIAL: &str = "/v2/zBxaSBUMfuH8XnA-uLIWeXfCx1T8ItkM";
pub const ALCHEMY_ETH_SEPOLIA_CREDENTIAL: &str = "/v2/Mbow19DWsfPXiTpdgvRu4HQq63iYycU-";
pub const BLOCKPI_ETH_MAINNET_CREDENTIAL: &str = "/v1/rpc/0edc81e20be23ddff051f61a97bb457ec7284a58";
pub const BLOCKPI_ETH_SEPOLIA_CREDENTIAL: &str = "/v1/rpc/1fe987fddded17db50862311720ff444991d4dab";

pub fn get_default_providers() -> Vec<RegisterProviderArgs> {
    vec![
        RegisterProviderArgs {
            chain_id: ETH_MAINNET_CHAIN_ID,
            hostname: CLOUDFLARE_HOSTNAME.to_string(),
            credential_path: "/v1/mainnet".to_string(),
            credential_headers: None,
            cycles_per_call: 0,
            cycles_per_message_byte: 0,
        },
        RegisterProviderArgs {
            chain_id: ETH_MAINNET_CHAIN_ID,
            hostname: ANKR_HOSTNAME.to_string(),
            credential_path: "/eth".to_string(),
            credential_headers: None,
            cycles_per_call: 0,
            cycles_per_message_byte: 0,
        },
        RegisterProviderArgs {
            chain_id: ETH_MAINNET_CHAIN_ID,
            hostname: PUBLICNODE_ETH_MAINNET_HOSTNAME.to_string(),
            credential_path: "".to_string(),
            credential_headers: None,
            cycles_per_call: 0,
            cycles_per_message_byte: 0,
        },
        RegisterProviderArgs {
            chain_id: ETH_MAINNET_CHAIN_ID,
            hostname: BLOCKPI_ETH_MAINNET_HOSTNAME.to_string(),
            credential_path: BLOCKPI_ETH_MAINNET_CREDENTIAL.to_string(),
            credential_headers: None,
            cycles_per_call: 0,
            cycles_per_message_byte: 0,
        },
        RegisterProviderArgs {
            chain_id: ETH_SEPOLIA_CHAIN_ID,
            hostname: ETH_SEPOLIA_HOSTNAME.to_string(),
            credential_path: "".to_string(),
            credential_headers: None,
            cycles_per_call: 0,
            cycles_per_message_byte: 0,
        },
        RegisterProviderArgs {
            chain_id: ETH_SEPOLIA_CHAIN_ID,
            hostname: ANKR_HOSTNAME.to_string(),
            credential_path: "/eth_sepolia".to_string(),
            credential_headers: None,
            cycles_per_call: 0,
            cycles_per_message_byte: 0,
        },
        RegisterProviderArgs {
            chain_id: ETH_SEPOLIA_CHAIN_ID,
            hostname: BLOCKPI_ETH_SEPOLIA_HOSTNAME.to_string(),
            credential_path: BLOCKPI_ETH_SEPOLIA_CREDENTIAL.to_string(),
            credential_headers: None,
            cycles_per_call: 0,
            cycles_per_message_byte: 0,
        },
        RegisterProviderArgs {
            chain_id: ETH_SEPOLIA_CHAIN_ID,
            hostname: PUBLICNODE_ETH_SEPOLIA_HOSTNAME.to_string(),
            credential_path: "".to_string(),
            credential_headers: None,
            cycles_per_call: 0,
            cycles_per_message_byte: 0,
        },
        RegisterProviderArgs {
            chain_id: ETH_MAINNET_CHAIN_ID,
            hostname: ALCHEMY_ETH_MAINNET_HOSTNAME.to_string(),
            credential_path: ALCHEMY_ETH_MAINNET_CREDENTIAL.to_string(),
            credential_headers: None,
            cycles_per_call: 0,
            cycles_per_message_byte: 0,
        },
        RegisterProviderArgs {
            chain_id: ETH_SEPOLIA_CHAIN_ID,
            hostname: ALCHEMY_ETH_SEPOLIA_HOSTNAME.to_string(),
            credential_path: ALCHEMY_ETH_SEPOLIA_CREDENTIAL.to_string(),
            credential_headers: None,
            cycles_per_call: 0,
            cycles_per_message_byte: 0,
        },
    ]
}

pub fn get_default_service_provider_hostnames() -> Vec<(RpcService, &'static str)> {
    vec![
        (
            RpcService::EthMainnet(EthMainnetService::Alchemy),
            ALCHEMY_ETH_MAINNET_HOSTNAME,
        ),
        (
            RpcService::EthMainnet(EthMainnetService::Ankr),
            ANKR_HOSTNAME,
        ),
        (
            RpcService::EthMainnet(EthMainnetService::BlockPi),
            BLOCKPI_ETH_MAINNET_HOSTNAME,
        ),
        (
            RpcService::EthMainnet(EthMainnetService::Cloudflare),
            CLOUDFLARE_HOSTNAME,
        ),
        (
            RpcService::EthMainnet(EthMainnetService::PublicNode),
            PUBLICNODE_ETH_MAINNET_HOSTNAME,
        ),
        (
            RpcService::EthSepolia(EthSepoliaService::Alchemy),
            ALCHEMY_ETH_SEPOLIA_HOSTNAME,
        ),
        (
            RpcService::EthSepolia(EthSepoliaService::Ankr),
            ANKR_HOSTNAME,
        ),
        (
            RpcService::EthSepolia(EthSepoliaService::BlockPi),
            BLOCKPI_ETH_SEPOLIA_HOSTNAME,
        ),
        (
            RpcService::EthSepolia(EthSepoliaService::PublicNode),
            PUBLICNODE_ETH_SEPOLIA_HOSTNAME,
        ),
    ]
}

pub fn find_provider(f: impl Fn(&Provider) -> bool) -> Option<Provider> {
    PROVIDERS.with(|providers| {
        let providers = providers.borrow();
        Some(
            providers
                .iter()
                .find(|(_, p)| p.primary && f(p))
                .or_else(|| providers.iter().find(|(_, p)| f(p)))?
                .1,
        )
    })
}

fn lookup_provider_for_service(service: &RpcService) -> Result<Provider, ProviderError> {
    let provider_id = SERVICE_PROVIDER_MAP.with(|map| {
        map.borrow()
            .get(&StorableRpcService::new(service))
            .ok_or(ProviderError::MissingRequiredProvider)
    })?;
    PROVIDERS
        .with(|providers| providers.borrow().get(&provider_id))
        .ok_or(ProviderError::ProviderNotFound)
}

pub fn get_known_chain_id(service: &RpcService) -> Option<u64> {
    match service {
        RpcService::EthMainnet(_) => Some(ETH_MAINNET_CHAIN_ID),
        RpcService::EthSepolia(_) => Some(ETH_SEPOLIA_CHAIN_ID),
        RpcService::Chain(chain_id) => Some(*chain_id),
        RpcService::Provider(_) => None,
        RpcService::Custom(_) => None,
    }
}

pub fn do_register_provider(caller: Principal, args: RegisterProviderArgs) -> u64 {
    validate_hostname(&args.hostname).unwrap();
    validate_credential_path(&args.credential_path).unwrap();
    let provider_id = METADATA.with(|m| {
        let mut metadata = m.borrow().get().clone();
        let id = metadata.next_provider_id;
        metadata.next_provider_id += 1;
        m.borrow_mut().set(metadata).unwrap();
        id
    });
    do_deauthorize(caller, Auth::RegisterProvider);
    log!(INFO, "[{}] Registering provider: {:?}", caller, provider_id);
    PROVIDERS.with(|providers| {
        providers.borrow_mut().insert(
            provider_id,
            Provider {
                provider_id,
                owner: caller,
                chain_id: args.chain_id,
                hostname: args.hostname,
                credential_path: args.credential_path,
                credential_headers: args.credential_headers.unwrap_or_default(),
                cycles_per_call: args.cycles_per_call,
                cycles_per_message_byte: args.cycles_per_message_byte,
                cycles_owed: 0,
                primary: false,
            },
        )
    });
    provider_id
}

pub fn do_unregister_provider(caller: Principal, is_controller: bool, provider_id: u64) -> bool {
    PROVIDERS.with(|providers| {
        let mut providers = providers.borrow_mut();
        if let Some(provider) = providers.get(&provider_id) {
            if provider.owner == caller || is_controller {
                log!(
                    INFO,
                    "[{}] Unregistering provider: {:?}",
                    caller,
                    provider_id
                );
                providers.remove(&provider_id).is_some()
            } else {
                ic_cdk::trap("You are not authorized: check provider owner");
            }
        } else {
            false
        }
    })
}

/// Changes provider details. The caller must be the owner of the provider.
pub fn do_update_provider(caller: Principal, is_controller: bool, args: UpdateProviderArgs) {
    PROVIDERS.with(|providers| {
        let mut providers = providers.borrow_mut();
        match providers.get(&args.provider_id) {
            Some(mut provider) => {
                if provider.owner == caller || is_controller {
                    log!(INFO, "[{}] Updating provider: {}", caller, args.provider_id);
                    if let Some(hostname) = args.hostname {
                        validate_hostname(&hostname).unwrap();
                        provider.hostname = hostname;
                    }
                    if let Some(path) = args.credential_path {
                        validate_credential_path(&path).unwrap();
                        provider.credential_path = path;
                    }
                    if let Some(headers) = args.credential_headers {
                        validate_credential_headers(&headers).unwrap();
                        provider.credential_headers = headers;
                    }
                    if let Some(cycles_per_call) = args.cycles_per_call {
                        provider.cycles_per_call = cycles_per_call;
                    }
                    if let Some(cycles_per_message_byte) = args.cycles_per_message_byte {
                        provider.cycles_per_message_byte = cycles_per_message_byte;
                    }
                    providers.insert(args.provider_id, provider);
                } else {
                    ic_cdk::trap("You are not authorized: check provider owner");
                }
            }
            None => ic_cdk::trap("Provider not found"),
        }
    });
}

/// Changes administrative details for a provider. The caller must have the `Auth::Manage` permission.
pub fn do_manage_provider(args: ManageProviderArgs) {
    PROVIDERS.with(|providers| {
        let mut providers = providers.borrow_mut();
        match providers.get(&args.provider_id) {
            Some(mut provider) => {
                if let Some(primary) = args.primary {
                    provider.primary = primary;
                }
                if let Some(service) = args.service {
                    set_service_provider(&service, &provider);
                }
                providers.insert(args.provider_id, provider);
            }
            None => ic_cdk::trap("Provider not found"),
        }
    })
}

pub fn do_get_accumulated_cycle_count(
    caller: Principal,
    is_controller: bool,
    provider_id: u64,
) -> u128 {
    let provider = PROVIDERS
        .with(|p| {
            p.borrow()
                .get(&provider_id)
                .ok_or(ProviderError::ProviderNotFound)
        })
        .expect("Provider not found");
    if caller == provider.owner || is_controller {
        provider.cycles_owed
    } else {
        ic_cdk::trap("You are not authorized: check provider owner");
    }
}

pub async fn do_withdraw_accumulated_cycles(
    caller: Principal,
    is_controller: bool,
    provider_id: u64,
    canister_id: Principal,
) {
    let mut provider = PROVIDERS
        .with(|p| {
            p.borrow()
                .get(&provider_id)
                .ok_or(ProviderError::ProviderNotFound)
        })
        .expect("Provider not found");
    if caller == provider.owner || is_controller {
        let amount = provider.cycles_owed;
        if amount < MINIMUM_WITHDRAWAL_CYCLES {
            ic_cdk::trap("Too few cycles to withdraw");
        }
        PROVIDERS.with(|p| {
            provider.cycles_owed = 0;
            p.borrow_mut().insert(provider_id, provider)
        });
        log!(
            INFO,
            "[{}] Withdrawing {} cycles from provider {} to canister: {}",
            caller,
            amount,
            provider_id,
            canister_id,
        );
        #[derive(CandidType)]
        struct DepositCyclesArgs {
            canister_id: Principal,
        }
        match ic_cdk::api::call::call_with_payment128(
            Principal::management_canister(),
            "deposit_cycles",
            (DepositCyclesArgs { canister_id },),
            amount,
        )
        .await
        {
            Ok(()) => add_metric!(cycles_withdrawn, amount),
            Err(err) => {
                // Refund on failure to send cycles
                log!(
                    INFO,
                    "[{}] Unable to send {} cycles from provider {}: {:?}",
                    canister_id,
                    amount,
                    provider_id,
                    err
                );
                // Protect against re-entrancy
                let provider = PROVIDERS.with(|p| {
                    p.borrow()
                        .get(&provider_id)
                        .ok_or(ProviderError::ProviderNotFound)
                });
                let mut provider = provider.expect("Provider not found during refund, cycles lost");
                PROVIDERS.with(|p| {
                    provider.cycles_owed += amount;
                    p.borrow_mut().insert(provider_id, provider)
                });
            }
        };
    } else {
        ic_cdk::trap("You are not authorized: check provider owner");
    }
}

pub fn set_service_provider(service: &RpcService, provider: &Provider) {
    log!(
        INFO,
        "Changing service {:?} to use provider: {}",
        service,
        provider.provider_id
    );
    if let Some(chain_id) = get_known_chain_id(service) {
        if chain_id != provider.chain_id {
            ic_cdk::trap(&format!(
                "Mismatch between service and provider chain ids ({} != {})",
                chain_id, provider.chain_id
            ))
        }
    }
    SERVICE_PROVIDER_MAP.with(|mappings| {
        mappings
            .borrow_mut()
            .insert(StorableRpcService::new(service), provider.provider_id);
    });
}

pub fn resolve_rpc_service(service: RpcService) -> Result<ResolvedRpcService, ProviderError> {
    Ok(match service {
        RpcService::EthMainnet(service) => ResolvedRpcService::Provider(
            lookup_provider_for_service(&RpcService::EthMainnet(service))?,
        ),
        RpcService::EthSepolia(service) => ResolvedRpcService::Provider(
            lookup_provider_for_service(&RpcService::EthSepolia(service))?,
        ),
        RpcService::Chain(id) => ResolvedRpcService::Provider(PROVIDERS.with(|providers| {
            let providers = providers.borrow();
            Ok(providers
                .iter()
                .find(|(_, p)| p.primary && p.chain_id == id)
                .or_else(|| providers.iter().find(|(_, p)| p.chain_id == id))
                .ok_or(ProviderError::ProviderNotFound)?
                .1)
        })?),
        RpcService::Provider(id) => ResolvedRpcService::Provider({
            PROVIDERS.with(|providers| {
                providers
                    .borrow()
                    .get(&id)
                    .ok_or(ProviderError::ProviderNotFound)
            })?
        }),
        RpcService::Custom(RpcApi { url, headers }) => {
            ResolvedRpcService::Api(RpcApi { url, headers })
        }
    })
}

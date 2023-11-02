use candid::candid_method;
use ic_cdk_macros::update;

use e2e::declarations::eth_rpc::{eth_rpc, EthRpcError, Source};

fn main() {}

#[update]
#[candid_method(update)]
pub async fn test() {
    // Define request parameters
    let params = (
        &Source::Service {
            hostname: "cloudflare-eth.com".to_string(),
            chain_id: Some(1), // Ethereum mainnet
        },
        "{\"jsonrpc\":\"2.0\",\"method\":\"eth_gasPrice\",\"params\":null,\"id\":1}".to_string(),
        1000 as u64,
    );

    // Get cycles cost
    let (cycles_result,): (Result<u128, EthRpcError>,) =
        ic_cdk::call(eth_rpc.0, "request_cost", params.clone())
            .await
            .unwrap();
    let cycles =
        cycles_result.unwrap_or_else(|e| ic_cdk::trap(&format!("error in `request_cost`: {}", e)));

    // Call without sending cycles
    let (result_without_cycles,): (Result<String, EthRpcError>,) =
        ic_cdk::api::call::call(eth_rpc.0, "request", params.clone())
            .await
            .unwrap();
    match result_without_cycles {
        Ok(s) => ic_cdk::trap(&format!("response from `request` without cycles: {:?}", s)),
        Err(EthRpcError::TooFewCycles { expected, .. }) => {
            assert_eq!(expected, cycles)
        }
        Err(err) => ic_cdk::trap(&format!("error in `request` without cycles: {}", err)),
    }

    // Call with expected number of cycles
    let (result,): (Result<String, EthRpcError>,) =
        ic_cdk::api::call::call_with_payment128(eth_rpc.0, "request", params, cycles)
            .await
            .unwrap();
    match result {
        Ok(response) => {
            // Check response structure around gas price
            assert_eq!(&response[..29], "{\"jsonrpc\":\"2.0\",\"result\":\"0x",);
            assert_eq!(&response[response.len() - 9..], "\",\"id\":1}")
        }
        Err(err) => ic_cdk::trap(&format!("error in `request` with cycles: {}", err)),
    }
}
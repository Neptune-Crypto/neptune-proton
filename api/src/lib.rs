//! This crate contains all shared fullstack server functions.

pub mod fiat_amount;
pub mod fiat_currency;
pub mod prefs;
#[cfg(not(target_arch = "wasm32"))]
mod price_caching;
pub mod price_map;
pub mod price_providers;
#[cfg(not(target_arch = "wasm32"))]
mod rpc_api;

use std::net::Ipv4Addr;
use std::net::SocketAddr;

use dioxus::prelude::*;
use neptune_types::address::KeyType;
use neptune_types::address::ReceivingAddress;
use neptune_types::address::SpendingKey;
use neptune_types::block_height::BlockHeight;
use neptune_types::block_info::BlockInfo;
use neptune_types::block_selector::BlockSelector;
use neptune_types::change_policy::ChangePolicy;
use neptune_types::dashboard_overview_data_from_client::DashBoardOverviewDataFromClient;
use neptune_types::mempool_transaction_info::MempoolTransactionInfo;
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use neptune_types::network::Network;
use neptune_types::output_format::OutputFormat;
use neptune_types::peer_info::PeerInfo as NeptunePeerInfo;
use neptune_types::timestamp::Timestamp;
use neptune_types::transaction_details::TransactionDetails;
use neptune_types::transaction_kernel::TransactionKernel;
use neptune_types::transaction_kernel_id::TransactionKernelId;
use prefs::user_prefs::UserPrefs;
use price_map::PriceMap;
use twenty_first::tip5::Digest;

pub type ApiError = anyhow::Error;

/// Retrieves the user's preferences.
///
/// In the future this may read from a settings file.  For now it just
/// returns the default settings, which read from env vars.
#[post("/api/get_user_prefs")]
pub async fn get_user_prefs() -> Result<UserPrefs, ApiError> {
    Ok(UserPrefs::default())
}

#[post("/api/network")]
pub async fn network() -> Result<Network, ApiError> {
    println!("DEBUG: [network] Called");

    // 1. Connection
    println!("DEBUG: [network] calling rpc_client()...");
    let client_res = neptune_rpc::rpc_client().await;

    let client = match client_res {
        Ok(c) => {
            println!("DEBUG: [network] rpc_client obtained successfully");
            c
        }
        Err(e) => {
            println!("DEBUG: [network] rpc_client failed: {:?}", e);
            // If this prints and then the frontend says "Shutdown",
            // it confirms the crash happens when returning this error.
            return Err(e);
        }
    };

    // 2. Execution
    println!("DEBUG: [network] calling client.network(context)...");
    let result = client.network(tarpc::context::current()).await;

    match result {
        Ok(Ok(n)) => {
            println!("DEBUG: [network] Success: {:?}", n);
            Ok(n)
        }
        Ok(Err(e)) => {
            println!("DEBUG: [network] Logic Error from Core: {:?}", e);
            Err(e.into())
        }
        Err(e) => {
            // This is the Tarpc Transport error (Shutdown/BrokenPipe)
            println!("DEBUG: [network] Transport Error: {:?}", e);
            Err(e.into())
        }
    }
}

// pub async fn network() -> Result<Network, ApiError> {
//     neptune_rpc::network().await
// }

#[post("/api/wallet_balance")]
pub async fn wallet_balance() -> Result<NativeCurrencyAmount, ApiError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let balance = client
        .confirmed_available_balance(tarpc::context::current(), token)
        .await??;

    let json = serde_json::to_string(&balance)?;
    dioxus_logger::tracing::info!("balance json: {}", json);

    Ok(balance)
}

#[post("/api/block_height")]
pub async fn block_height() -> Result<BlockHeight, ApiError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let height = client
        .block_height(tarpc::context::current(), token)
        .await??;
    Ok(height.into())
}

#[post("/api/known_keys")]
pub async fn known_keys() -> Result<Vec<SpendingKey>, ApiError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let known_keys = client
        .known_keys(tarpc::context::current(), token)
        .await??;
    Ok(known_keys)
}

#[post("/api/next_receiving_address")]
pub async fn next_receiving_address(key_type: KeyType) -> Result<ReceivingAddress, ApiError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let address = client
        .next_receiving_address(tarpc::context::current(), token, key_type)
        .await??;
    Ok(address)
}

#[post("/api/send")]
pub async fn send(
    outputs: Vec<OutputFormat>,
    change_policy: ChangePolicy,
    fee: NativeCurrencyAmount,
) -> Result<(TransactionKernelId, TransactionDetails), ApiError> {
    neptune_rpc::send(outputs, change_policy, fee).await
}

#[server(input = Json, output = Json)]
#[post("/api/history")]
pub async fn history(
) -> Result<Vec<(Digest, BlockHeight, Timestamp, NativeCurrencyAmount)>, ApiError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let history = client.history(tarpc::context::current(), token).await??;
    Ok(history)
}

#[post("/api/mempool_overview")]
pub async fn mempool_overview(
    start_index: usize,
    number: usize,
) -> Result<Vec<MempoolTransactionInfo>, ApiError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let data = client
        .mempool_overview(tarpc::context::current(), token, start_index, number)
        .await??;
    Ok(data)
}

#[post("/api/mempool_tx_kernel")]
pub async fn mempool_tx_kernel(
    txid: TransactionKernelId,
) -> Result<Option<TransactionKernel>, ApiError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let data = client
        .mempool_tx_kernel(tarpc::context::current(), token, txid)
        .await??;
    Ok(data)
}

#[post("/api/block_info")]
pub async fn block_info(selector: BlockSelector) -> Result<Option<BlockInfo>, ApiError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let data = client
        .block_info(tarpc::context::current(), token, selector)
        .await??;
    Ok(data)
}

#[post("/api/dashboard_overview_data")]
pub async fn dashboard_overview_data() -> Result<DashBoardOverviewDataFromClient, ApiError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let data = client
        .dashboard_overview_data(tarpc::context::current(), token)
        .await??;
    Ok(data)
}

#[post("/api/peer_info")]
pub async fn peer_info() -> Result<Vec<NeptunePeerInfo>, ApiError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let data = client.peer_info(tarpc::context::current(), token).await??;
    Ok(data)
}

#[post("/api/fiat_prices")]
pub async fn fiat_prices() -> Result<PriceMap, ApiError> {
    Ok(price_caching::get_cached_fiat_prices().await?)
}

#[get("/api/neptune_core_rpc_socket_addr")]
pub async fn neptune_core_rpc_socket_addr() -> Result<SocketAddr, ApiError> {
    Ok(SocketAddr::new(
        std::net::IpAddr::V4(Ipv4Addr::LOCALHOST),
        neptune_rpc::neptune_core_rpc_port(),
    ))
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
mod neptune_rpc {
    // use neptune_cash::api::export::Transaction;
    // use neptune_cash::api::export::TransactionDetails;
    use std::net::Ipv4Addr;
    use std::net::SocketAddr;

    use neptune_cash::application::rpc::auth as rpc_auth;
    use neptune_cash::application::rpc::server::RPCClient;
    use neptune_types::change_policy::ChangePolicy;
    use neptune_types::native_currency_amount::NativeCurrencyAmount;
    use neptune_types::network::Network;
    use neptune_types::output_format::OutputFormat;
    use neptune_types::transaction_details::TransactionDetails;
    use neptune_types::transaction_kernel_id::TransactionKernelId;
    use tarpc::client;
    use tarpc::context;
    use tarpc::tokio_serde::formats::Json;

    use super::rpc_api;
    use super::ApiError;

    pub fn neptune_core_rpc_port() -> u16 {
        const DEFAULT_PORT: u16 = 9799;
        std::env::var("NEPTUNE_CORE_RPC_PORT")
            .unwrap_or("".to_string())
            .parse()
            .unwrap_or(DEFAULT_PORT)
    }

    async fn gen_rpc_client() -> Result<rpc_api::RPCClient, ApiError> {
        let server_socket = SocketAddr::new(
            std::net::IpAddr::V4(Ipv4Addr::LOCALHOST),
            neptune_core_rpc_port(),
        );
        let transport = tarpc::serde_transport::tcp::connect(server_socket, Json::default).await?;

        Ok(rpc_api::RPCClient::new(client::Config::default(), transport).spawn())
    }

    async fn gen_nc_rpc_client() -> Result<RPCClient, ApiError> {
        let server_socket = SocketAddr::new(
            std::net::IpAddr::V4(Ipv4Addr::LOCALHOST),
            neptune_core_rpc_port(),
        );
        let transport = tarpc::serde_transport::tcp::connect(server_socket, Json::default).await?;

        Ok(RPCClient::new(client::Config::default(), transport).spawn())
    }
    pub async fn rpc_client() -> Result<rpc_api::RPCClient, ApiError> {
        // no caching for now.  very fast to establish a connection on localhost
        // and this way there is no need to invalidate cache on connection error.
        gen_rpc_client().await
    }

    pub async fn cookie_hint() -> Result<rpc_auth::CookieHint, ApiError> {
        let client = rpc_client().await?;
        Ok(client.cookie_hint(context::current()).await??)
    }

    async fn gen_token() -> Result<rpc_auth::Token, ApiError> {
        let hint = cookie_hint().await?;
        Ok(rpc_auth::Cookie::try_load(&hint.data_directory)
            .await?
            .into())
    }

    pub async fn get_token() -> Result<rpc_auth::Token, ApiError> {
        // no caching for now. it's fast enough just to get from disk each time
        // and no need to invalidate upon connection error.
        return gen_token().await;
    }

    async fn get_network() -> Result<Network, ApiError> {
        let client = rpc_client().await?;
        let network = client.network(tarpc::context::current()).await??;
        Ok(network)
    }

    pub async fn network() -> Result<Network, ApiError> {
        // no caching for now. it's fast enough just to query from neptune-core
        // and no need to invalidate upon connection error.
        get_network().await
    }

    pub async fn send(
        outputs: Vec<OutputFormat>,
        change_policy: ChangePolicy,
        fee: NativeCurrencyAmount,
    ) -> Result<(TransactionKernelId, TransactionDetails), ApiError> {
        let serialized = bincode::serialize(&outputs).unwrap();
        let nc_outputs: Vec<neptune_cash::api::export::OutputFormat> =
            bincode::deserialize(&serialized).unwrap();

        let serialized = bincode::serialize(&change_policy).unwrap();
        let nc_change_policy: neptune_cash::api::export::ChangePolicy =
            bincode::deserialize(&serialized).unwrap();

        let serialized = bincode::serialize(&fee).unwrap();
        let nc_fee: neptune_cash::api::export::NativeCurrencyAmount =
            bincode::deserialize(&serialized).unwrap();

        let client = gen_nc_rpc_client().await?;
        let token = get_token().await?;

        let tx_artifacts = client
            .send(
                tarpc::context::current(),
                token,
                nc_outputs,
                nc_change_policy,
                nc_fee,
            )
            .await??;

        let serialized = bincode::serialize(&tx_artifacts.transaction().txid()).unwrap();
        let tx_kernel_id: TransactionKernelId = bincode::deserialize(&serialized).unwrap();

        let serialized = bincode::serialize(tx_artifacts.details()).unwrap();
        let tx_details: TransactionDetails = bincode::deserialize(&serialized).unwrap();
        Ok((tx_kernel_id, tx_details))
    }

    // fn tx_artifacts_to_tx_details(tx_artifacts: TxCreationArtifacts) -> Result<TransactionDetails, ApiError> {
    //     let json = serde_json::to_string(tx_artifacts.details())?;
    //     let tx_details: TransactionDetails = serde_json::from_str(&json)?;
    //     Ok(tx_details)
    // }
}

//! This crate contains all shared fullstack server functions.

#[cfg(not(target_arch = "wasm32"))]
mod rpc_api;
use dioxus::prelude::server_fn::codec::Json;
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
use neptune_types::timestamp::Timestamp;
use neptune_types::transaction_details::TransactionDetails;
use neptune_types::transaction_kernel::TransactionKernel;
use neptune_types::transaction_kernel_id::TransactionKernelId;
use twenty_first::tip5::Digest;

/// Echo the user input on the server.
#[server(Echo)]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(format!("{}", input))
}

#[server(NetworkApi, input = Json, output = Json)]
pub async fn network() -> Result<Network, ServerFnError> {
    neptune_rpc::network().await
}

#[server(input = Json, output = Json)]
pub async fn wallet_balance() -> Result<NativeCurrencyAmount, ServerFnError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let balance = client
        .confirmed_available_balance(tarpc::context::current(), *token)
        .await??;

    let json = serde_json_wasm::to_string(&balance).unwrap();
    dioxus_logger::tracing::info!("balance json: {}", json);

    Ok(balance)
}

#[server(BlockHeightApi, input = Json, output = Json)]
pub async fn block_height() -> Result<BlockHeight, ServerFnError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let height = client
        .block_height(tarpc::context::current(), *token)
        .await??;
    Ok(height.into())
}

#[server(input = Json, output = Json)]
pub async fn known_keys() -> Result<Vec<SpendingKey>, ServerFnError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let known_keys = client
        .known_keys(tarpc::context::current(), *token)
        .await??;
    Ok(known_keys)
}

#[server(input = Json, output = Json)]
pub async fn next_receiving_address(key_type: KeyType) -> Result<ReceivingAddress, ServerFnError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let address = client
        .next_receiving_address(tarpc::context::current(), *token, key_type)
        .await??;
    Ok(address)
}

#[server(SendApi, input = Json, output = Json)]
pub async fn send(
    outputs: Vec<OutputFormat>,
    change_policy: ChangePolicy,
    fee: NativeCurrencyAmount,
) -> Result<(TransactionKernelId, TransactionDetails), ServerFnError> {
    neptune_rpc::send(outputs, change_policy, fee).await
}

#[server(input = Json, output = Json)]
pub async fn history(
) -> Result<Vec<(Digest, BlockHeight, Timestamp, NativeCurrencyAmount)>, ServerFnError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let history = client.history(tarpc::context::current(), *token).await??;
    Ok(history)
}

#[server(input = Json, output = Json)]
pub async fn mempool_overview(
    start_index: usize,
    number: usize,
) -> Result<Vec<MempoolTransactionInfo>, ServerFnError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let data = client
        .mempool_overview(tarpc::context::current(), *token, start_index, number)
        .await??;
    Ok(data)
}

#[server(input = Json, output = Json)]
pub async fn mempool_tx_kernel(
    txid: TransactionKernelId,
) -> Result<Option<TransactionKernel>, ServerFnError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let data = client
        .mempool_tx_kernel(tarpc::context::current(), *token, txid)
        .await??;
    Ok(data)
}

#[server(BlockInfoApi, input = Json, output = Json)]
pub async fn block_info(selector: BlockSelector) -> Result<Option<BlockInfo>, ServerFnError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let data = client
        .block_info(tarpc::context::current(), *token, selector)
        .await??;
    Ok(data)
}

#[server(input = Json, output = Json)]
pub async fn dashboard_overview_data() -> Result<DashBoardOverviewDataFromClient, ServerFnError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let data = client
        .dashboard_overview_data(tarpc::context::current(), *token)
        .await??;
    Ok(data)
}

#[cfg(not(target_arch = "wasm32"))]
mod neptune_rpc {
    use super::rpc_api;
    use dioxus::prelude::ServerFnError;
    // use neptune_cash::api::export::Transaction;
    // use neptune_cash::api::export::TransactionDetails;

    use std::net::Ipv4Addr;
    use std::net::SocketAddr;

    use neptune_cash::rpc_auth;
    use neptune_cash::rpc_server::error::RpcError;
    use neptune_cash::rpc_server::RPCClient;

    use neptune_types::change_policy::ChangePolicy;
    use neptune_types::native_currency_amount::NativeCurrencyAmount;
    use neptune_types::network::Network;
    use neptune_types::output_format::OutputFormat;
    use neptune_types::transaction_details::TransactionDetails;
    use neptune_types::transaction_kernel_id::TransactionKernelId;

    use tarpc::client;
    use tarpc::context;
    use tarpc::tokio_serde::formats::Json;
    use tokio::sync::OnceCell;

    async fn gen_rpc_client() -> Result<rpc_api::RPCClient, ServerFnError> {
        let server_socket = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST), 29999);
        let transport = tarpc::serde_transport::tcp::connect(server_socket, Json::default).await?;

        Ok(rpc_api::RPCClient::new(client::Config::default(), transport).spawn())
    }

    async fn gen_nc_rpc_client() -> Result<RPCClient, ServerFnError> {
        let server_socket = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST), 29999);
        let transport = tarpc::serde_transport::tcp::connect(server_socket, Json::default).await?;

        Ok(RPCClient::new(client::Config::default(), transport).spawn())
    }

    pub async fn rpc_client() -> Result<&'static rpc_api::RPCClient, ServerFnError> {
        static STATE: OnceCell<Result<rpc_api::RPCClient, ServerFnError>> = OnceCell::const_new();

        STATE
            .get_or_init(|| async { gen_rpc_client().await })
            .await
            .as_ref()
            .map_err(|err| err.clone())
    }

    pub async fn cookie_hint() -> Result<rpc_auth::CookieHint, ServerFnError> {
        let client = rpc_client().await?;
        Ok(client.cookie_hint(context::current()).await??)
    }

    async fn gen_token() -> Result<rpc_auth::Token, ServerFnError> {
        let hint = cookie_hint().await?;
        Ok(rpc_auth::Cookie::try_load(&hint.data_directory)
            .await?
            .into())
    }

    pub async fn get_token() -> Result<&'static rpc_auth::Token, ServerFnError> {
        static STATE: OnceCell<Result<rpc_auth::Token, ServerFnError>> = OnceCell::const_new();

        STATE
            .get_or_init(|| async { gen_token().await })
            .await
            .as_ref()
            .map_err(|err| err.clone())
    }

    async fn get_network() -> Result<Network, ServerFnError> {
        let client = rpc_client().await?;
        let token = get_token().await?;
        let network = client.network(tarpc::context::current()).await??;
        Ok(network)
    }

    pub async fn network() -> Result<Network, ServerFnError> {
        static STATE: OnceCell<Result<Network, ServerFnError>> = OnceCell::const_new();

        STATE
            .get_or_init(|| async { get_network().await })
            .await
            .as_ref()
            .map_err(|err| err.clone())
            .copied()
    }

    pub async fn send(
        outputs: Vec<OutputFormat>,
        change_policy: ChangePolicy,
        fee: NativeCurrencyAmount,
    ) -> Result<(TransactionKernelId, TransactionDetails), ServerFnError> {
        use neptune_cash::api::export::ChangePolicy;
        use neptune_cash::api::export::NativeCurrencyAmount;
        use neptune_cash::api::export::OutputFormat;

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
                *token,
                nc_outputs,
                nc_change_policy,
                nc_fee,
            )
            .await??;
        // let tx_artifacts = client.send(tarpc::context::current(), *token, vec![], , nc_fee).await??;

        let serialized = bincode::serialize(&tx_artifacts.transaction().txid()).unwrap();
        let tx_kernel_id: TransactionKernelId = bincode::deserialize(&serialized).unwrap();

        let serialized = bincode::serialize(tx_artifacts.details()).unwrap();
        let tx_details: TransactionDetails = bincode::deserialize(&serialized).unwrap();
        Ok((tx_kernel_id, tx_details))
    }

    // fn tx_artifacts_to_tx_details(tx_artifacts: TxCreationArtifacts) -> Result<TransactionDetails, ServerFnError> {
    //     let json = serde_json::to_string(tx_artifacts.details())?;
    //     let tx_details: TransactionDetails = serde_json::from_str(&json)?;
    //     Ok(tx_details)
    // }
}

// let rpc_auth::CookieHint {
//     data_directory,
//     network,
// } = get_cookie_hint(&client, &args).await;

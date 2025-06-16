//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;

/// Echo the user input on the server.
#[server(Echo)]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(format!("{}", input))
}

#[server(WalletBalance)]
pub async fn wallet_balance() -> Result<String, ServerFnError> {
    let client = neptune_rpc::rpc_client().await;
    let token = neptune_rpc::get_token().await;

    let balance = client.confirmed_available_balance(tarpc::context::current(), token).await.unwrap().unwrap();
    Ok(balance.display_n_decimals(8))
}

#[server(BlockHeight)]
pub async fn block_height() -> Result<u64, ServerFnError> {
    let client = neptune_rpc::rpc_client().await;
    let token = neptune_rpc::get_token().await;

    let height = client.block_height(tarpc::context::current(), token).await.unwrap().unwrap();
    Ok(height.into())
}


// #[server(DashboardOverview)]
// pub async fn dashboard_overview() -> Result<f32, ServerFnError> {
//     let client = rpc_client();
//     let token = get_token();

//     Ok(client.dashboard_overview_data(context::current(), token).await.unwrap().unwrap())
// }

#[cfg(not(target_arch = "wasm32"))]
mod neptune_rpc {
    use neptune_cash::rpc_server::DashBoardOverviewDataFromClient;

    use std::net::Ipv4Addr;
    use std::net::SocketAddr;

    use neptune_cash::rpc_auth;
    use neptune_cash::rpc_server::error::RpcError;
    use neptune_cash::rpc_server::RPCClient;
    use neptune_cash::config_models::network::Network;
    use neptune_cash::models::blockchain::block::block_selector::BlockSelector;

    use tarpc::client;
    use tarpc::context;
    use tarpc::tokio_serde::formats::Json;

    pub(super) async fn rpc_client() -> RPCClient {
        let server_socket = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST), 9799);
        let transport = tarpc::serde_transport::tcp::connect(server_socket, Json::default).await.unwrap();

        RPCClient::new(client::Config::default(), transport).spawn()
    }

    pub async fn cookie_hint() -> rpc_auth::CookieHint {
        let client = rpc_client().await;
        client.cookie_hint(context::current()).await.unwrap().unwrap()
    }

    pub(super) async fn get_token() -> rpc_auth::Token {
        let hint = cookie_hint().await;
        rpc_auth::Cookie::try_load(&hint.data_directory).await.unwrap().into()
    }
}

// let rpc_auth::CookieHint {
//     data_directory,
//     network,
// } = get_cookie_hint(&client, &args).await;


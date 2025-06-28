//! This crate contains all shared fullstack server functions.

#[cfg(not(target_arch = "wasm32"))]
mod rpc_api;
use dioxus::prelude::*;
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use neptune_types::block_height::BlockHeight;

/// Echo the user input on the server.
#[server(Echo)]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(format!("{}", input))
}

#[server]
pub async fn wallet_balance() -> Result<NativeCurrencyAmount, ServerFnError> {
    let client = &neptune_rpc::shared_state().await.rpc_client;
    let token = neptune_rpc::get_token().await;

    let balance = client.confirmed_available_balance(tarpc::context::current(), token).await.unwrap().unwrap();
    Ok(balance)
}

#[server(BlockHeightApi)]
pub async fn block_height() -> Result<BlockHeight, ServerFnError> {
    let client = &neptune_rpc::shared_state().await.rpc_client;
    let token = neptune_rpc::get_token().await;

    let height = client.block_height(tarpc::context::current(), token).await.unwrap().unwrap();
    Ok(height.into())
}


#[cfg(not(target_arch = "wasm32"))]
mod neptune_rpc {
    use super::rpc_api;

    use std::net::Ipv4Addr;
    use std::net::SocketAddr;

    use neptune_cash::rpc_auth;
    use neptune_cash::rpc_server::error::RpcError;
    use neptune_cash::rpc_server::RPCClient;
    use neptune_cash::config_models::network::Network;

    use tarpc::client;
    use tarpc::context;
    use tarpc::tokio_serde::formats::Json;
    use tokio::sync::OnceCell;

    pub(super) struct State {
        pub rpc_client: rpc_api::RPCClient,
    }

    pub(super) async fn gen_rpc_client() -> rpc_api::RPCClient {
        let server_socket = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST), 9799);
        let transport = tarpc::serde_transport::tcp::connect(server_socket, Json::default).await.unwrap();

        rpc_api::RPCClient::new(client::Config::default(), transport).spawn()
    }

    pub async fn gen_shared_state() -> State {
        let rpc_client = gen_rpc_client().await;

        State {
            rpc_client,
        }
    }

    pub(super) async fn shared_state() -> &'static State {
        static STATE: OnceCell<State> = OnceCell::const_new();

        STATE.get_or_init(|| async { gen_shared_state().await } ).await
    }

    pub async fn cookie_hint() -> rpc_auth::CookieHint {
        let client = &shared_state().await.rpc_client;
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


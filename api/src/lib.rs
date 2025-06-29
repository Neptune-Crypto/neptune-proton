//! This crate contains all shared fullstack server functions.

#[cfg(not(target_arch = "wasm32"))]
mod rpc_api;
use dioxus::prelude::*;
use neptune_types::address::ReceivingAddress;
use neptune_types::address::KeyType;
use neptune_types::address::BaseSpendingKey;
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use neptune_types::block_height::BlockHeight;

/// Echo the user input on the server.
#[server(Echo)]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(format!("{}", input))
}

#[server]
pub async fn wallet_balance() -> Result<NativeCurrencyAmount, ServerFnError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let balance = client.confirmed_available_balance(tarpc::context::current(), *token).await??;
    Ok(balance)
}

#[server(BlockHeightApi)]
pub async fn block_height() -> Result<BlockHeight, ServerFnError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let height = client.block_height(tarpc::context::current(), *token).await??;
    Ok(height.into())
}

#[server]
pub async fn known_keys() -> Result<Vec<BaseSpendingKey>, ServerFnError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let known_keys = client.known_keys(tarpc::context::current(), *token).await??;
    Ok(known_keys)
}

#[server]
pub async fn next_receiving_address(key_type: KeyType) -> Result<ReceivingAddress, ServerFnError> {
    let client = neptune_rpc::rpc_client().await?;
    let token = neptune_rpc::get_token().await?;

    let address = client.next_receiving_address(tarpc::context::current(), *token, key_type).await??;
    Ok(address)
}



#[cfg(not(target_arch = "wasm32"))]
mod neptune_rpc {
    use super::rpc_api;
    use dioxus::prelude::ServerFnError;

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

    async fn gen_rpc_client() -> Result<rpc_api::RPCClient, ServerFnError> {
        let server_socket = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST), 9799);
        let transport = tarpc::serde_transport::tcp::connect(server_socket, Json::default).await?;

        Ok(rpc_api::RPCClient::new(client::Config::default(), transport).spawn())
    }

    pub async fn rpc_client() -> Result<&'static rpc_api::RPCClient, ServerFnError> {
        static STATE: OnceCell<Result<rpc_api::RPCClient, ServerFnError>> = OnceCell::const_new();

        STATE.get_or_init(|| async { gen_rpc_client().await } ).await.as_ref().map_err(|err| err.clone())
    }

    pub async fn cookie_hint() -> Result<rpc_auth::CookieHint, ServerFnError> {
        let client = rpc_client().await?;
        Ok(client.cookie_hint(context::current()).await??)
    }

    async fn gen_token() -> Result<rpc_auth::Token, ServerFnError> {
        let hint = cookie_hint().await?;
        Ok(rpc_auth::Cookie::try_load(&hint.data_directory).await?.into())
    }

    pub async fn get_token() -> Result<&'static rpc_auth::Token, ServerFnError> {
        static STATE: OnceCell<Result<rpc_auth::Token, ServerFnError>> = OnceCell::const_new();

        STATE.get_or_init(|| async { gen_token().await } ).await.as_ref().map_err(|err| err.clone())
    }

}

// let rpc_auth::CookieHint {
//     data_directory,
//     network,
// } = get_cookie_hint(&client, &args).await;


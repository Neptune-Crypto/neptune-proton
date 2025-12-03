use api::ApiError;
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug, strum::EnumIs)]
pub enum NeptuneRpcConnectionStatus {
    Connected,
    Disconnected(String),
}

#[derive(Clone, Copy)]
pub struct RpcChecker {
    status: Signal<NeptuneRpcConnectionStatus>,
}

impl RpcChecker {
    /// Inspects a Result from an API call.
    /// - If `Ok`: Updates status to Connected (if previously disconnected) and returns value.
    /// - If `Err`: Checks if it's a connection error. If so, updates status to Disconnected. Returns None.
    pub fn check<T>(&mut self, result: Result<T, ApiError>) -> Option<T> {
        match result {
            Ok(val) => {
                // If we were disconnected, we are back now.
                if matches!(
                    *self.status.peek(),
                    NeptuneRpcConnectionStatus::Disconnected(_)
                ) {
                    self.status.set(NeptuneRpcConnectionStatus::Connected);
                }
                Some(val)
            }
            Err(e) => {
                let error_msg = e.to_string();
                dioxus_logger::tracing::warn!("RPC Error: {}", error_msg);

                // Heuristic: Check if this is a connection-related error.
                if self.is_connection_error(&error_msg) {
                    self.status
                        .set(NeptuneRpcConnectionStatus::Disconnected(error_msg));
                    None
                } else {
                    None
                }
            }
        }
    }

    /// Checks a result by reference without consuming it.
    /// Returns `true` if the result is Ok.
    /// If Err, checks if it is a connection error and updates global status if so.
    pub fn check_result_ref<T, E: std::fmt::Display>(&mut self, result: &Result<T, E>) -> bool {
        match result {
            Ok(_) => {
                // If we were disconnected, we are back now.
                if matches!(
                    *self.status.peek(),
                    NeptuneRpcConnectionStatus::Disconnected(_)
                ) {
                    self.status.set(NeptuneRpcConnectionStatus::Connected);
                }
                true
            }
            Err(e) => {
                let error_msg = e.to_string();
                // Only log warnings if it looks like a connection drop, otherwise it might just be valid logic flow
                if self.is_connection_error(&error_msg) {
                    dioxus_logger::tracing::warn!("RPC Error (Ref): {}", error_msg);
                    self.status
                        .set(NeptuneRpcConnectionStatus::Disconnected(error_msg));
                }
                false
            }
        }
    }

    /// Returns the read-only signal for the connection status.
    /// Call .read() on this in a component/resource to subscribe to changes.
    pub fn status(&self) -> Signal<NeptuneRpcConnectionStatus> {
        self.status
    }

    fn is_connection_error(&self, msg: &str) -> bool {
        let msg = msg.to_lowercase();
        msg.contains("connection refused")
            || msg.contains("broken pipe")
            || msg.contains("network unreachable")
            || msg.contains("connection reset")
            || msg.contains("failed to connect")
            || msg.contains("rpc client unavailable")
            // Dioxus/Hyper specific transport errors
            || msg.contains("error running server function")
            || msg.contains("connection to the server was already shutdown")
            || msg.contains("channel closed")
    }
}

pub fn use_rpc_checker() -> RpcChecker {
    let status = use_context::<Signal<NeptuneRpcConnectionStatus>>();
    RpcChecker { status }
}

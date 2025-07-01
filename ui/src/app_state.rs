use neptune_types::network::Network;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq)]
pub struct AppStateData {
    pub network: Network,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppState(Arc<AppStateData>);

impl Deref for AppState {
    type Target = AppStateData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AppState {
    pub fn new(network: Network) -> Self {
        Self(Arc::new(AppStateData { network }))
    }
}

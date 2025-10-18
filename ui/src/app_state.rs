use api::price_map::PriceMap;
use neptune_types::network::Network;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq)]
pub struct AppStateData {
    pub network: Network,
    pub price_map: PriceMap,
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
        Self(Arc::new(AppStateData {
            network,
            price_map: Default::default(),
        }))
    }
}

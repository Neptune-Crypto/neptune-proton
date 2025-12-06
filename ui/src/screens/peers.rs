// File: src/screens/peers.rs

// Added for SocketAddr and IpAddr types
use std::net::IpAddr;
use std::net::SocketAddr;
#[cfg(not(target_arch = "wasm32"))]
use std::time::SystemTime;
#[cfg(not(target_arch = "wasm32"))]
use std::time::UNIX_EPOCH;

use chrono::NaiveDateTime;
use chrono::TimeZone;
use chrono::Utc;
use dioxus::prelude::*;
use neptune_types::peer_info::PeerInfo;
#[cfg(target_arch = "wasm32")]
use web_time::SystemTime;
#[cfg(target_arch = "wasm32")]
use web_time::UNIX_EPOCH;
use std::time::Duration;

use crate::components::pico::Card;
use crate::hooks::use_rpc_checker::use_rpc_checker;

#[derive(Clone, Copy, PartialEq)]
enum SortableColumn {
    Ip,
    Version,
    Established,
    Standing,
    LastPunishment,
    LastReward,
}

#[derive(Clone, Copy, PartialEq)]
enum SortDirection {
    Ascending,
    Descending,
}

fn format_sanction(sanction_info: Option<(impl ToString, SystemTime)>) -> String {
    match sanction_info {
        Some((sanction, time)) => {
            let duration = SystemTime::now().duration_since(time).unwrap_or_default();
            let secs = duration.as_secs();
            format!("{} ({}s ago)", sanction.to_string(), secs)
        }
        None => "N/A".to_string(),
    }
}

/// Formats a SocketAddr to display IPv4-mapped addresses as plain IPv4.
fn format_socket_addr(addr: SocketAddr) -> String {
    match addr {
        SocketAddr::V4(addr_v4) => addr_v4.to_string(),
        SocketAddr::V6(addr_v6) => {
            // Call .ip() to get the Ipv6Addr, which has the to_ipv4_mapped method.
            if let Some(addr_v4) = addr_v6.ip().to_ipv4_mapped() {
                // Construct a new SocketAddrV4 to get the correct formatting.
                std::net::SocketAddrV4::new(addr_v4, addr_v6.port()).to_string()
            } else {
                addr_v6.to_string()
            }
        }
    }
}

/// Returns a canonical IpAddr, converting IPv4-mapped V6 addresses to V4 for consistent sorting.
fn get_canonical_ip(addr: &SocketAddr) -> IpAddr {
    match addr.ip() {
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                IpAddr::V4(v4)
            } else {
                IpAddr::V6(v6)
            }
        }
        ip => ip,
    }
}

#[component]
fn SortableHeader(
    title: &'static str,
    column: SortableColumn,
    sort_column: Signal<SortableColumn>,
    sort_direction: Signal<SortDirection>,
) -> Element {
    let (arrow_char, is_active) = if *sort_column.read() == column {
        (
            match *sort_direction.read() {
                SortDirection::Ascending => "▲",
                SortDirection::Descending => "▼",
            },
            true,
        )
    } else {
        ("\u{00A0}", false)
    };

    rsx! {
        th {
            style: "position: sticky; top: 0; background: var(--pico-card-background-color); cursor: pointer; white-space: nowrap;",
            onclick: move |_| {
                if is_active {
                    sort_direction
                        .with_mut(|dir| {
                            *dir = match dir {
                                SortDirection::Ascending => SortDirection::Descending,
                                SortDirection::Descending => SortDirection::Ascending,
                            };
                        });
                } else {
                    sort_column.set(column);
                    sort_direction.set(SortDirection::Ascending);
                }
            },
            "{title}"
            span {
                style: "display: inline-block; width: 1.2em; text-align: right;",
                "{arrow_char}"
            }
        }
    }
}

#[component]
fn EstablishedCell(time: SystemTime) -> Element {
    let duration_since_epoch = time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    #[allow(deprecated)]
    let naive_datetime = NaiveDateTime::from_timestamp_opt(
        duration_since_epoch.as_secs() as i64,
        duration_since_epoch.subsec_nanos(),
    )
    .unwrap();
    let established_utc = Utc.from_utc_datetime(&naive_datetime);
    let established_local = established_utc.with_timezone(&chrono::Local);
    let date = established_local.format("%Y-%m-%d").to_string();
    let hour = established_local.format("%H:%M:%S").to_string();

    let elapsed_time_secs = Duration::from_secs(SystemTime::now()
        .duration_since(time)
        .unwrap_or_default()
        .as_secs());

    let human_duration = humantime::format_duration(elapsed_time_secs);

    let seconds_ago = SystemTime::now()
        .duration_since(time)
        .unwrap_or_default()
        .as_secs();

    rsx! {
        td {
            title: "{human_duration}",
            "{date}"
            br {}
            "{hour}"
        }
    }
}

#[component]
pub fn PeersScreen() -> Element {
    let mut rpc = use_rpc_checker(); // Initialize Hook

    let mut peer_info = use_resource(move || async move { api::peer_info().await });

    // Effect: Restarts the resource when connection is restored.
    let status_sig = rpc.status();
    use_effect(move || {
        if status_sig.read().is_connected() {
            peer_info.restart();
        }
    });

    // for refreshing from neptune-core every N secs
    use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let rpc_status = rpc.status(); // Use signal handle
        let mut data_resource = peer_info;

        async move {
            loop {
                // Wait 60 seconds
                crate::compat::sleep(std::time::Duration::from_secs(60)).await;

                // Only restart the resource if we are currently connected.
                // When connection is lost, rpc_status.read() will be Disconnected,
                // and we rely on the resource's *dependency* on rpc.status().read()
                // (in the resource closure) to trigger the restart when it comes back.
                if (*rpc_status.read()).is_connected() {
                    data_resource.restart();
                }
            }
        }
    });

    let sort_column = use_signal(|| SortableColumn::Standing);
    let sort_direction = use_signal(|| SortDirection::Descending);

    rsx! {
        match &*peer_info.read() {
            None => rsx! {
                Card {

                    h3 {

                        "Connected Peers"
                    }
                    p {

                        "Loading..."
                    }
                    progress {


                    }
                }
            },
            // check if neptune-core rpc connection lost
            Some(result) if !rpc.check_result_ref(&result) => rsx! {
                // modal ConnectionLost is displayed by rpc.check_result_ref
                Card {
                    h3 {
                        "Connected Peers"
                    }
                }
            },
            Some(Err(e)) => rsx! {
                Card {

                    h3 {

                        "Error"
                    }
                    p {

                        "Failed to load peer data: {e}"
                    }
                    button {
                        onclick: move |_| peer_info.restart(),
                        "Retry"
                    }
                }
            },
            Some(Ok(peers)) => {
                let mut sorted_peers = peers.clone();
                sorted_peers
                    .sort_by(|a: &PeerInfo, b: &PeerInfo| {
                        let ordering = match sort_column() {
                            SortableColumn::Ip => {
                                get_canonical_ip(&a.connected_address())
                                    .cmp(&get_canonical_ip(&b.connected_address()))
                            }
                            SortableColumn::Version => a.version().cmp(b.version()),
                            SortableColumn::Established => {
                                a.connection_established().cmp(&b.connection_established())
                            }
                            SortableColumn::Standing => {
                                a.standing.standing.cmp(&b.standing.standing)
                            }
                            SortableColumn::LastPunishment => {
                                a.standing
                                    .latest_punishment
                                    .map(|p| p.1)
                                    .cmp(&b.standing.latest_punishment.map(|p| p.1))
                            }
                            SortableColumn::LastReward => {
                                a.standing
                                    .latest_reward
                                    .map(|r| r.1)
                                    .cmp(&b.standing.latest_reward.map(|p| p.1))
                            }
                        };
                        match sort_direction() {
                            SortDirection::Ascending => ordering,
                            SortDirection::Descending => ordering.reverse(),
                        }
                    });
                rsx! {
                    Card {

                        h3 {

                            "Connected Peers ({peers.len()})"
                        }
                        div {
                            style: "max-height: 70vh; overflow-y: auto;",
                            table {

                                thead {

                                    tr {

                                        SortableHeader {
                                            title: "IP Address",
                                            column: SortableColumn::Ip,
                                            sort_column,
                                            sort_direction,
                                        }
                                        SortableHeader {
                                            title: "Version",
                                            column: SortableColumn::Version,
                                            sort_column,
                                            sort_direction,
                                        }
                                        SortableHeader {
                                            title: "Established",
                                            column: SortableColumn::Established,
                                            sort_column,
                                            sort_direction,
                                        }
                                        SortableHeader {
                                            title: "Standing",
                                            column: SortableColumn::Standing,
                                            sort_column,
                                            sort_direction,
                                        }
                                        SortableHeader {
                                            title: "Last Punishment",
                                            column: SortableColumn::LastPunishment,
                                            sort_column,
                                            sort_direction,
                                        }
                                        SortableHeader {
                                            title: "Last Reward",
                                            column: SortableColumn::LastReward,
                                            sort_column,
                                            sort_direction,
                                        }
                                    }
                                }
                                tbody {

                                    for peer in sorted_peers.iter() {
                                        tr {

                                            td {

                                                code {

                                                    "{format_socket_addr(peer.connected_address())}"
                                                }
                                            }
                                            td {

                                                "{peer.version()}"
                                            }
                                            EstablishedCell {
                                                time: peer.connection_established(),
                                            }
                                            td {

                                                "{peer.standing.standing}"
                                            }
                                            td {

                                                "{format_sanction(peer.standing.latest_punishment)}"
                                            }
                                            td {

                                                "{format_sanction(peer.standing.latest_reward)}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

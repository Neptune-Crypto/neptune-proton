// File: src/screens/peers.rs

use std::net::IpAddr;
use std::net::SocketAddr;
use std::rc::Rc;
use std::time::Duration;
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

use crate::components::empty_state::EmptyState;
use crate::components::pico::Card;
use crate::components::pico::{Button, ButtonType, NoTitleModal};
use crate::hooks::use_rpc_checker::use_rpc_checker;

// Embed the SVG content as a static string at compile time.
const PEERS_EMPTY_SVG: &str = include_str!("../../assets/svg/peers-empty.svg");

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

// Props for the modal content
#[derive(Clone)]
struct ClearStandingModalContentProps {
    peer_ip: Option<IpAddr>,
    show_modal: Signal<bool>,
    on_success: std::rc::Rc<dyn Fn()>,
}

impl PartialEq for ClearStandingModalContentProps {
    fn eq(&self, other: &Self) -> bool {
        // Skip comparison for Rc<dyn Fn()>.
        self.peer_ip == other.peer_ip && self.show_modal == other.show_modal
    }
}

// Component containing the modal's internal logic and buttons
fn ClearStandingModalContent(props: ClearStandingModalContentProps) -> Element {
    let peer_ip = props.peer_ip;
    let mut show_modal = props.show_modal;
    let on_success = props.on_success;

    let mut clear_status = use_signal::<Option<Result<(), String>>>(|| None);
    let mut api_in_progress = use_signal(|| false);

    let action_title = match peer_ip {
        Some(ip) => format!("IP {}", ip),
        None => "All Peers".to_string(),
    };

    let ip_to_clear = peer_ip;

    let handle_clear = move |_| {
        if *api_in_progress.read() {
            return;
        }

        api_in_progress.set(true);
        clear_status.set(None);

        let on_success = on_success.clone();
        let mut show_modal = show_modal.clone();
        let ip_to_clear = ip_to_clear; // Capture the IP value

        spawn(async move {
            let result = match ip_to_clear {
                Some(ip) => api::clear_standing_by_ip(ip)
                    .await
                    .map_err(|e| format!("API Error: {}", e)),
                None => api::clear_all_standings()
                    .await
                    .map_err(|e| format!("API Error: {}", e)),
            };

            api_in_progress.set(false);

            let is_success = result.is_ok();
            clear_status.set(Some(result));

            if is_success {
                show_modal.set(false);
                on_success();
            }
        });
    };

    let handle_close = move |_| {
        show_modal.set(false);
        clear_status.set(None);
    };

    let error_message = clear_status
        .read()
        .as_ref()
        .and_then(|res| res.as_ref().err().cloned());

    rsx! {
        div {

            header {
                h3 {
                    "Clear Peer Standings"
                }
            }

            if let Some(err) = error_message {
                p { "Error clearing standing." }
                p { "Details: {err}" }
                footer {
                    Button {
                        button_type: ButtonType::Secondary,
                        on_click: handle_close,
                        "Close"
                    }
                }
            } else {
                p { "Are you sure you want to clear the standing for:" }
                ul {
                    li { b { "{action_title}" } }
                }

                footer {
                    Button {
                        button_type: ButtonType::Secondary,
                        on_click: handle_close,
                        disabled: *api_in_progress.read(),
                        style: "margin-right: 1rem;",
                        "Cancel"
                    }
                    Button {
                        button_type: ButtonType::Primary,
                        on_click: handle_clear,
                        disabled: *api_in_progress.read(),
                        {
                            if *api_in_progress.read() {
                                rsx! { "Clearing..." }
                            } else {
                                rsx! { "Confirm Clear" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ClearStandingCell(
    /// Content to display in the cell (e.g., IP address or sanction info).
    display_content: Element,
    /// The SocketAddr of the peer.
    peer_addr: SocketAddr,
    /// Signal to control modal visibility.
    show_modal: Signal<bool>,
    /// Signal to set the IP address for the modal.
    modal_ip: Signal<Option<IpAddr>>,
) -> Element {
    let canonical_ip = get_canonical_ip(&peer_addr);

    rsx! {
        td {
            style: "cursor: pointer;",
            onclick: move |_| {
                modal_ip.set(Some(canonical_ip));
                show_modal.set(true);
            },
            {display_content}
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

    let elapsed_time_secs = Duration::from_secs(
        SystemTime::now()
            .duration_since(time)
            .unwrap_or_default()
            .as_secs(),
    );

    let human_duration = humantime::format_duration(elapsed_time_secs);

    let _seconds_ago = SystemTime::now()
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

    // Resource type explicitly targets Vec<PeerInfo> with a String error type,
    // and maps the internal error to String for consistency.
    let mut peer_info: Resource<Result<Vec<PeerInfo>, String>> =
        use_resource(move || async move { api::peer_info().await.map_err(|e| e.to_string()) });

    // Clone the resource handle for the immutable Fn() closure
    let peer_info_handle = peer_info.clone();

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

    // MODAL STATE:
    let mut show_clear_standing_modal = use_signal(|| false);
    let mut modal_peer_ip = use_signal::<Option<IpAddr>>(|| None);

    // ACTION/CONTROL LOGIC:
    let refresh_data_on_success = Rc::new(move || {
        peer_info_handle.clone().restart();
    }) as Rc<dyn Fn()>;

    rsx! {
        // MODAL RENDER: Using the imported NoTitleModal component
        if *show_clear_standing_modal.read() {
            NoTitleModal {
                is_open: show_clear_standing_modal,
                children: rsx! {
                    {
                        ClearStandingModalContent(ClearStandingModalContentProps {
                            peer_ip: *modal_peer_ip.read(),
                            show_modal: show_clear_standing_modal,
                            on_success: refresh_data_on_success.clone(),
                        })
                    }
                }
            }
        }

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
                    Button {
                        on_click: move |_| peer_info.restart(),
                        "Retry"
                    }
                }
            },
            Some(Ok(peers)) if peers.is_empty() => rsx! {
                Card {

                    h3 {
                        "Connected Peers"
                    }

                    EmptyState {
                        title: "No Peers Connected".to_string(),
                        description: Some("Your node is currently scanning the network. New connections will appear here automatically.".to_string()),
                        icon: rsx! {
                            // Inject the SVG string directly into the DOM
                            span {
                                dangerous_inner_html: PEERS_EMPTY_SVG,
                                style: "width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;",
                            }
                        }
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
                        div {
                            // MODIFIED: Added align-items: center and adjusted margins for vertical alignment
                            style: "display: flex; align-items: center; width: 100%;",

                            h3 {
                                style: "margin-right: 0.5rem; margin-bottom: 0;",
                                "Connected Peers"
                            }
                            small {
                                style: "font-weight: normal; font-size: 0.8rem; color: var(--pico-muted-color);",
                                "({peers.len()})"
                            }
                            // Added button to clear all standings
                            Button {
                                button_type: ButtonType::Secondary,
                                outline: true,
                                // RESTORED inline styles for small button size
                                style: "margin-left: auto; margin-right: 0; padding: 0.2rem 0.5rem; font-size: 0.8rem;",
                                title: "Resets standing scores for all connected peers back to zero",
                                on_click: move |_| {
                                    modal_peer_ip.set(None); // Set to None for "All Peers"
                                    show_clear_standing_modal.set(true);
                                },
                                "Clear All Standings"
                            }
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

                                            // Fixed: Use peer.connected_address() directly
                                            ClearStandingCell {
                                                display_content: rsx! {
                                                    code {
                                                        "{format_socket_addr(peer.connected_address())}"
                                                    }
                                                },
                                                peer_addr: peer.connected_address(),
                                                show_modal: show_clear_standing_modal,
                                                modal_ip: modal_peer_ip,
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
                                            // Fixed: Use peer.connected_address() directly
                                            ClearStandingCell {
                                                display_content: rsx! { "{format_sanction(peer.standing.latest_punishment)}" },
                                                peer_addr: peer.connected_address(),
                                                show_modal: show_clear_standing_modal,
                                                modal_ip: modal_peer_ip,
                                            }
                                            // Fixed: Use peer.connected_address() directly
                                            ClearStandingCell {
                                                display_content: rsx! { "{format_sanction(peer.standing.latest_reward)}" },
                                                peer_addr: peer.connected_address(),
                                                show_modal: show_clear_standing_modal,
                                                modal_ip: modal_peer_ip,
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

// File: src/screens/peers.rs

#[cfg(target_arch = "wasm32")]
use web_time::{SystemTime, UNIX_EPOCH};

#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

use crate::components::pico::Card;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use dioxus::prelude::*;
use neptune_types::peer_info::PeerInfo;
use std::cmp::Ordering;

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
            let duration = SystemTime::now()
                .duration_since(time)
                .unwrap_or_default();
            let secs = duration.as_secs();
            format!("{} ({}s ago)", sanction.to_string(), secs)
        }
        None => "N/A".to_string(),
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
            // Add `white-space: nowrap;` to prevent the header text from wrapping
            style: "position: sticky; top: 0; background: var(--pico-card-background-color); cursor: pointer; white-space: nowrap;",
            onclick: move |_| {
                if is_active {
                    sort_direction.with_mut(|dir| *dir = match dir {
                        SortDirection::Ascending => SortDirection::Descending,
                        SortDirection::Descending => SortDirection::Ascending,
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
    let naive_datetime = NaiveDateTime::from_timestamp_opt(
        duration_since_epoch.as_secs() as i64,
        duration_since_epoch.subsec_nanos(),
    )
    .unwrap();
    let established_utc = Utc.from_utc_datetime(&naive_datetime);
    let established_local = established_utc.with_timezone(&chrono::Local);
    let formatted_timestamp = established_local.format("%Y-%m-%d %H:%M:%S").to_string();
    let seconds_ago = SystemTime::now().duration_since(time).unwrap_or_default().as_secs();

    rsx! {
        td {
            title: "{formatted_timestamp}",
            "{seconds_ago}s ago"
        }
    }
}


#[component]
pub fn PeersScreen() -> Element {
    let mut peer_info = use_resource(move || async move { api::peer_info().await });

    let mut sort_column = use_signal(|| SortableColumn::Standing);
    let mut sort_direction = use_signal(|| SortDirection::Descending);

    rsx! {
        match &*peer_info.read() {
            None => rsx! {
                Card {
                    h3 { "Connected Peers" }
                    p { "Loading..." }
                    progress {}
                }
            },
            Some(Err(e)) => rsx! {
                Card {
                    h3 { "Error" }
                    p { "Failed to load peer data: {e}" }
                    button { onclick: move |_| peer_info.restart(), "Retry" }
                }
            },
            Some(Ok(peers)) => {
                let mut sorted_peers = peers.clone();

                sorted_peers.sort_by(|a: &PeerInfo, b: &PeerInfo| {
                    let ordering = match sort_column() {
                        SortableColumn::Ip => a.connected_address().ip().cmp(&b.connected_address().ip()),
                        SortableColumn::Version => a.version().cmp(b.version()),
                        SortableColumn::Established => a.connection_established().cmp(&b.connection_established()),
                        SortableColumn::Standing => a.standing.standing.cmp(&b.standing.standing),
                        SortableColumn::LastPunishment => a.standing.latest_punishment.map(|p| p.1).cmp(&b.standing.latest_punishment.map(|p| p.1)),
                        SortableColumn::LastReward => a.standing.latest_reward.map(|r| r.1).cmp(&b.standing.latest_reward.map(|p| p.1)),
                    };

                    match sort_direction() {
                        SortDirection::Ascending => ordering,
                        SortDirection::Descending => ordering.reverse(),
                    }
                });


                rsx! {
                    Card {
                        h3 { "Connected Peers ({peers.len()})" }

                        div {
                            style: "max-height: 70vh; overflow-y: auto;",
                            table {
                                thead {
                                    tr {
                                        SortableHeader { title: "IP Address", column: SortableColumn::Ip, sort_column, sort_direction }
                                        SortableHeader { title: "Version", column: SortableColumn::Version, sort_column, sort_direction }
                                        SortableHeader { title: "Established", column: SortableColumn::Established, sort_column, sort_direction }
                                        SortableHeader { title: "Standing", column: SortableColumn::Standing, sort_column, sort_direction }
                                        SortableHeader { title: "Last Punishment", column: SortableColumn::LastPunishment, sort_column, sort_direction }
                                        SortableHeader { title: "Last Reward", column: SortableColumn::LastReward, sort_column, sort_direction }
                                    }
                                }
                                tbody {
                                    for peer in sorted_peers.iter() {
                                        tr {
                                            td { code { "{peer.connected_address()}" } }
                                            td { "{peer.version()}" }
                                            EstablishedCell { time: peer.connection_established() }
                                            td { "{peer.standing.standing}" }
                                            td { "{format_sanction(peer.standing.latest_punishment)}" }
                                            td { "{format_sanction(peer.standing.latest_reward)}" }
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
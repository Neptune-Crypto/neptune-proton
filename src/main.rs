// The main application file, now structured to use external component modules.

use dioxus::prelude::*;

// Declare the modules that contain our components.
mod components;
mod screens;

// Use the Container component from our pico library.
use components::pico::Container;

// Use all the screen components we just created.
use screens::{
    addresses::AddressesScreen, balance::BalanceScreen, blockchain::BlockChainScreen,
    history::HistoryScreen, mempool::MempoolScreen, receive::ReceiveScreen, send::SendScreen,
};

/// Enum to represent the different screens in our application.
#[derive(Clone, PartialEq, Default)]
enum Screen {
    #[default]
    Balance,
    Send,
    Receive,
    History,
    Addresses,
    BlockChain,
    Mempool,
}

impl Screen {
    /// Helper to get the display name for each screen.
    fn name(&self) -> &'static str {
        match self {
            Screen::Balance => "Balance",
            Screen::Send => "Send",
            Screen::Receive => "Receive",
            Screen::History => "History",
            Screen::Addresses => "Addresses",
            Screen::BlockChain => "BlockChain",
            Screen::Mempool => "Mempool",
        }
    }
}

/// A list of all available screens for easy iteration.
const ALL_SCREENS: [Screen; 7] = [
    Screen::Balance,
    Screen::Send,
    Screen::Receive,
    Screen::History,
    Screen::Addresses,
    Screen::BlockChain,
    Screen::Mempool,
];

/// The desktop navigation tabs component.
#[component]
fn Tabs(active_screen: Signal<Screen>) -> Element {
    rsx! {
        nav {
            class: "hide-on-mobile",
            ul {
                for screen in ALL_SCREENS {
                    li {
                        a {
                            href: "#",
                            "aria-current": if *active_screen.read() == screen { "page" } else { "false" },
                            onclick: move |_| active_screen.set(screen.clone()),
                            "{screen.name()}"
                        }
                    }
                }
            }
        }
    }
}

/// The mobile "hamburger" dropdown menu component.
#[component]
fn HamburgerMenu(active_screen: Signal<Screen>) -> Element {
    rsx! {
        nav {
            class: "hide-on-desktop",
            details {
                class: "dropdown",
                summary { role: "button", class: "secondary", "☰ Menu" }
                ul {
                    dir: "rtl",
                    for screen in ALL_SCREENS {
                        li {
                            a {
                                href: "#",
                                onclick: move |_| active_screen.set(screen.clone()),
                                "{screen.name()}"
                            }
                        }
                    }
                }
            }
        }
    }
}

//=============================================================================
// MAIN APPLICATION
//=============================================================================
fn main() {
    launch(App);
}

#[allow(non_snake_case)]
fn App() -> Element {
    let mut active_screen = use_signal(Screen::default);

    let responsive_css = r#"
        @media (min-width: 768px) { .hide-on-desktop { display: none; } }
        @media (max-width: 767px) { .hide-on-mobile { display: none; } }
    "#;

    rsx! {
        document::Stylesheet { href: asset!("/assets/css/pico.cyan.min.css") }
        style { "{responsive_css}" }

        Container {
            header {
                nav {
                    ul {
                        li { h1 { style: "margin: 0; font-size: 1.5rem;", "Dioxus Crypto Wallet" } }
                    }
                    ul {
                        li {
                            Tabs { active_screen: active_screen }
                            HamburgerMenu { active_screen: active_screen }
                        }
                    }
                }
            }

            // Main Content Area
            div {
                class: "content",
                match active_screen() {
                    Screen::Balance => rsx!{ BalanceScreen {} },
                    Screen::Send => rsx!{ SendScreen {} },
                    Screen::Receive => rsx!{ ReceiveScreen {} },
                    Screen::History => rsx!{ HistoryScreen {} },
                    Screen::Addresses => rsx!{ AddressesScreen {} },
                    Screen::BlockChain => rsx!{ BlockChainScreen {} },
                    Screen::Mempool => rsx!{ MempoolScreen {} },
                }
            }
        }
    }
}

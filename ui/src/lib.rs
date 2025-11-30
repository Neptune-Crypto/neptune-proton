// The client-side Dioxus application logic.

use dioxus::prelude::*;

mod app_state;
mod app_state_mut;
pub mod compat;
mod components;
mod currency;
pub mod hooks;
mod screens;

use api::prefs::user_prefs::UserPrefs;
use api::price_map::PriceMap;
use app_state::AppState;
use app_state_mut::AppStateMut;
use components::pico::Button;
use components::pico::ButtonType;
use components::pico::Container;
use neptune_types::block_selector::BlockSelector;
use neptune_types::transaction_kernel_id::TransactionKernelId;
use screens::addresses::AddressesScreen;
use screens::balance::BalanceScreen;
use screens::block::BlockScreen;
use screens::blockchain::BlockChainScreen;
use screens::history::HistoryScreen;
use screens::mempool::MempoolScreen;
use screens::mempool_tx::MempoolTxScreen;
use screens::peers::PeersScreen;
use screens::receive::ReceiveScreen;
use screens::send::SendScreen;

/// Enum to represent the different screens in our application.
#[derive(Clone, PartialEq, Default)]
enum Screen {
    #[default]
    Balance,
    Send,
    Receive,
    History,
    Addresses,
    Peers,
    BlockChain,
    Mempool,
    MempoolTx(TransactionKernelId),
    Block(BlockSelector),
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
            Screen::Peers => "Peers",
            Screen::BlockChain => "BlockChain",
            Screen::Mempool => "Mempool",
            Screen::MempoolTx(_) => "Mempool Transaction",
            Screen::Block(_) => "Block",
        }
    }
}

/// Enum to represent the current view mode (for simulation).
#[derive(Clone, PartialEq, Default)]
enum ViewMode {
    #[default]
    Desktop,
    Mobile,
}

/// A list of all available screens for easy iteration.
const ALL_SCREENS: [Screen; 8] = [
    Screen::Balance,
    Screen::Send,
    Screen::Receive,
    Screen::History,
    Screen::Addresses,
    Screen::Peers,
    Screen::BlockChain,
    Screen::Mempool,
];
/// The desktop navigation tabs component.
#[component]
fn Tabs(active_screen: Signal<Screen>) -> Element {
    rsx! {
        nav {
            class: "tab-menu",
            ul {
                for screen in ALL_SCREENS {
                    li {
                        a {
                            href: "#",
                            // LOGIC FIX: Determine active state including nested screens
                            class: {
                                let is_active = match (&*active_screen.read(), &screen) {
                                    (Screen::MempoolTx(_), Screen::Mempool) => true,
                                    (Screen::Block(_), Screen::BlockChain) => true,
                                    (active, current) => active == current,
                                };
                                if is_active { "active-tab" } else { "" }
                            },
                            "aria-current": {
                                let is_active = match (&*active_screen.read(), &screen) {
                                    (Screen::MempoolTx(_), Screen::Mempool) => true,
                                    (Screen::Block(_), Screen::BlockChain) => true,
                                    (active, current) => active == current,
                                };
                                if is_active { "page" } else { "false" }
                            },
                            onclick: move |event| {
                                event.prevent_default();
                                active_screen.set(screen.clone());
                            },
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
fn HamburgerMenu(active_screen: Signal<Screen>, view_mode: Signal<ViewMode>) -> Element {
    let mut is_open = use_signal(|| false);

    rsx! {
        div {
            class: "hamburger-menu-container",
            Button {
                button_type: ButtonType::Secondary,
                outline: true,
                on_click: move |_| is_open.toggle(),
                "≡"
            }
            if is_open() {
                div {
                    class: "menu-backdrop",
                    onclick: move |_| is_open.set(false),
                }
                article {
                    class: "custom-dropdown-menu",
                    for screen in ALL_SCREENS {
                        a {
                            // LOGIC FIX: Apply active class to mobile items too using fuzzy match
                            class: {
                                let is_active = match (&*active_screen.read(), &screen) {
                                    (Screen::MempoolTx(_), Screen::Mempool) => true,
                                    (Screen::Block(_), Screen::BlockChain) => true,
                                    (active, current) => active == current,
                                };
                                if is_active { "custom-dropdown-item active-tab" } else { "custom-dropdown-item" }
                            },
                            href: "#",
                            onclick: move |event| {
                                event.prevent_default();
                                active_screen.set(screen.clone());
                                is_open.set(false);
                            },
                            "{screen.name()}"
                        }
                    }
                    hr {}
                    a {
                        class: "custom-dropdown-item",
                        href: "#",
                        onclick: move |event| {
                            event.prevent_default();
                            view_mode.set(ViewMode::Desktop);
                            is_open.set(false);
                        },
                        "Desktop View"
                    }
                }
            }
        }
    }
}

//=============================================================================
// MAIN APPLICATION COMPONENT (Client-side)
//=============================================================================

#[allow(non_snake_case)]
pub fn App() -> Element {
    // CSS FIX: Added styling for .active-tab in both desktop (tab-menu) and mobile contexts
    let responsive_css = r#"
    /* --- RESET --- */
    * { box-sizing: border-box; }

    html, body {
        height: 100%;
        width: 100%;
        margin: 0;
        padding: 0;
        overflow: hidden;
        background-color: var(--muted-border-color);
    }

    /* --- APP FRAME --- */
    .app-main-container {
        position: fixed;
        top: 0; left: 0; right: 0; bottom: 0;
        padding: 10px; /* Margin from window edge */

        display: flex;
        flex-direction: column;
        overflow: hidden;
        background-color: var(--background-color);
        z-index: 100;
    }

    /* --- PICO CONTAINER OVERRIDE --- */
    .app-main-container > * {
        flex: 1;
        display: flex !important;
        flex-direction: column;
        height: 100%;
        min-height: 0;
        overflow: hidden;

        margin: 0 !important;
        width: 100% !important;
        max-width: 100% !important;
    }

    .app-main-container header {
        flex-shrink: 0;
        padding: 0 1rem;
        margin-bottom: 0;
        border solid black 1px;
        --pico-nav-element-spacing-vertical: 0.5rem;
    }

    /* Active Tab: Rounded corners + Simulated Fading Borders */
    .tab-menu a.active-tab {
        color: var(--pico-primary) !important;
        text-decoration: none;
        opacity: 1 !important;

        /* 1. The Shape */
        border-radius: 10px 10px 0 0; /* Rounded top corners */
        border: none;                 /* clear standard borders */

        /* 2. Top Border (Real border, allows curving) */
        /* 90% Transparent (10% opacity) - slightly darker than background */
        border-top: 3px solid color-mix(in srgb, var(--pico-primary), transparent 90%) !important;

        /* 4. The Magic: Multiple Backgrounds to fake the rest */
        background:
            /* Layer 1: Left "Border" (1px wide line, fading down) */
            linear-gradient(
                to bottom,
                color-mix(in srgb, var(--pico-primary), transparent 90%),
                transparent
            ) left top / 2px 100% no-repeat, /* 2px width ensures visibility on high-res screens */

            /* Layer 2: Right "Border" (1px wide line, fading down) */
            linear-gradient(
                to bottom,
                color-mix(in srgb, var(--pico-primary), transparent 90%),
                transparent
            ) right top / 2px 100% no-repeat,

            /* Layer 3: Main Background Fill (Fades from 97% transparent) */
            linear-gradient(
                to bottom,
                color-mix(in srgb, var(--pico-primary), transparent 97%),
                transparent
            ) center / 100% 100% no-repeat

            !important;
    }

    /* --- NAVIGATION TABS --- */

    .tab-menu a:not(.active-tab) {
        color: var(--pico-muted-color);
        border-bottom: 3px solid transparent;
    }

    /* --- MOBILE MENU HIGHLIGHTS --- */
    .custom-dropdown-item.active-tab {
        color: var(--pico-primary);
        font-weight: bold;
        border-left: 4px solid var(--pico-primary);
        padding-left: calc(1rem - 4px); /* Keep text aligned */
        background-color: var(--pico-card-background-color);
    }

    /* --- CONTENT AREA --- */
    .app-main-container .content {
        flex: 1;
        display: flex;
        flex-direction: column;
        overflow: hidden;
        min-height: 0;
        padding: 0 1rem;
        margin-top: 0;
    }

    /* FIX: FORCE SCREEN ROOT (e.g., CARD) TO BE FLEX COLUMN
       This allows us to control the Card layout without passing 'style' props */
    .app-main-container .content > * {
        flex: 1;                /* Fill the .content area */
        display: flex;          /* Become a flex container itself */
        flex-direction: column; /* Stack H3, Table, etc. */
        overflow: hidden;       /* Prevent outer scroll */
        min-height: 0;

        /* Optional: Reset Pico margins if Card adds them */
        margin-bottom: 1rem;
    }

    /* --- Mobile Styles --- */
    .mobile-view-wrapper { display: flex; justify-content: center; align-items: flex-start; padding-top: 2rem; min-height: 100vh; background-color: var(--muted-border-color); }
    .mobile-view-content { width: 100%; max-width: 400px; height: 800px; border-radius: 1.5rem; overflow: hidden; display: flex; flex-direction: column; border: 4px solid #374151; box-shadow: 0 10px 40px rgba(0,0,0,0.25); background-color: var(--card-background-color); }
    .mobile-view-content header { flex-shrink: 0; padding: 1rem; border-bottom: 1px solid var(--card-border-color); background-color: var(--card-background-color); }
    .mobile-view-content .content { flex-grow: 1; overflow-y: auto; padding: 1rem; }
"#;

    rsx! {
        document::Meta {
            name: "viewport",
            content: "width=device-width, initial-scale=1.0",
        }
        document::Stylesheet {
            href: asset!("/assets/css/pico.cyan.min.css"),
        }
        style {
            "{responsive_css}"
        }
        AppBody {}
    }
}

// In ui/src/lib.rs

#[component]
fn AppBody() -> Element {
    // this will be processed on server before initial page is delivered.
    let initial_data_future = use_server_future(move || async move {
        // call the server apis concurrently
        let (network_result, prefs_result) = tokio::join!(api::network(), api::get_user_prefs());

        let network = match network_result {
            Ok(n) => n,
            Err(e) => return Err(e),
        };
        let user_prefs = match prefs_result {
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        dioxus_logger::tracing::info!("prefs: {:#?}", user_prefs);

        Ok((network, user_prefs))
    })?;

    // Read from the single future to ensure it's polled during SSR.
    let body = match &*initial_data_future.read() {
        Some(Ok((network, prefs))) => {
            rsx! {
                LoadedApp {
                    app_state: AppState::new(*network),
                    user_prefs: *prefs,
                }
            }
        }
        Some(Err(e)) => rsx! {
            p {
                "An error occurred: {e}"
            }
        },
        _ => rsx! {
            p {
                "Loading..."
            }
        },
    };
    body
}

/// This component holds the main app logic and only runs when data is ready.
#[component]
fn LoadedApp(app_state: AppState, user_prefs: UserPrefs) -> Element {
    // Provide the stable, non-reactive AppState.
    use_context_provider(|| app_state.clone());

    // Create signals for mutable state at the top level of the component.
    let prices_signal = use_signal(|| None);
    let display_preference_signal = use_signal(|| user_prefs.display_preference().to_owned());

    // Provide the mutable state by passing the already created signals.
    use_context_provider(|| AppStateMut {
        prices: prices_signal,
        display_preference: display_preference_signal,
    });
    // Get a handle to the mutable state to populate it.
    let mut app_state_mut = use_context::<AppStateMut>();

    let fiat_enabled = app_state_mut.display_preference.read().is_fiat_enabled();
    let prices_resource = use_resource(move || async move {
        if fiat_enabled {
            // Fetch fiat prices from the backend ONLY if fiat mode is enabled.
            api::fiat_prices().await
        } else {
            Ok(PriceMap::default())
        }
    });

    use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let mut res = prices_resource;
        async move {
            loop {
                compat::sleep(std::time::Duration::from_secs(60)).await;
                // The conditional logic is now INSIDE the hook's closure.
                if display_preference_signal.read().is_fiat_enabled() {
                    res.restart();
                }
            }
        }
    });

    use_effect(move || {
        // The conditional logic is also moved inside here.
        if display_preference_signal.read().is_fiat_enabled() {
            if let Some(Ok(price_map)) = prices_resource.read().as_ref() {
                // This check prevents infinite loops if the resource returns the same data.
                if app_state_mut.prices.peek().as_ref() != Some(price_map) {
                    app_state_mut.prices.set(Some(price_map.clone()));
                }
            }
        } else {
            // Ensure prices are cleared if fiat mode is turned off.
            if app_state_mut.prices.peek().is_some() {
                app_state_mut.prices.set(None);
            }
        }
    });

    let active_screen = use_signal(Screen::default);
    let mut view_mode = use_signal(ViewMode::default);

    // --- Provide the active_screen signal to the context ---
    use_context_provider(|| active_screen);
    let wrapper_class = if view_mode() == ViewMode::Mobile {
        "mobile-view-wrapper"
    } else {
        ""
    };
    let content_class = if view_mode() == ViewMode::Mobile {
        "mobile-view-content"
    } else {
        ""
    };
    rsx! {
        if view_mode() == ViewMode::Desktop {
            div {
                class: "app-main-container",
                Container {
                    header {
                        nav {
                            ul {
                                // Conditionally render the button based on the environment variable.
                                if option_env!("VIEW_MODE_TOGGLE") == Some("1") {
                                    li {
                                        Button {
                                            button_type: ButtonType::Contrast,
                                            outline: true,
                                            on_click: move |_| view_mode.set(ViewMode::Mobile),
                                            "Mobile View"
                                        }
                                    }
                                }
                                li {
                                    Tabs {
                                        active_screen,
                                    }
                                }
                            }
                        }
                    }
                    div {
                        class: "content",
                        match active_screen() {
                            Screen::Balance => rsx! {
                                BalanceScreen {}
                            },
                            Screen::Send => rsx! {
                                SendScreen {}
                            },
                            Screen::Receive => rsx! {
                                ReceiveScreen {}
                            },
                            Screen::History => rsx! {
                                HistoryScreen {}
                            },
                            Screen::Addresses => rsx! {
                                AddressesScreen {}
                            },
                            Screen::Peers => rsx! {
                                PeersScreen {}
                            },
                            Screen::BlockChain => rsx! {
                                BlockChainScreen {}
                            },
                            Screen::Mempool => rsx! {
                                MempoolScreen {}
                            },
                            Screen::MempoolTx(tx_id) => rsx! {
                                MempoolTxScreen {
                                    tx_id,
                                }
                            },
                            Screen::Block(selector) => {
                                let key = std::fmt::format(format_args!("{:?}", selector));
                                rsx! {
                                    BlockScreen {
                                        key: "{key}",
                                        selector,
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            div {
                class: "{wrapper_class}",
                div {
                    class: "{content_class}",
                    header {
                        nav {
                            ul {
                                li {
                                    h1 {
                                        style: "margin: 0; font-size: 1.5rem;",
                                        "Neptune Wallet"
                                    }
                                }
                            }
                            ul {
                                li {
                                    HamburgerMenu {
                                        active_screen,
                                        view_mode,
                                    }
                                }
                            }
                        }
                    }
                    div {
                        class: "content",
                        match active_screen() {
                            Screen::Balance => rsx! {
                                BalanceScreen {}
                            },
                            Screen::Send => rsx! {
                                SendScreen {}
                            },
                            Screen::Receive => rsx! {
                                ReceiveScreen {}
                            },
                            Screen::History => rsx! {
                                HistoryScreen {}
                            },
                            Screen::Addresses => rsx! {
                                AddressesScreen {}
                            },
                            Screen::Peers => rsx! {
                                PeersScreen {}
                            },
                            Screen::BlockChain => rsx! {
                                BlockChainScreen {}
                            },
                            Screen::Mempool => rsx! {
                                MempoolScreen {}
                            },
                            Screen::MempoolTx(tx_id) => rsx! {
                                MempoolTxScreen {
                                    tx_id,
                                }
                            },
                            Screen::Block(selector) => {
                                let key = std::fmt::format(format_args!("{:?}", selector));
                                rsx! {
                                    BlockScreen {
                                        key: "{key}",
                                        selector,
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

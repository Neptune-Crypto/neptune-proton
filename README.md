# neptune-proton

This crate implements an experimental GUI wallet prototype for [neptune cash](https://neptune.cash).  This is a binary/application crate, built on the Dioxus framework.

## Naming

The "proto" in `neptune-proton` refers to the prototype nature of this wallet.

## Overview

`neptune-proton` is a first attempt to build a cross-platform desktop
wallet that interfaces with the RPC interface of
[neptune-core](https://github.com/Neptune-Crypto/neptune-core/).

It can be thought of as a GUI dashboard for neptune-core.

This means that neptune-proton is an interface for neptune-core's built-in
wallet.  neptune-proton does not generate any wallet keys of its own.

## Quick Start

1. Install neptune-core, if you haven't already.

   https://github.com/Neptune-Crypto/neptune-core

2. Run neptune-core.
   
3. Download neptune-proton binary for your platform from

   https://github.com/Neptune-Crypto/neptune-proton/releases/latest

4. Install neptune-proton using your operating system's package manager.

5. run neptune-proton.

   If everything is correct it should automatically connect to neptune-core and
   display your wallet.

   See the Environment Variables section below if you should need to modify the
   RPC port or other settings.


## Project Goals

The prototype has a few primary objectives:

1. Help identify and hopefully overcome rough edges and problem areas for
   developers of Neptune wallet software.

2. Build a wallet foundation that is truly cross platform with possibility to
   run on Desktop (Mac, Linux, Windows) and mobile (Android, Iphone).

3. Pioneer usage of neptune-cash data types in a browser (wasm) environment.

4. Provide a starting point for wallet developers to launch from for more
   advanced wallets.

5. Provide a functional, if simple, GUI wallet/dashboard for
   the neptune-cash community to interact with neptune-core.

## Non Goals (at this time)

* Not attempting to be a direct participant of neptune p2p network (independent of neptune-core)
* Not attempting to avoid requirement that user run an instance of neptune-core.
* Not attempting to provide multi-wallet functionality.
* Not attempting to manage keys independently of neptune-core's wallet.

## Status

As of 2025-11-28:

* windows, mac, and linux desktop binaries are available in `Releases` (v0.1.2)
* web (wasm) build works, must build from source.

As of 2025-11-18:

* fetching price data from goingecko and coinpaprika.
* displaying and entering amounts in many national currencies.
* generation and scanning of animated QR codes.
* export and import of animated QR code files.
* builds for web and desktop (linux)  windows, mac coming soon.
* mempool screen works
* history screen works
* a new peers screen works
* blockchain screen works -- has a minimalistic block explorer

As of 2025-07-09:

* Connectivity with neptune-core is working via RPC (only for localhost so far)
* A basic set of screens is implemented, best viewed in desktop mode.
* The Addresses screen lists used addresses.
* The Balance screen lists confirmed balance.  (needs to be fleshed out)
* The Receive screen functions, for both Generation Addresses and Symmetric keys.
* QR codes are invalid for Generation addresses, because they are too long.
* The BlockChain screen shows the current block height.  (needs to be fleshed out.)
* The Send screen functions and supports sending to multiple recipients.
* The Mempool screen is a non-functional place-holder.
* The History screen is a non-functional place-holder.
* There is not yet any settings screen, or any way to generate a new wallet.

## Building and Running

### Dependencies

1. Rust compiler -- [Instructions](https://www.rust-lang.org/tools/install).
2. Dioxus 0.7.1 -- [Instructions](https://dioxuslabs.com/learn/0.7/getting_started/)
3. neptune-core -- [Instructions](https://github.com/Neptune-Crypto/neptune-core/)

for desktop platform:
* libv4l-dev  ubuntu: apt install libv4l-dev
* libxdo-dev  ubuntu: apt install libxdo-dev

### Start neptune-core

start neptune-core, if not already running, listening on the default RPC port. In this example we will use the regtest network, which generates transactions and blocks quickly.

```
neptune-core --regtest
```

#### mine some regtest (fake) NPT coins to your neptune-core wallet

```
neptune-cli mine-blocks-to-wallet 1
```

### Build and Run neptune-proton web app

The web app is probably the simplest to build and run with the least that can go wrong.

```
cd neptune-proton/web
dx serve --port 9999
```

Open [http://localhost:9999](http://localhost:9999) in your browser.  You should now be able to use the wallet.


### Build and Run neptune-proton desktop app

```
cd neptune-proton/desktop
dx run
```

### Bundle desktop release packages

```
cd neptune-proton/desktop
dx bundle --release
```

The wallet app should appear in a native desktop window.

### Environment variables

Some env vars can be set to affect how neptune-proton iruns.

They apply to any platform, eg web, desktop, android, ios, etc.


```
- NEPTUNE_CORE_RPC_PORT: <port>

- NPT_ONLY: 1 or 0
    0 --> Fiat/NPT toggle mode (default)
    1 --> NPT-only mode.

- FIAT_CURRENCY:
    "USD", "EUR", "JPY", etc.

- DISPLAY_AS_FIAT:
    "true" to make fiat the default display.

- PRICE_PROVIDER:
    "coingecko" or "coinpaprika".

- VIEW_MODE_TOGGLE:
    enables display of the desktop/mobile toggle button. for dev purposes.  1 or 0
```


## Development

See [README-dioxus-workspace.md](README-dioxus-workspace.md) for an overview of how the workspace is laid out.

Familiarize yourself with:

1. [Dioxus docs](https://docs.rs/dioxus/latest/dioxus/)
2. [neptune-core docs](https://docs.rs/neptune-cash/latest/neptune_cash/), in particular [rpc_server](https://docs.rs/neptune-cash/latest/neptune_cash/rpc_server/trait.RPC.html#tymethod.network).  Note however that neptune-proton may at times use the most recent neptune-cash from github, rather than the published crate.

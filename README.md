# neptune-proton

This crate implements an experimental GUI wallet prototype for [neptune cash](https://neptune.cash).  This is a binary/application crate, built on the Dioxus framework.

## WARNING: EXPERIMENTAL!

This crate is in an early, very rough prototype state. Everything is subject to change, or it could be abandoned altogether.

## Naming

The "proto" in `neptune-proton` refers to the prototype nature of this wallet.

## Overview

`neptune-proton` is a first attempt to build a cross-platform desktop and mobile wallet that interfaces with the RPC interface of [neptune-core](https://github.com/Neptune-Crypto/neptune-core/).

## Project Goals

The prototype has a few primary objectives:

1. Help identify and hopefully overcome rough edges and problem areas for any developers of Neptune wallet software.

2. Build a wallet app that is truly cross platform and runs on Desktop (Mac, Linux, Windows) and mobile (Android, Iphone).

3. Pioneer usage of neptune-cash data types in a browser (wasm) environment.

4. Provide a starting point for wallet developers to launch from.

5. Eventually provide a functional, if simple, GUI wallet app for the neptune-cash community.

## Non Goals (at this time)

* Not attempting to be a direct participant of neptune p2p network (independent of neptune-core)
* Not attempting to avoid requirement that user run an instance of neptune-core.
* Not attempting to provide multi-wallet functionality.
* Not attempting to manage keys independently of neptune-core's wallet.

## Status

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
2. Dioxus 0.6 -- [Instructions](https://dioxuslabs.com/learn/0.6/getting_started/)
3. neptune-core -- [Instructions](https://github.com/Neptune-Crypto/neptune-core/)

for desktop platform:
* libv4l-dev  ubuntu: apt install libv4l-dev

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
dx run --platform desktop
```

The wallet app should appear in a native desktop window.


## Development

See [README-dioxus-workspace.md](README-dioxus-workspace.md) for an overview of how the workspace is laid out.

Familiarize yourself with:

1. [Dioxus docs](https://docs.rs/dioxus/latest/dioxus/)
2. [neptune-core docs](https://docs.rs/neptune-cash/latest/neptune_cash/), in particular [rpc_server](https://docs.rs/neptune-cash/latest/neptune_cash/rpc_server/trait.RPC.html#tymethod.network).  Note however that neptune-proton may at times use the most recent neptune-cash from github, rather than the published crate.

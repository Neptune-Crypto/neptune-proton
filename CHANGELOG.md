## v0.2.1

🔧 Fixes

* improve numeric entry in amount field ([4bbd146](https://github.com/Neptune-Crypto/neptune-proton/commit/4bbd14667975de6b3fb5a88d1d6358c9785c55de))

⚙️  CI/CD

* remove dup checksum files ([194570c](https://github.com/Neptune-Crypto/neptune-proton/commit/194570c6580b83102bc316066b0c4bfa66959529))

## v0.2.0

🚀 Features

* clear peer standings ([49b2cad](https://github.com/Neptune-Crypto/neptune-proton/commit/49b2cad37e82f87f891e811c41e47e1e537d3ec9))
* increase size of the SVG graphic ([c7620ea](https://github.com/Neptune-Crypto/neptune-proton/commit/c7620ea71f61de51dcf255a51e579d844dea7d69))
* add utxos screen ([84e572b](https://github.com/Neptune-Crypto/neptune-proton/commit/84e572b644ba6e3399a3563b9956f15679443e8e))
* screens handle empty list better ([fdb2cc1](https://github.com/Neptune-Crypto/neptune-proton/commit/fdb2cc14c75da680fa72fe81e3866c4ad355fe59))
* improve display of peer connection time ([b7ce016](https://github.com/Neptune-Crypto/neptune-proton/commit/b7ce016119f0f89383cac7056d3bab3d36191e3a))

🔧 Fixes

* make svg anims work on webkit(gtk) ([be89be5](https://github.com/Neptune-Crypto/neptune-proton/commit/be89be506c9110c2d0d316fd8e0e0d60b820d57e))
* right align amount field ([c8e337d](https://github.com/Neptune-Crypto/neptune-proton/commit/c8e337d32e8f59fc11a3a6d842e256191658ba95))

📦 Build System

* add changelog command/script ([efcec56](https://github.com/Neptune-Crypto/neptune-proton/commit/efcec56b17685d54fd3e90f521bc8d96737ddb04))
* bump minor version to 0.2.0 ([3630759](https://github.com/Neptune-Crypto/neptune-proton/commit/3630759ef6450cba16d32ca67479314c6c1e3610))
* add checksums to release page, and some cleanup ([039f283](https://github.com/Neptune-Crypto/neptune-proton/commit/039f28303283d81b8bc4aec2521bff5ddfa837c6))
* include changelog with each release ([7f6b18d](https://github.com/Neptune-Crypto/neptune-proton/commit/7f6b18d63b02a3c2c98b9a9720a931840c9650f9))

⚙️  CI/CD

* read changelog file for release page ([c51cc3f](https://github.com/Neptune-Crypto/neptune-proton/commit/c51cc3f4d90db721adec314ae4b63eb36663c33f))
* fix how repo is cloned, for changelog ([3bfba45](https://github.com/Neptune-Crypto/neptune-proton/commit/3bfba45fb3ad4c4940cd0dbbebfb7bc6744b4f59))

🛠️ Chore

* fix rustc warnings ([d3dd4df](https://github.com/Neptune-Crypto/neptune-proton/commit/d3dd4dfb8fa7a2a2b758407632ac9d65417e390a))
* cargo +nightly fmt ([9509058](https://github.com/Neptune-Crypto/neptune-proton/commit/9509058338c731cfe26753ed6c9ccdda2156e249))
* improve mempool formatting ([673a72b](https://github.com/Neptune-Crypto/neptune-proton/commit/673a72b965248b0bd053d5b81c3f18ec61dd1ddb))
* improve mempool-empty.svg animation ([2918eae](https://github.com/Neptune-Crypto/neptune-proton/commit/2918eaef8880b4a88a531f279a7ef4dc6feb6259))
* cargo +nightly fmt ([71252c2](https://github.com/Neptune-Crypto/neptune-proton/commit/71252c2aab53491d9fdf9c447a94c57c7b5329dc))
* cargo +nightly fmt ([dbf750e](https://github.com/Neptune-Crypto/neptune-proton/commit/dbf750e2ea9be568cd4dfa9001a91658a1bf8679))

📝 Documentation

* add changelog file ([d0d5f13](https://github.com/Neptune-Crypto/neptune-proton/commit/d0d5f13c0c6db1bb72e15455982251bbc3b40241))
* Fix broken link ([917e6bb](https://github.com/Neptune-Crypto/neptune-proton/commit/917e6bb6f65793c9eb6b53410963952a363b9e99))
* fix build deps ([0958a57](https://github.com/Neptune-Crypto/neptune-proton/commit/0958a5755177d3e0a999728513b6c91f10496513))

## v0.1.3

🔧 Fixes

* add scrollbar to mempool_tx screen ([129caf7](https://github.com/Neptune-Crypto/neptune-proton/commit/129caf7b588f451307ff66943d16fe2b09655e1b))
* prevent infinite loop from client when neptune-core becomes unavailable ([5623d72](https://github.com/Neptune-Crypto/neptune-proton/commit/5623d72b2c5c74abcde7099aa46943817b92c2e6))
* regression in use_is_touch_device.rs (desktop) ([0f0db7d](https://github.com/Neptune-Crypto/neptune-proton/commit/0f0db7d3999003dbcae908c9c88c0d64bc098607))
* amount keypad entry in send wizard ([4c3b44e](https://github.com/Neptune-Crypto/neptune-proton/commit/4c3b44e1f5fbade99f44c295e708bef49774a717))
* abbrev transaction_id after send ([d032072](https://github.com/Neptune-Crypto/neptune-proton/commit/d032072b32b4159325913d9637336cb039a4c783))
* bind icon.ico to neptune-proton.exe ([73fa218](https://github.com/Neptune-Crypto/neptune-proton/commit/73fa21861435c99194bcfcc98396c374f2631998))
* icon display for desktop (linux, windows) ([47fe16f](https://github.com/Neptune-Crypto/neptune-proton/commit/47fe16fba7c86934ceed1ae5e1ec1ea8baa3398c))
* reverse fiat/npt button on wizard fee page ([2a84d38](https://github.com/Neptune-Crypto/neptune-proton/commit/2a84d38df9a6031623dbd75ebffdb9810e9e49b8))
* highlight active tab ([bbdec72](https://github.com/Neptune-Crypto/neptune-proton/commit/bbdec72ee898f6ee70beedcdb2d1fa6c1aa047f2))

📦 Build System

* bump version to 0.1.3 ([ef4d1b3](https://github.com/Neptune-Crypto/neptune-proton/commit/ef4d1b3c670d9082165d274f092f5d56ffb183ba))
* fix qs_scanner.rs conditional compilation for web build ([7c1398b](https://github.com/Neptune-Crypto/neptune-proton/commit/7c1398b60ca0a3584b5ade7b5821c60e95841f0c))
* update Cargo.lock ([65f80cc](https://github.com/Neptune-Crypto/neptune-proton/commit/65f80cc5c73bad5e506bb833b02475f9ccc81bee))

🛠️ Chore

* cargo +nightly fmt ([4baf2d6](https://github.com/Neptune-Crypto/neptune-proton/commit/4baf2d605c3df8eb5a3990766690ef642a36a088))
* fix warnings ([5635c77](https://github.com/Neptune-Crypto/neptune-proton/commit/5635c771f128caf51e7553db648ff68ae80ad733))
* remove un-needed caching code for network(), rpc_client(), get_token() ([d747c4f](https://github.com/Neptune-Crypto/neptune-proton/commit/d747c4f8d02cd283aa50ea2dae3a4316a1f4c3bf))
* cargo +nightly fmt ([fe4d0b6](https://github.com/Neptune-Crypto/neptune-proton/commit/fe4d0b6c4d1df6199b79ea191f4f984c499f3271))

📝 Documentation

* update README ([71e555a](https://github.com/Neptune-Crypto/neptune-proton/commit/71e555a61abff22ec93f4ae55bb6a3b4ecf5cef5))

⚠️  Work in Progress

* update all screens to handle rpc connection loss.  also adds periodic polling to some screens ([7627485](https://github.com/Neptune-Crypto/neptune-proton/commit/76274853035ed30d26dbf4a5eab1a7c2cd62f6cb))
* first 'working' cut at handling failed neptune-core rpc connection and resuming ([fef5541](https://github.com/Neptune-Crypto/neptune-proton/commit/fef55410b467d7281a656b9881b287ab66096109))

## v0.1.2

🚀 Features

* refactor qr_scanner. remove camera-capture dep. make it compatible for MacOs, ios, android ([fb355fc](https://github.com/Neptune-Crypto/neptune-proton/commit/fb355fc55ff423bb617236eb9a3c7a33e34b01bc))

🔧 Fixes

* make windows and mac also use nokhwa for camera capture ([7a33a96](https://github.com/Neptune-Crypto/neptune-proton/commit/7a33a968e12e68331f7c123e21051844679e9fb1))
* use native camera access on linux due to gtkwebview perms issues.  see dioxus issue 5037 ([f6c914e](https://github.com/Neptune-Crypto/neptune-proton/commit/f6c914e085fdf57273c8402f2e5aa67cc1d82c36))
* issue #1. link click regression. add ActionLink component ([cef13e4](https://github.com/Neptune-Crypto/neptune-proton/commit/cef13e41a3a36e8123d0bc4300d1dc72b1a96290))

📦 Build System

* upload installers individually in CI ([a226aa9](https://github.com/Neptune-Crypto/neptune-proton/commit/a226aa9c7ce9ce3022d299cf2058afbe8968e8c7))
* attempt to workaround CI upload-build issue ([9677f8d](https://github.com/Neptune-Crypto/neptune-proton/commit/9677f8d5a954f2d9cd5d97d08bec4da461421759))
* fix the 'Upload Build Artifacts' step ([b7933b2](https://github.com/Neptune-Crypto/neptune-proton/commit/b7933b2b680456bdd6c96d285c9b292814189f79))
* upload build artifacts after every build, retain for 1 day ([27dec4f](https://github.com/Neptune-Crypto/neptune-proton/commit/27dec4f893d583c857568b2a2a828dee22e4a6da))
* add icons for MacOS generated by 'dioxus tauri icon' ([6f4401e](https://github.com/Neptune-Crypto/neptune-proton/commit/6f4401e4d0fd4d8999fd2f6f0df71a196f9ed058))
* perform build on new commits, as well as tags ([8b9c7e5](https://github.com/Neptune-Crypto/neptune-proton/commit/8b9c7e5713e69535bf1650cce1882a86ba32fec4))

🛠️ Chore

* add project-version.sh for project versioning ([a4c5ba7](https://github.com/Neptune-Crypto/neptune-proton/commit/a4c5ba771ef699d4e18f4095cc081b30fc18d906))
* make NPT_ONLY=0 the default ([07d3427](https://github.com/Neptune-Crypto/neptune-proton/commit/07d3427a98a90be25fdedc2763cdbf0dbd40eda2))

Other Changes

* bump to v0.1.2 ([ad7eceb](https://github.com/Neptune-Crypto/neptune-proton/commit/ad7eceb3174309d5d190c4048f952bffa9aa4d1c))

## v0.1.1

📦 Build System

* create target dir if not existing ([45d487e](https://github.com/Neptune-Crypto/neptune-proton/commit/45d487ebbebcfbbb9601e04fd2ababf9e39b7d7f))
* fix build/bundle on windows ([f6c58c0](https://github.com/Neptune-Crypto/neptune-proton/commit/f6c58c0137499780e049803036910912c06c5a84))
* bump to 0.1.1 ([de7c408](https://github.com/Neptune-Crypto/neptune-proton/commit/de7c40802da970cb8a4226f762b04620dbd5a945))
* install dioxus-cli from source, all platforms.  includes fix for windows link error: https://github.com/DioxusLabs/dioxus/issues/4998 ([9d9a8a7](https://github.com/Neptune-Crypto/neptune-proton/commit/9d9a8a733811eff1e702ee8cb166c4e4b861a887))

## v0.1.0

🚀 Features

* Add GitHub Actions workflow for desktop releases ([dda15ac](https://github.com/Neptune-Crypto/neptune-proton/commit/dda15ac40c184f9bd04ffd77160df84590e845c8))
* improve fiat support ([bd9b4a9](https://github.com/Neptune-Crypto/neptune-proton/commit/bd9b4a9b60bab1b587bc1d2c77d01e55273d497b))
* sort columns in mempool.rs ([dcb19b2](https://github.com/Neptune-Crypto/neptune-proton/commit/dcb19b28592abe77e14eefffaaeee6a9c059fc04))
* make columns sortable in history.rs ([4777b44](https://github.com/Neptune-Crypto/neptune-proton/commit/4777b44d810cd68df23ee205826c50385b33a6b9))
* add peers screen ([fce7c75](https://github.com/Neptune-Crypto/neptune-proton/commit/fce7c757f3d97143e220fb7f810ed6dd0feab713))
* add block lookup form to blockchain screen ([ba06966](https://github.com/Neptune-Crypto/neptune-proton/commit/ba06966f270f74125b0d2fa7ff1e39f91f5653be))
* upload QR file (static or animated).  tested on desktop ([26fbb9e](https://github.com/Neptune-Crypto/neptune-proton/commit/26fbb9e4db3e1a629ac8d97bc4645cff06955653))
* generates animated QR as svg file and provides option to download it. ([2b7c84f](https://github.com/Neptune-Crypto/neptune-proton/commit/2b7c84f64019ef62b66ee6a20a79de4272d9bf92))
* impl qr scanning for desktop ([a174c2c](https://github.com/Neptune-Crypto/neptune-proton/commit/a174c2c3ebb5d8d014997bbd599fb54634089a2e))
* first cut at qr scanner. web platform only for now ([fae4f7a](https://github.com/Neptune-Crypto/neptune-proton/commit/fae4f7a92b2ddcb83bc431a483345e6217602c15))
* qr-codes for Generation addresses ([f5c9588](https://github.com/Neptune-Crypto/neptune-proton/commit/f5c9588bf5b32acc80a8a4d0f2220dddcac8392b))
* add NEPTUNE_CORE_RPC_PORT env var ([e8b4991](https://github.com/Neptune-Crypto/neptune-proton/commit/e8b499196351bd14062443ef19cd85640eb98bda))
* display dashboard data in balance screen ([72e6071](https://github.com/Neptune-Crypto/neptune-proton/commit/72e6071d4600239898f769c99735b70f73ec3055))
* add block screen and link from history and blockchain screens ([eaf27a9](https://github.com/Neptune-Crypto/neptune-proton/commit/eaf27a9858167bb14ce179f75a487c4a468e8f8f))
* add mempool-tx screen ([bfec0dc](https://github.com/Neptune-Crypto/neptune-proton/commit/bfec0dc4bcafb2e03edae2b935e5a07c54316708))

🔧 Fixes

* footer button spacing in send screen ([12cf130](https://github.com/Neptune-Crypto/neptune-proton/commit/12cf130cac64a77c3d9f51331735c34b6b4c6e2a))
* improve amount displays to avoid resizing during toggle ([174c26d](https://github.com/Neptune-Crypto/neptune-proton/commit/174c26d57a9025c8d6c867c41426598ccc701abd))
* use compat::sleep() instead of tokio sleep ([48bcfd3](https://github.com/Neptune-Crypto/neptune-proton/commit/48bcfd3c2f6df8753a82ac933d14336f996bdb88))
* get web (wasm) build working again with dioxus 0.7.1 ([eb05248](https://github.com/Neptune-Crypto/neptune-proton/commit/eb052480b06f3148f9c81e5c607c27dd4c58c32d))
* add libxdo-dev dep and build on ubuntu 22.04 for better C-lib compat ([b7be00b](https://github.com/Neptune-Crypto/neptune-proton/commit/b7be00b77fd63f73413c450fa2495de850b6b73f))
* now builds for both desktop and web ([5cc65de](https://github.com/Neptune-Crypto/neptune-proton/commit/5cc65de674584db01129d31ee85b5ea9c1817b1e))
* add --platform desktop to dx bundle cmd ([f871c5a](https://github.com/Neptune-Crypto/neptune-proton/commit/f871c5a3be5e67a5ac73b70f969972f425caff27))
* cargo binstall dioxus-cli ([d4eee83](https://github.com/Neptune-Crypto/neptune-proton/commit/d4eee8349e18e57886a17c42451bcf2420cff2ed))
* download dioxus-cli binary, disable fail-fast ([0f72787](https://github.com/Neptune-Crypto/neptune-proton/commit/0f727875ec0494e22cee703f232402138e46a929))
* install dioxus-cli with --locked ([78637a7](https://github.com/Neptune-Crypto/neptune-proton/commit/78637a78950b9d767b6b34082283c1de5a5a4b3b))
* use dioxus-cli 0.6.3 for github actions ([fcddb65](https://github.com/Neptune-Crypto/neptune-proton/commit/fcddb65a864446e8e76df353f6ceecb19639a07b))
* revert qr_scanner.rs to ecc7e4832 ([07cd40d](https://github.com/Neptune-Crypto/neptune-proton/commit/07cd40d80daff67253a3891889dbebcf871ac89b))
* mempool balance-delta calculation ([405fd77](https://github.com/Neptune-Crypto/neptune-proton/commit/405fd770de9af5ad0e1e40976ced914f73fe4d1c))
* first cut at adding fiat currency support ([f311fee](https://github.com/Neptune-Crypto/neptune-proton/commit/f311fee01644f4ad469aee0eb9dc11189efe8001))
* display block digest as hex ([67bdc80](https://github.com/Neptune-Crypto/neptune-proton/commit/67bdc80b6d51a8448a94e59b015558c6ca4fd4f1))
* fit copy and qr buttons into address row so it doesn't expand ([022db72](https://github.com/Neptune-Crypto/neptune-proton/commit/022db72e40cfe7d8d72855902eace29ccd720d2b))
* build warnings ([104fd75](https://github.com/Neptune-Crypto/neptune-proton/commit/104fd75189cc2da1ac748dc6d47413c2ac4f0094))
* make desktop build again ([11bfe2b](https://github.com/Neptune-Crypto/neptune-proton/commit/11bfe2b58794bac4fe34ebf3c51b00fdf46723b0))
* get app building for web platform again.  qr download, scan, upload work ([b2753a6](https://github.com/Neptune-Crypto/neptune-proton/commit/b2753a6f7c36c311938be5dfe43be9cb9d030050))
* prevent loading tab urls in external browser on desktop ([9330012](https://github.com/Neptune-Crypto/neptune-proton/commit/9330012e236d0eb14aa6d29711fd4409dbf48032))
* get rid of dioxus log warnings in qr_scanner.rs ([f9c6b57](https://github.com/Neptune-Crypto/neptune-proton/commit/f9c6b5783cfa336b0801d0e945ddc0cbc91433f8))
* turn off camera when dialog is canceled or closed by clicking outside ([cff68a3](https://github.com/Neptune-Crypto/neptune-proton/commit/cff68a37eb80859705b3a75f492e0765058aab14))
* clipboard was not working on wasm32 ([3e4406c](https://github.com/Neptune-Crypto/neptune-proton/commit/3e4406cf3ef12bfe8e1e9c75fb6ffa83913a198f))
* add compat.rs for cross platform sleep, clipboard, etc ([b7aca4f](https://github.com/Neptune-Crypto/neptune-proton/commit/b7aca4f09c53ccc77ee46486d7eef2b10e08376d))
* highlight active tab; avoid jumping around ([9cc81b0](https://github.com/Neptune-Crypto/neptune-proton/commit/9cc81b0b26caa3dedfebcaceed5344167ce5ad06))
* display history in desc order ([a769186](https://github.com/Neptune-Crypto/neptune-proton/commit/a7691867780e1aa0d3ecf6d889311809c3f9f3dd))
* improve button navigation after send rpc called ([dbd47af](https://github.com/Neptune-Crypto/neptune-proton/commit/dbd47afd4c0ea48bd867e730a2dc240bbaba002b))

📦 Build System

* set CARGO_INCREMENTAL: 0.  to baby windows ([08d6a21](https://github.com/Neptune-Crypto/neptune-proton/commit/08d6a214e7b2411ec79c01598d1d6d29b4a46bdd))
* install dioxus-cli 0.7.1 from source on linux because binary package uses newer GLIBC ([656e50d](https://github.com/Neptune-Crypto/neptune-proton/commit/656e50d6e1695f662528bbb2fe68325d31dc4a3f))
* use dioxus 0.7.1 ([1d40c35](https://github.com/Neptune-Crypto/neptune-proton/commit/1d40c35f661d8e5eb5f3607607d84dbe9c629cfd))
* update 'dx bundle' cmd for github actions ([e2dfc35](https://github.com/Neptune-Crypto/neptune-proton/commit/e2dfc3506811e9c33ff0d03b72390af3ddc7e3fb))
* neptune-gui --> neptune-proton ([8afeebe](https://github.com/Neptune-Crypto/neptune-proton/commit/8afeebe3a3600ee4e99792a218de2ed8c760ea05))
* fix desktop build and make it so 'dx bundle' builds fullstack binaries without --features 'server,desktop' ([e6b911d](https://github.com/Neptune-Crypto/neptune-proton/commit/e6b911d53efc5591176db6155298884042b2fc3f))
* build against neptune-cash 0.5.0 ([ab7b528](https://github.com/Neptune-Crypto/neptune-proton/commit/ab7b528456108618e8e5d78b57eb5d06ea986b2c))
* desktop build working with dioxus-clipboard 0.3.0 ([0ae1597](https://github.com/Neptune-Crypto/neptune-proton/commit/0ae15970738c8205d11614132eefd2514fdd3616))
* make client and server deps optional, as per dioxus examples ([70fce7b](https://github.com/Neptune-Crypto/neptune-proton/commit/70fce7b5e4c279a8c1b5267cda1beea43db81de6))
* simplify file matching ([a22240f](https://github.com/Neptune-Crypto/neptune-proton/commit/a22240f7f3dd3c20ae089e13de75997a1cb088ce))
* use correct paths for binaries to upload ([06d2db6](https://github.com/Neptune-Crypto/neptune-proton/commit/06d2db6832854678d10a9a29618578f8171a3ef0))
* move assets/icon to icons, to satisfy dx bundle on windows ([5e66b82](https://github.com/Neptune-Crypto/neptune-proton/commit/5e66b827498d7a8c20fa99e2c41c81a1d84a863c))
* set macos vars ([c912c90](https://github.com/Neptune-Crypto/neptune-proton/commit/c912c907054ce0ae622cab91eca9f46b61beccef))
* add icon.ico for windows ([8bd5bef](https://github.com/Neptune-Crypto/neptune-proton/commit/8bd5bef6700b37f24739fbd727817a78ce60f59e))
* add debug session for ssh login ([ae4d5c4](https://github.com/Neptune-Crypto/neptune-proton/commit/ae4d5c40f2e1a3ee4039de67b8bd7c17f338271d))
* try env vars with dx bundle ([b87043d](https://github.com/Neptune-Crypto/neptune-proton/commit/b87043d9a660794515784da78225a48bc73b07a7))
* add --json-output ([cbc5387](https://github.com/Neptune-Crypto/neptune-proton/commit/cbc53873b3cb98dff4cab4ceff6902b82b5fc90a))
* just try bundle --verbose --trace ([364be4a](https://github.com/Neptune-Crypto/neptune-proton/commit/364be4ac26ab74d4d5b47208207ebc404a0a67e7))
* try bundle with -- --message-format=json ([d2b6849](https://github.com/Neptune-Crypto/neptune-proton/commit/d2b68490a1817a8d7871043d3546e2c6bc248d0e))
* set RUSTFLAGS and CMAKE_FLAGS ([2109636](https://github.com/Neptune-Crypto/neptune-proton/commit/21096368722e22a96dbd392fbe39e278daac982c))
* hacks and workarounds for mac and windows ([fb33c87](https://github.com/Neptune-Crypto/neptune-proton/commit/fb33c870725126ca7f894e1dcb826af77f415e02))
* add more debugging info for mac and windows ([3441667](https://github.com/Neptune-Crypto/neptune-proton/commit/3441667eb7195c0a524a74d9aee153f7d70a42ad))
* add a debug step for logging purposes, and add --trace ([d33d239](https://github.com/Neptune-Crypto/neptune-proton/commit/d33d2399661a150ad2d4623e6b8a22e9b80504d5))
* add --verbose flag to dx bundle ([697e01f](https://github.com/Neptune-Crypto/neptune-proton/commit/697e01f524ae26f70ffebd1891620568f350250b))

🛠️ Chore

* add scrollbar to block screen ([efe0b7e](https://github.com/Neptune-Crypto/neptune-proton/commit/efe0b7e031b0671a8f3d6be84ee11b633dc5def5))
* improve layout. remove browser scrollbar ([6cfd5a8](https://github.com/Neptune-Crypto/neptune-proton/commit/6cfd5a82e64abdc23623a326999b783dd7d3cd09))
* remove menubar from desktop window ([4ea8ce5](https://github.com/Neptune-Crypto/neptune-proton/commit/4ea8ce5681d4ff95061772a281037cb6376ef747))
* upgrade to dioxus 0.7.1. desktop can now be built as a single binary with --features 'desktop,server' ([09705cb](https://github.com/Neptune-Crypto/neptune-proton/commit/09705cb6327cf879fd4a69f1caae4d656680d7c6))
* rename binary 'desktop' to 'neptune-gui' ([c0207d8](https://github.com/Neptune-Crypto/neptune-proton/commit/c0207d8621644f535d6295939bf1afa6e047024e))
* satisfy clippy for desktop ([5ebbe43](https://github.com/Neptune-Crypto/neptune-proton/commit/5ebbe436f58f4e723dc35eb8118e151a9e3459a2))
* dx fmt -s on qr_scanner.rs ([2b6f14c](https://github.com/Neptune-Crypto/neptune-proton/commit/2b6f14cb20391400b5345a1b6ca7a96c4d27ba67))
* add logo for linux desktop bundle ([bd5e798](https://github.com/Neptune-Crypto/neptune-proton/commit/bd5e7981278b261884d8dc84ceb1ac7e2a3e9c77))
* use rusttls instead of openssl ([555ab6d](https://github.com/Neptune-Crypto/neptune-proton/commit/555ab6d15e370966ef3ea62ed9eddcb1c1853247))
* remove unused dep ([8753615](https://github.com/Neptune-Crypto/neptune-proton/commit/8753615c22c9d14eada590f70249695029eae83e))
* use neptune-types dep from github ([f4b2316](https://github.com/Neptune-Crypto/neptune-proton/commit/f4b2316d1e6c38febce60b68db18c39977cde17b))
* improve display of balances ([d82962e](https://github.com/Neptune-Crypto/neptune-proton/commit/d82962e10829025ee0cd7785315c3839b62a7e4d))
* add dashboard_overview_data() rpc API ([f627742](https://github.com/Neptune-Crypto/neptune-proton/commit/f627742fe41b399203c3b240d789f052caf0f683))

📝 Documentation

* update README.md ([65d472b](https://github.com/Neptune-Crypto/neptune-proton/commit/65d472b3092f5e72373d4c3a7b9a05cb9761b230))

Other Changes

* doc: update readme ([a64435d](https://github.com/Neptune-Crypto/neptune-proton/commit/a64435d32814733f8c797419a6f8f94ca1505ed0))
* ui polishing. improve scrolling. use fixed headers, buttons.  re-work send wizard ([7e84530](https://github.com/Neptune-Crypto/neptune-proton/commit/7e84530240d25021cf0bd867095c4d58f096dd99))
* hide view mode toggle unless env var VIEW_MODE_TOGGLE is set to 1 ([739e76f](https://github.com/Neptune-Crypto/neptune-proton/commit/739e76f74a029eed669c3444d960e0908b509f82))
* change title from 'Neptune Wallet' to 'NeptuneCore Wallet' ([ae2063e](https://github.com/Neptune-Crypto/neptune-proton/commit/ae2063effce3d40d75ab90a50e29100caa9961a2))
* convert debug output in mempool_tx screen into formatted display ([45a8973](https://github.com/Neptune-Crypto/neptune-proton/commit/45a8973b27b422faaf985376c95aef10aec28e3c))
* doc NEPTUNE_CORE_RPC_PORT in README.md ([3f0212c](https://github.com/Neptune-Crypto/neptune-proton/commit/3f0212cf204329de3e01cc0d7633a2637e7d29de))
* remove 'Prev Digest' field in block.rs ([3012a0a](https://github.com/Neptune-Crypto/neptune-proton/commit/3012a0a5eeda8aeea13569f2779028e763e31e68))
* add prev and next block buttons to block.rs and make them load smoothly ([e6794c0](https://github.com/Neptune-Crypto/neptune-proton/commit/e6794c0d13a7bece48c386a002a83f801b50226f))
* fix dup buttons in address screen after address modal is closed ([fa7c148](https://github.com/Neptune-Crypto/neptune-proton/commit/fa7c14811ef7cf368bb147ee37269fec4a7e92c9))
* fix scrollbars in address popup ([c595a2f](https://github.com/Neptune-Crypto/neptune-proton/commit/c595a2f79b4e3a0eeae5846f9e69c50289fa1b30))
* build against neptune-cash v0.4.0 ([7882678](https://github.com/Neptune-Crypto/neptune-proton/commit/7882678c0b2606c73c297c74442c73ea1531fb2e))
* add missing fix to qr uploading ([39a6097](https://github.com/Neptune-Crypto/neptune-proton/commit/39a60975fafbf9dffe6390bfd99caa494130a9f8))
* 1. generate static qr as svg and save to file. 2. fix animated svg so last frame is same size as the rest ([ecc7e48](https://github.com/Neptune-Crypto/neptune-proton/commit/ecc7e48325b37442b58294cf75288a3c82208533))
* desktop qr_scanner compiles, untested ([cd3cb31](https://github.com/Neptune-Crypto/neptune-proton/commit/cd3cb31c6f114b9a2877f81ab3001d84bdfef4db))
* finally animated qr scanning (web platform) is working without warnings or panics ([bf453a5](https://github.com/Neptune-Crypto/neptune-proton/commit/bf453a5dd3ee7f3535934322be2bb315500be3f9))
* fixes log warning.  a silent panic remains ([dbc92b6](https://github.com/Neptune-Crypto/neptune-proton/commit/dbc92b6c813fe9c042d5a57e331333accdecc791))
* scanner can read animated QR, with progress meter. has log warning though ([f46a347](https://github.com/Neptune-Crypto/neptune-proton/commit/f46a347bdae730fc0e25a9f74a73b95114ca7455))
* changes for compat with neptune-core v0.3.0 ([e9d2c66](https://github.com/Neptune-Crypto/neptune-proton/commit/e9d2c66c5b179505e7de192d2720e58346d726f9))
* mempool screen now displays real mempool data from neptune-core via mempool_overview rpc endpoint ([5e51bc1](https://github.com/Neptune-Crypto/neptune-proton/commit/5e51bc17e686bf013227dde8b4302f29cfdc316e))
* make a clickable component for block digests with popup window for copying ([f8cd959](https://github.com/Neptune-Crypto/neptune-proton/commit/f8cd95954a1fb076c777465a29f284ea9375b16c))
* implement mempool_overview api for mempool screen ([94e1728](https://github.com/Neptune-Crypto/neptune-proton/commit/94e1728d6b703eef384201c466aae577a6be8abb))
* implement history screen for confirmed tx with utxos grouped by block ([464ebd1](https://github.com/Neptune-Crypto/neptune-proton/commit/464ebd131cda3eaf0c1025d84e9b8bf75f43537e))
* formatting ([4658fee](https://github.com/Neptune-Crypto/neptune-proton/commit/4658feeccdb1fe66928967d379a006358f55b2ea))
* add a README in preparation to publish to github ([18a1c2e](https://github.com/Neptune-Crypto/neptune-proton/commit/18a1c2ea883610da7d45dc36832f3fe113988ca6))
* get dioxus --platform desktop to build ([be7c339](https://github.com/Neptune-Crypto/neptune-proton/commit/be7c339571cff0cc1e9c0c41562fa7eae0ddb90a))
* send screen now formats the send() tx results ([6488734](https://github.com/Neptune-Crypto/neptune-proton/commit/6488734746b7c1322f5219d7c544df857991ecd1))
* latest changes for send screen ([6dd5616](https://github.com/Neptune-Crypto/neptune-proton/commit/6dd5616ea1d9bd50be7d28863f79f29ffa60a341))
* send screen is basically functional now.  and addresses screen now uses address component for viewing full addresses in popup ([9262436](https://github.com/Neptune-Crypto/neptune-proton/commit/9262436ee3f442a198ce711cfdaf842a395c6cb9))
* use Address component ([325ad9a](https://github.com/Neptune-Crypto/neptune-proton/commit/325ad9a769fc5ce3c94971f28aff74abdae5467e))
* wip. first cut at adding Address component ([da8ad4a](https://github.com/Neptune-Crypto/neptune-proton/commit/da8ad4adeaf67991576a5c3b901b7a8ddb667e3e))
* send screen: make non-editable recipient rows use static text ([dcd43c9](https://github.com/Neptune-Crypto/neptune-proton/commit/dcd43c95f83aa8338136ef1da4a9dc9e27523b44))
* fixes for edit functonality in wizard step 1 ([b91665c](https://github.com/Neptune-Crypto/neptune-proton/commit/b91665c5d7d769b4643a7c98977afc4f8d8c8854))
* send screen: improve wizard step 1 ([ebfb648](https://github.com/Neptune-Crypto/neptune-proton/commit/ebfb6486ae43beda4269f287cbaf66207c29c0b1))
* send screen: further improvements ([b447340](https://github.com/Neptune-Crypto/neptune-proton/commit/b447340f6dcaa9ead5591d782ae907ab7ae7af6a))
* send screen: improve add recipients wizard screen ([30bce51](https://github.com/Neptune-Crypto/neptune-proton/commit/30bce51487a278cb85518b1c2be6dfe93a62f3fc))
* send screen: improve first screen of the wizard ([def60d4](https://github.com/Neptune-Crypto/neptune-proton/commit/def60d49436c162b19a1aad276ec85019247bd1b))
* send screen: simplify first screen of the wizard ([4372648](https://github.com/Neptune-Crypto/neptune-proton/commit/43726485932c20bd41c305e65916461d457eaab1))
* send screen: start moving towards a multi-screen wizard ([5318d8d](https://github.com/Neptune-Crypto/neptune-proton/commit/5318d8d96dff32cb3d3fd059ddb4c4eaf7766deb))
* send screen: add 'send another transaction' button ([8ddfb51](https://github.com/Neptune-Crypto/neptune-proton/commit/8ddfb5126217375ef9d62015f1d0ad356e5aa54b))
* move fee field into card with Review Transaction button ([7262ece](https://github.com/Neptune-Crypto/neptune-proton/commit/7262ececadb2e68c31b2b3dd0a2bc0452d15b36e))
* send screen: display tx details in 'confirm transaction' popup ([a262cae](https://github.com/Neptune-Crypto/neptune-proton/commit/a262cae76727670b1299dd4dbf1be3a18c944a4a))
* send screen: added a warning if address is used by multiple recipients ([b4557df](https://github.com/Neptune-Crypto/neptune-proton/commit/b4557df33831f526cc47e29d65191f9463e5f1cd))
* work begins on send screen to make it functional ([4e79cb5](https://github.com/Neptune-Crypto/neptune-proton/commit/4e79cb544994c88aef123ce1836e06a70fe5e696))
* minor tweak to send screen so it displays address hrp according to network ([6386bd1](https://github.com/Neptune-Crypto/neptune-proton/commit/6386bd1e3d54dd4d4a21e470384761f2fa1cc271))
* creates a new AppState for shared state between screens.  obtains network dynamically from neptune-core at startup and stores in in AppState ([18e2b45](https://github.com/Neptune-Crypto/neptune-proton/commit/18e2b450f6ee62fc153e8b5579c31a3aa82d1327))
* remove server_bin.  I'm not sure how/why it got created ([152a918](https://github.com/Neptune-Crypto/neptune-proton/commit/152a9188e202f98ba58f99f6fba8d26ebb453669))
* display qr code for generation address (abbreviated form, not a full address).  this is just to demo QC code functionality ([c3557b9](https://github.com/Neptune-Crypto/neptune-proton/commit/c3557b9544e37f46b4a51d791074e218579d1385))
* refactor copy button logic into CopyButton component and make it reset after 5 secs ([09ec081](https://github.com/Neptune-Crypto/neptune-proton/commit/09ec081668412c8ed5433f481cc736badbcb0f27))
* receive screen now generates an actual receive address. generation addr only for now ([f6bdcf5](https://github.com/Neptune-Crypto/neptune-proton/commit/f6bdcf5d9ea942d3a297a016ad9b38498787e987))
* set Modal title, and add a NoTitleModal ([4af0f4d](https://github.com/Neptune-Crypto/neptune-proton/commit/4af0f4d3f70b6a13d74c3a87f0b73f98914e01a4))
* move address-row into its own component ([f5a0888](https://github.com/Neptune-Crypto/neptune-proton/commit/f5a08888d0ca59646208bb9508d9c89f025b298a))
* add 'Copy' and 'Qr' buttons to each row in addresses screen ([ee09ac0](https://github.com/Neptune-Crypto/neptune-proton/commit/ee09ac00e652659ce9fa1ba1ee5858848c8363c0))
* addresses screen now displays list of addresses ([a5c5266](https://github.com/Neptune-Crypto/neptune-proton/commit/a5c5266d2570aa26463881d36bf9a17691c21f6f))
* get rid of unwrap() in api crate ([bb81024](https://github.com/Neptune-Crypto/neptune-proton/commit/bb81024166db2bc4519eddccaf2dcbb07ef9503d))
* simplify how RPCClient is stored in OnceCell ([0984032](https://github.com/Neptune-Crypto/neptune-proton/commit/0984032678989564117510ea23144545f8cd57ee))
* store rpc client in OnceCell so connection is made only once ([d60b0f7](https://github.com/Neptune-Crypto/neptune-proton/commit/d60b0f7adeeefffd95477396b0a96c5823de10d9))
* wallet_balance API now returns NativeCurrencyAmount from neptune-types instead of String ([70b4b00](https://github.com/Neptune-Crypto/neptune-proton/commit/70b4b0067fdd816c5af72f0d90b70beb79f9cd6c))
* add scripts for generating and syncing rpc api from neptune-core ([49434e9](https://github.com/Neptune-Crypto/neptune-proton/commit/49434e975510269cf8d0766db89aae4bc0d35c8d))
* integrate neptune-types and rpc_api.  the block-height api now returns a neptune_types::block_height::BlockHeight deserialized from neptune-core instead of a simple u64. ([1dac2c7](https://github.com/Neptune-Crypto/neptune-proton/commit/1dac2c77ab99551d951176caa79b41dd549d7ded))
* minor ui tweaks ([66c50f0](https://github.com/Neptune-Crypto/neptune-proton/commit/66c50f0203f8810eda233f2e17741ee2138e75f3))
* dioxus frontend calls dioxus backend apis, and those apis call neptune rpcs ([51bcef8](https://github.com/Neptune-Crypto/neptune-proton/commit/51bcef81812a6b14a8817e7539ade222250b92b7))
* convert to workspace, and dioxus fullstack, so that the server can have a dep on neptune-cash, which cannot be built for wasm ([3457cc3](https://github.com/Neptune-Crypto/neptune-proton/commit/3457cc302b197428db7c1014058675f6bbe23a9d))
* first attempt to add neptune-cash dep.  does not work for wasm frontend ([48bdef8](https://github.com/Neptune-Crypto/neptune-proton/commit/48bdef8c84ac0eb92de3443930d19b3e32c18502))
* improve display, especially of mobile view ([7d80eb3](https://github.com/Neptune-Crypto/neptune-proton/commit/7d80eb39a6a8e97e57e9d5d3f854cdd0446c798d))
* add wallet screens ([f1d6d83](https://github.com/Neptune-Crypto/neptune-proton/commit/f1d6d83feffbd9c56b4312b3fbaa7382ed78fd21))
* initial import. starting neptune-gui wallet app ([2320a6a](https://github.com/Neptune-Crypto/neptune-proton/commit/2320a6ac14c15ae75abc8d5ba629db26760340e5))
* Initial commit ([f4597e9](https://github.com/Neptune-Crypto/neptune-proton/commit/f4597e92953315c0d392adfffa3101da100b7582))

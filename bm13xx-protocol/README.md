# `bm13xx-protocol`

[![Crate](https://img.shields.io/crates/v/bm13xx-protocol.svg)](https://crates.io/crates/bm13xx-protocol)
[![Docs](https://docs.rs/bm13xx-protocol/badge.svg)](https://docs.rs/bm13xx-protocol)
[![License](https://img.shields.io/crates/l/bm13xx-protocol.svg)](#license)
[![CI](https://github.com/GPTechinno/bm13xx-rs/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/GPTechinno/bm13xx-rs/actions/workflows/rust-ci.yml)
![MSRV](https://img.shields.io/badge/rustc-1.87+-blue.svg)

UART protocol of the Bitmain BM13xx Bitcoin mining ASICs, `no_std` and without allocation:

- command frames with their CRC: register reads and writes (to one chip or all), chip addressing,
  chain inactive, jobs with midstates or with a full block header (version rolling);
- response parsing: register values and nonces, with the job ID, the small core ID, the rolled
  version bits and the extra zero bits of the share's hash.

It is the lowest layer of the [`bm13xx-rs`](https://github.com/GPTechinno/bm13xx-rs) workspace:
the chip crates build their commands with it and [`bm13xx-chain`](https://crates.io/crates/bm13xx-chain)
parses the chips' responses with it.

Part of the [`bm13xx-rs`](https://github.com/GPTechinno/bm13xx-rs) workspace.

## License

Licensed under either of [Apache License, Version 2.0](https://github.com/GPTechinno/bm13xx-rs/blob/main/LICENSE-APACHE) or
[MIT license](https://github.com/GPTechinno/bm13xx-rs/blob/main/LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

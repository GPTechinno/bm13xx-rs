# `bm13xx-rs`

[![CI](https://github.com/GPTechinno/bm13xx-rs/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/GPTechinno/bm13xx-rs/actions/workflows/rust-ci.yml)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)
![MSRV](https://img.shields.io/badge/rustc-1.87+-blue.svg)

Rust `no_std` drivers for the Bitmain BM13xx Bitcoin mining ASIC family.

| Crate | What | Version | Docs |
|---|---|---|---|
| [`bm13xx-protocol`](https://github.com/GPTechinno/bm13xx-rs/tree/main/bm13xx-protocol) | UART frames: commands and responses | [![Crate](https://img.shields.io/crates/v/bm13xx-protocol.svg)](https://crates.io/crates/bm13xx-protocol) | [![Docs](https://docs.rs/bm13xx-protocol/badge.svg)](https://docs.rs/bm13xx-protocol) |
| [`bm13xx-asic`](https://github.com/GPTechinno/bm13xx-rs/tree/main/bm13xx-asic) | registers, PLL, core topology and the `Asic` trait | [![Crate](https://img.shields.io/crates/v/bm13xx-asic.svg)](https://crates.io/crates/bm13xx-asic) | [![Docs](https://docs.rs/bm13xx-asic/badge.svg)](https://docs.rs/bm13xx-asic) |
| [`bm13xx-chain`](https://github.com/GPTechinno/bm13xx-rs/tree/main/bm13xx-chain) | async driver for a chain of chips, frequency tuning | [![Crate](https://img.shields.io/crates/v/bm13xx-chain.svg)](https://crates.io/crates/bm13xx-chain) | [![Docs](https://docs.rs/bm13xx-chain/badge.svg)](https://docs.rs/bm13xx-chain) |
| [`bm1362`](https://github.com/GPTechinno/bm13xx-rs/tree/main/bm1362) | BM1362 (Antminer S19j Pro, S19) | [![Crate](https://img.shields.io/crates/v/bm1362.svg)](https://crates.io/crates/bm1362) | [![Docs](https://docs.rs/bm1362/badge.svg)](https://docs.rs/bm1362) |
| [`bm1366`](https://github.com/GPTechinno/bm13xx-rs/tree/main/bm1366) | BM1366 (Antminer S19 XP, S19k Pro, Bitaxe Ultra) | [![Crate](https://img.shields.io/crates/v/bm1366.svg)](https://crates.io/crates/bm1366) | [![Docs](https://docs.rs/bm1366/badge.svg)](https://docs.rs/bm1366) |
| [`bm1368`](https://github.com/GPTechinno/bm13xx-rs/tree/main/bm1368) | BM1368 (Antminer S21, Bitaxe Supra) | [![Crate](https://img.shields.io/crates/v/bm1368.svg)](https://crates.io/crates/bm1368) | [![Docs](https://docs.rs/bm1368/badge.svg)](https://docs.rs/bm1368) |
| [`bm1370`](https://github.com/GPTechinno/bm13xx-rs/tree/main/bm1370) | BM1370 (Antminer S21 Pro, S21 XP, Bitaxe Gamma) | [![Crate](https://img.shields.io/crates/v/bm1370.svg)](https://crates.io/crates/bm1370) | [![Docs](https://docs.rs/bm1370/badge.svg)](https://docs.rs/bm1370) |
| [`bm1372`](https://github.com/GPTechinno/bm13xx-rs/tree/main/bm1372) | BM1372 and BM1373 (NajaDuo) | [![Crate](https://img.shields.io/crates/v/bm1372.svg)](https://crates.io/crates/bm1372) | [![Docs](https://docs.rs/bm1372/badge.svg)](https://docs.rs/bm1372) |
| [`bm1397`](https://github.com/GPTechinno/bm13xx-rs/tree/main/bm1397) | BM1397 (Antminer S17, T17, Bitaxe Max) | [![Crate](https://img.shields.io/crates/v/bm1397.svg)](https://crates.io/crates/bm1397) | [![Docs](https://docs.rs/bm1397/badge.svg)](https://docs.rs/bm1397) |

`bm13xx-protocol` builds the frames, `bm13xx-asic` holds what the chips share, each chip crate
implements the `Asic` trait for its chip, and `bm13xx-chain` drives a chain of chips over
`embedded-hal-async` and `embedded-io-async`.

## Resources

- [BM1397 documentation](https://github.com/skot/BM1397/blob/master/bm1397.md)
- [BM1366 documentation](https://github.com/skot/BM1397/blob/master/bm1366.md)

## License

Dual licensed under your choice of either of:

 - Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 - MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

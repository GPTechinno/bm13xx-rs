# `bm13xx-asic`

[![Crate](https://img.shields.io/crates/v/bm13xx-asic.svg)](https://crates.io/crates/bm13xx-asic)
[![Docs](https://docs.rs/bm13xx-asic/badge.svg)](https://docs.rs/bm13xx-asic)
[![License](https://img.shields.io/crates/l/bm13xx-asic.svg)](#license)
[![CI](https://github.com/GPTechinno/bm13xx-rs/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/GPTechinno/bm13xx-rs/actions/workflows/rust-ci.yml)
![MSRV](https://img.shields.io/badge/rustc-1.87+-blue.svg)

Common ground of the Bitmain BM13xx Bitcoin mining ASIC drivers, `no_std`:

- register and core register types, with their fields;
- PLL settings, core topology, hash counting number, die temperature and supply voltage
  conversions;
- the `Asic` trait every chip crate implements.

The chips do no I/O: each operation is a sequence of commands, given one by one with the delay to
wait after each, that [`bm13xx-chain`](https://crates.io/crates/bm13xx-chain) sends.
Chip crates: [`bm1362`](https://crates.io/crates/bm1362), [`bm1366`](https://crates.io/crates/bm1366),
[`bm1368`](https://crates.io/crates/bm1368), [`bm1370`](https://crates.io/crates/bm1370),
[`bm1372`](https://crates.io/crates/bm1372), [`bm1397`](https://crates.io/crates/bm1397).

Part of the [`bm13xx-rs`](https://github.com/GPTechinno/bm13xx-rs) workspace.

## License

Licensed under either of [Apache License, Version 2.0](https://github.com/GPTechinno/bm13xx-rs/blob/main/LICENSE-APACHE) or
[MIT license](https://github.com/GPTechinno/bm13xx-rs/blob/main/LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

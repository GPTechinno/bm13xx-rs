# `bm1370`

[![Crate](https://img.shields.io/crates/v/bm1370.svg)](https://crates.io/crates/bm1370)
[![Docs](https://docs.rs/bm1370/badge.svg)](https://docs.rs/bm1370)
[![License](https://img.shields.io/crates/l/bm1370.svg)](#license)
[![CI](https://github.com/GPTechinno/bm13xx-rs/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/GPTechinno/bm13xx-rs/actions/workflows/rust-ci.yml)
![MSRV](https://img.shields.io/badge/rustc-1.87+-blue.svg)

Driver for the Bitmain BM1370 Bitcoin mining ASIC (Antminer S21 Pro and S21 XP hashboards, Bitaxe Gamma), `no_std`.

It implements the [`Asic`](https://docs.rs/bm13xx-asic/latest/bm13xx_asic/trait.Asic.html) trait of
[`bm13xx-asic`](https://crates.io/crates/bm13xx-asic): its default register values and the command
sequences of the chip (init, baud rate, core reset, hash frequency, nonce range split, version rolling, die temperature, supply voltages).
Drive a chain of BM1370 chips with [`bm13xx-chain`](https://crates.io/crates/bm13xx-chain):

```rust,ignore
use bm1370::BM1370;
use bm13xx_chain::Chain;

let mut chain = Chain::enumerate(BM1370::default(), uart, busy, reset, delay).await?;
chain.init(256).await?;
```

Part of the [`bm13xx-rs`](https://github.com/GPTechinno/bm13xx-rs) workspace.

## License

Licensed under either of [Apache License, Version 2.0](https://github.com/GPTechinno/bm13xx-rs/blob/main/LICENSE-APACHE) or
[MIT license](https://github.com/GPTechinno/bm13xx-rs/blob/main/LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

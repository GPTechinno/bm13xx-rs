# `bm1397`

[![Crate](https://img.shields.io/crates/v/bm1397.svg)](https://crates.io/crates/bm1397)
[![Docs](https://docs.rs/bm1397/badge.svg)](https://docs.rs/bm1397)
[![License](https://img.shields.io/crates/l/bm1397.svg)](#license)
[![CI](https://github.com/GPTechinno/bm13xx-rs/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/GPTechinno/bm13xx-rs/actions/workflows/rust-ci.yml)
![MSRV](https://img.shields.io/badge/rustc-1.87+-blue.svg)

Driver for the Bitmain BM1397 Bitcoin mining ASIC (Antminer S17 and T17 hashboards, Bitaxe Max), `no_std`.

It implements the [`Asic`](https://docs.rs/bm13xx-asic/latest/bm13xx_asic/trait.Asic.html) trait of
[`bm13xx-asic`](https://crates.io/crates/bm13xx-asic): its default register values and the command
sequences of the chip (init, baud rate, hash frequency, access to the I2C devices behind the chip).
Drive a chain of BM1397 chips with [`bm13xx-chain`](https://crates.io/crates/bm13xx-chain):

```rust,ignore
use bm1397::BM1397;
use bm13xx_chain::Chain;

let mut chain = Chain::enumerate(BM1397::default(), uart, busy, reset, delay).await?;
chain.init(256).await?;
```

[BM1397 documentation](https://github.com/skot/BM1397/blob/master/bm1397.md).

Part of the [`bm13xx-rs`](https://github.com/GPTechinno/bm13xx-rs) workspace.

## License

Licensed under either of [Apache License, Version 2.0](https://github.com/GPTechinno/bm13xx-rs/blob/main/LICENSE-APACHE) or
[MIT license](https://github.com/GPTechinno/bm13xx-rs/blob/main/LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

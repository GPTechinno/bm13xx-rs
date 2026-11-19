# `bm13xx-chain`

[![Crate](https://img.shields.io/crates/v/bm13xx-chain.svg)](https://crates.io/crates/bm13xx-chain)
[![Docs](https://docs.rs/bm13xx-chain/badge.svg)](https://docs.rs/bm13xx-chain)
[![License](https://img.shields.io/crates/l/bm13xx-chain.svg)](#license)
[![CI](https://github.com/GPTechinno/bm13xx-rs/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/GPTechinno/bm13xx-rs/actions/workflows/rust-ci.yml)
![MSRV](https://img.shields.io/badge/rustc-1.87+-blue.svg)

Async driver for a chain of Bitmain BM13xx Bitcoin mining ASICs, `no_std`, over
[`embedded-hal-async`](https://crates.io/crates/embedded-hal-async) (reset and busy pins, delays)
and [`embedded-io-async`](https://crates.io/crates/embedded-io-async) (serial port):

- enumeration and addressing, init, baud rate change, hash frequency ramp, nonce range split,
  version rolling;
- jobs and nonces, share difficulty, register, die temperature and supply voltage readings, I2C
  devices behind a chip;
- per-chip hash frequencies, nonce counts and hardware errors, frequency tuning (`tuning`) and
  known-nonce tests (`pattern`).

It is generic over the chip crate: [`bm1362`](https://crates.io/crates/bm1362),
[`bm1366`](https://crates.io/crates/bm1366), [`bm1368`](https://crates.io/crates/bm1368),
[`bm1370`](https://crates.io/crates/bm1370), [`bm1372`](https://crates.io/crates/bm1372),
[`bm1397`](https://crates.io/crates/bm1397).

```rust,ignore
use bm1370::BM1370;
use bm13xx_chain::Chain;
use fugit::HertzU64;

let mut chain = Chain::enumerate(BM1370::default(), uart, busy, reset, delay).await?;
chain.init(256).await?;
chain.change_baudrate(3_125_000).await?;
chain.set_hash_freq(HertzU64::MHz(500)).await?;
chain.split_nonce_between_chips().await?;
chain.enable_version_rolling(0x1fff_e000).await?;
```

## Features

- `defmt` or `log`: logging (one at most);
- `nonce-check`: check each nonce against the job it was found for and count the hardware errors
  of each chip (adds [`sha2`](https://crates.io/crates/sha2)).

Part of the [`bm13xx-rs`](https://github.com/GPTechinno/bm13xx-rs) workspace.

## License

Licensed under either of [Apache License, Version 2.0](https://github.com/GPTechinno/bm13xx-rs/blob/main/LICENSE-APACHE) or
[MIT license](https://github.com/GPTechinno/bm13xx-rs/blob/main/LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

xbdm
====

Xbox Debug Monitor client for Rust

[![crates.io](https://img.shields.io/crates/v/xbdm.svg)](https://crates.io/crates/xbdm)
[![Build Status](https://travis-ci.org/docbrown/xbdm-rs.svg?branch=master)](https://travis-ci.org/docbrown/xbdm-rs)

[Documentation](https://docs.rs/xbdm)

## Usage

First, add the following to your `Cargo.toml`:

```toml
[dependencies]
xbdm = "0.1.0-alpha"
```

Next, add this to your crate root:

```rust
extern crate xbdm;
```

Finally, lookup an Xbox by its debug name or IP address and connect to it:

```rust
let xbox = xbdm::resolve("MYXBOX").unwrap();
let mut client = xbdm::Client::connect(xbox).unwrap();
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

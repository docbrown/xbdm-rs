// Copyright 2017 xbdm-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate bufstream;

mod client;
mod error;
mod status;
mod xbox;

pub use client::{Client, Execute};
pub use error::{Error, ErrorKind, Result};
pub use status::StatusCode;
pub use xbox::{Discover, Xbox, discover, resolve, resolve_ip, resolve_name};

/// TCP/UDP port number used by the Xbox 360 for XBDM.
pub const PORT_360: u16 = 730;
/// TCP/UDP port number used by the Classic Xbox for XBDM.
pub const PORT_CLASSIC: u16 = 731;

/// Maximum length of an Xbox debug name.
pub const MAX_NAME_LENGTH: usize = 255;

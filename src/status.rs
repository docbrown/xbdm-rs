// Copyright 2017 xbdm-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::fmt;

/// A status code returned by an XBDM command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatusCode {
    /// 200- OK
    Ok,
    /// 201- connected
    Connected,
    /// 202- multiline response follows
    MultilineResponseFollows,
    /// 203- binary response follows
    BinaryResponseFollows,
    /// 204- send binary data
    SendBinaryData,
    /// 205- connection dedicated
    ConnectionDedicated,
    /// 400- unexpected error
    UnexpectedError,
    /// 401- max number of connections exceeded
    MaxConnectionsExceeded,
    /// 402- file not found
    FileNotFound,
    /// 403- no such module
    NoSuchModule,
    /// 404- memory not mapped
    MemoryNotMapped,
    /// 405- no such thread
    NoSuchThread,
    /// 406-
    ClockNotSet,
    /// 407- unknown command
    UnknownCommand,
    /// 408- not stopped
    NotStopped,
    /// 409- file must be copied
    MustCopy,
    /// 410- file already exists
    FileExists,
    /// 411- directory not empty
    NotEmpty,
    /// 412- filename is invalid
    InvalidFilename,
    /// 413- file cannot be created
    CannotCreate,
    /// 414- access denied
    AccessDenied,
    /// 415- no room on device
    DeviceFull,
    /// 416- not debuggable
    NotDebuggable,
    /// 417- type invalid
    InvalidCounterType,
    /// 418- data not available
    NoCounterData,
    /// 420- box not locked
    NotLocked,
    /// 421- key exchange required
    NeedKeyExchange,
    /// 422- dedicated connection required
    MustBeDedicated,
    Other(u16),
}

impl StatusCode {
    pub fn from_u16(n: u16) -> StatusCode {
        match n {
            200 => StatusCode::Ok,
            201 => StatusCode::Connected,
            202 => StatusCode::MultilineResponseFollows,
            203 => StatusCode::BinaryResponseFollows,
            204 => StatusCode::SendBinaryData,
            205 => StatusCode::ConnectionDedicated,
            400 => StatusCode::UnexpectedError,
            401 => StatusCode::MaxConnectionsExceeded,
            402 => StatusCode::FileNotFound,
            403 => StatusCode::NoSuchModule,
            404 => StatusCode::MemoryNotMapped,
            405 => StatusCode::NoSuchThread,
            406 => StatusCode::ClockNotSet,
            407 => StatusCode::UnknownCommand,
            408 => StatusCode::NotStopped,
            409 => StatusCode::MustCopy,
            410 => StatusCode::FileExists,
            411 => StatusCode::NotEmpty,
            412 => StatusCode::InvalidFilename,
            413 => StatusCode::CannotCreate,
            414 => StatusCode::AccessDenied,
            415 => StatusCode::DeviceFull,
            416 => StatusCode::NotDebuggable,
            417 => StatusCode::InvalidCounterType,
            418 => StatusCode::NoCounterData,
            420 => StatusCode::NotLocked,
            421 => StatusCode::NeedKeyExchange,
            422 => StatusCode::MustBeDedicated,
            _ => StatusCode::Other(n),
        }
    }

    pub fn to_u16(&self) -> u16 {
        match *self {
            StatusCode::Ok => 200,
            StatusCode::Connected => 201,
            StatusCode::MultilineResponseFollows => 202,
            StatusCode::BinaryResponseFollows => 203,
            StatusCode::SendBinaryData => 204,
            StatusCode::ConnectionDedicated => 205,
            StatusCode::UnexpectedError => 400,
            StatusCode::MaxConnectionsExceeded => 401,
            StatusCode::FileNotFound => 402,
            StatusCode::NoSuchModule => 403,
            StatusCode::MemoryNotMapped => 404,
            StatusCode::NoSuchThread => 405,
            StatusCode::ClockNotSet => 406,
            StatusCode::UnknownCommand => 407,
            StatusCode::NotStopped => 408,
            StatusCode::MustCopy => 409,
            StatusCode::FileExists => 410,
            StatusCode::NotEmpty => 411,
            StatusCode::InvalidFilename => 412,
            StatusCode::CannotCreate => 413,
            StatusCode::AccessDenied => 414,
            StatusCode::DeviceFull => 415,
            StatusCode::NotDebuggable => 416,
            StatusCode::InvalidCounterType => 417,
            StatusCode::NoCounterData => 418,
            StatusCode::NotLocked => 420,
            StatusCode::NeedKeyExchange => 421,
            StatusCode::MustBeDedicated => 422,
            StatusCode::Other(n) => n,
        }
    }

    pub fn default_message(&self) -> Option<&'static str> {
        match *self {
            StatusCode::Ok => Some("OK"),
            StatusCode::Connected => Some("connected"),
            StatusCode::MultilineResponseFollows => Some("multiline response follows"),
            StatusCode::BinaryResponseFollows => Some("binary response follows"),
            StatusCode::SendBinaryData => Some("send binary data"),
            StatusCode::ConnectionDedicated => Some("connection dedicated"),
            StatusCode::UnexpectedError => Some("unexpected error"),
            StatusCode::MaxConnectionsExceeded => Some("max number of connections exceeded"),
            StatusCode::FileNotFound => Some("file not found"),
            StatusCode::NoSuchModule => Some("no such module"),
            StatusCode::MemoryNotMapped => Some("memory not mapped"),
            StatusCode::NoSuchThread => Some("no such thread"),
            StatusCode::ClockNotSet => None,
            StatusCode::UnknownCommand => Some("unknown command"),
            StatusCode::NotStopped => Some("not stopped"),
            StatusCode::MustCopy => Some("file must be copied"),
            StatusCode::FileExists => Some("file already exists"),
            StatusCode::NotEmpty => Some("directory not empty"),
            StatusCode::InvalidFilename => Some("filename is invalid"),
            StatusCode::CannotCreate => Some("file cannot be created"),
            StatusCode::AccessDenied => Some("access denied"),
            StatusCode::DeviceFull => Some("no room on device"),
            StatusCode::NotDebuggable => Some("not debuggable"),
            StatusCode::InvalidCounterType => Some("type invalid"),
            StatusCode::NoCounterData => Some("data not available"),
            StatusCode::NotLocked => Some("box not locked"),
            StatusCode::NeedKeyExchange => Some("key exchange required"),
            StatusCode::MustBeDedicated => Some("dedicated connection required"),
            _ => None,
        }
    }

    pub fn is_success(&self) -> bool {
        self.to_u16() < 400
    }

    pub fn is_failure(&self) -> bool {
        self.to_u16() >= 400
    }
}

impl Copy for StatusCode {}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_u16())
    }
}

impl IntoIterator for StatusCode {
    type Item = StatusCode;
    type IntoIter = ::std::option::IntoIter<StatusCode>;

    fn into_iter(self) -> Self::IntoIter {
        Some(self).into_iter()
    }
}

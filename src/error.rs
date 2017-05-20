// Copyright 2017 xbdm-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::error;
use std::fmt;
use std::io;
use std::result;

use status::StatusCode;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum ErrorKind {
    /// An I/O error occurred.
    Io(io::Error),
    /// A response line was malformed or unexpected.
    BadResponse(String),
    /// A command returned a 4xx status code.
    CommandFailed(StatusCode, String),
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    command: String,
}

impl Error {
    pub fn new<C: Into<String>>(kind: ErrorKind, command: C) -> Error {
        Error { kind: kind, command: command.into() }
    }

    pub fn io<C: Into<String>>(err: io::Error, command: C) -> Error {
        Error::new(ErrorKind::Io(err), command)
    }

    pub fn io_custom<E, C>(kind: io::ErrorKind, error: E, command: C) -> Error
        where E: Into<Box<error::Error + Send + Sync>>, C: Into<String>
    {
        Error::new(ErrorKind::Io(io::Error::new(kind, error)), command)
    }

    pub fn bad_response<D, C>(desc: D, command: C) -> Error
        where D: Into<String>, C: Into<String>
    {
        Error::new(ErrorKind::BadResponse(desc.into()), command)
    }

    pub fn command_failed<M, C>(code: StatusCode, message: M, command: C) -> Error
        where M: Into<String>, C: Into<String>
    {
        Error::new(ErrorKind::CommandFailed(code, message.into()), command)
    }

    pub fn kind(&self) -> &ErrorKind { &self.kind }
    pub fn command(&self) -> &str { &self.command }

    /// If true, this `Error` requires the `Client` to reconnect.
    pub fn is_fatal(&self) -> bool {
        match self.kind {
            ErrorKind::Io(_) | ErrorKind::BadResponse(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let have_command = self.command.len() > 0;
        if have_command {
            write!(f, "command failed: '{}' (", self.command)?;
        }
        match self.kind {
            ErrorKind::Io(ref err) => write!(f, "I/O error: {}", err)?,
            ErrorKind::BadResponse(ref desc) => write!(f, "bad response: {}", desc)?,
            ErrorKind::CommandFailed(code, ref msg) => write!(f, "{}- {}", code, msg)?,
        }
        if have_command {
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::Io(ref err) => err.description(),
            ErrorKind::BadResponse(ref desc) => desc.as_ref(),
            ErrorKind::CommandFailed(_, ref msg) => msg.as_ref(),
        }
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        match self.kind {
            ErrorKind::Io(ref err) => Some(err),
            _ => None,
        }
    }
}

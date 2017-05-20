// Copyright 2017 xbdm-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::io;
use std::io::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};

use bufstream::BufStream;

use status::StatusCode;
use error::{Error, Result};

fn send_command<W: Write>(writer: &mut W, command: &str) -> Result<()> {
    writer.write_all(command.as_bytes())
        .and_then(|_| writer.write_all("\r\n".as_bytes()))
        .and_then(|_| writer.flush())
        .map_err(|e| Error::io(e, command))
}

fn read_response<R, E>(reader: &mut R, expect: E, command: &str)
    -> Result<(StatusCode, String)>
    where R: io::BufRead, E: IntoIterator<Item=StatusCode>
{
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(0) |
        Ok(_) if !line.ends_with("\n") => return Err(Error::io_custom(
            io::ErrorKind::UnexpectedEof, "did not receive a line", command)),
        Err(e) => return Err(Error::io(e, command)),
        _ => {},
    }

    line.pop();
    if line.ends_with("\r") {
        line.pop();
    }

    if line.len() < 5 {
        return Err(Error::bad_response("too short", command));
    }

    let code = StatusCode::from_u16(line[0..3].parse().map_err(|_| {
        Error::bad_response("invalid status code", command)
    })?);

    let message = line[5..].to_owned();

    if code.is_failure() {
        Err(Error::command_failed(code, message, command))
    } else if expect.into_iter().any(|c| code == c) {
        Ok((code, message))
    } else {
        Err(Error::bad_response(
            format!("unexpected response: {}- {}", code, message), command))
    }
}

#[derive(Debug)]
enum Stream<S: BufRead + Write> {
    None,
    Raw(S),
    Dot(DotReader<S>),
    Take(io::Take<S>),
    Give(Give<S>),
}

impl<S: BufRead + Write> Stream<S> {
    pub fn into_inner(self) -> S {
        match self {
            Stream::None => unreachable!(),
            Stream::Raw(s) => s,
            Stream::Dot(s) => s.into_inner(),
            Stream::Take(s) => s.into_inner(),
            Stream::Give(s) => s.into_inner(),
        }
    }
}

/// An Xbox Debug Monitor client.
#[derive(Debug)]
pub struct Client {
    stream: Stream<BufStream<TcpStream>>,
}

impl Client {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Client> {
        let mut stream = BufStream::new(TcpStream::connect(addr)
            .map_err(|e| Error::io(e, "connect"))?);
        read_response(&mut stream, StatusCode::Connected, "connect")?;
        Ok(Client { stream: Stream::Raw(stream) })
    }

    pub fn execute<'a, E>(&'a mut self, expect: E, command: &'a str)
        -> Result<Execute>
        where E: IntoIterator<Item=StatusCode>
    {
        let (code, message) = if let Stream::Raw(ref mut s) = self.stream {
            send_command(s, command)?;
            read_response(s, expect, command)?
        } else {
            unreachable!()
        };

        if code == StatusCode::MultilineResponseFollows {
            let mut stream = Stream::None;
            ::std::mem::swap(&mut stream, &mut self.stream);
            self.stream = Stream::Dot(DotReader::new(stream.into_inner()));
        }

        Ok(Execute {
            client: self,
            command: command,
            code: code,
            message: message,
        })
    }
}

#[derive(Debug)]
pub struct Execute<'client> {
    client: &'client mut Client,
    command: &'client str,
    code: StatusCode,
    message: String,
}

impl<'client> Execute<'client> {
    /// The command that was passed to [`execute`].
    ///
    /// [`execute`]: struct.Client.html#method.execute
    pub fn command(&self) -> &str {
        self.command
    }

    /// The initial response code.
    pub fn code(&self) -> StatusCode {
        self.code
    }

    /// The initial response message.
    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn limit(&self) -> Option<u64> {
        match self.client.stream {
            Stream::Take(ref s) => Some(s.limit()),
            Stream::Give(ref s) => Some(s.limit()),
            _ => None,
        }
    }

    // TODO: pub fn set_limit(&mut self, limit: u64)

    pub fn finish(self) -> Result<(StatusCode, String)> {
        let command = self.command;

        match (&mut self.client.stream, self.code) {
            (&mut Stream::Dot(ref mut s), StatusCode::MultilineResponseFollows) => {
                io::copy(s, &mut io::sink())
                    .map_err(|e| Error::io(e, command))?;
            },
            (&mut Stream::Take(ref mut s), StatusCode::BinaryResponseFollows) => {
                io::copy(s, &mut io::sink())
                    .map_err(|e| Error::io(e, command))?;
            },
            // TODO: We should probably warn the user that the connection state
            // may be invalid if they didn't read/write all of the data.
            (&mut Stream::Give(_), StatusCode::SendBinaryData) => {},
            (&mut Stream::Raw(_), StatusCode::BinaryResponseFollows) => {},
            (&mut Stream::Raw(_), StatusCode::SendBinaryData) => {},
            (&mut Stream::Raw(_), _) => {},
            _ => { unreachable!(); },
        }

        let mut stream = Stream::None;
        ::std::mem::swap(&mut stream, &mut self.client.stream);
        self.client.stream = Stream::Raw(stream.into_inner());

        Ok((self.code, self.message))
    }
}

impl<'client> BufRead for Execute<'client> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        match self.client.stream {
            Stream::None => unreachable!(),
            Stream::Raw(ref mut s) => s.fill_buf(),
            Stream::Dot(ref mut s) => s.fill_buf(),
            Stream::Take(ref mut s) => s.fill_buf(),
            Stream::Give(_) => {
                let x: &'static [u8] = &[];
                Ok(x)
            },
        }
    }

    fn consume(&mut self, amt: usize) {
        match self.client.stream {
            Stream::None => unreachable!(),
            Stream::Raw(ref mut s) => s.consume(amt),
            Stream::Dot(ref mut s) => s.consume(amt),
            Stream::Take(ref mut s) => s.consume(amt),
            Stream::Give(_) => {},
        }
    }
}

impl<'client> Read for Execute<'client> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.client.stream {
            Stream::None => unreachable!(),
            Stream::Raw(ref mut s) => s.read(buf),
            Stream::Dot(ref mut s) => s.read(buf),
            Stream::Take(ref mut s) => s.read(buf),
            Stream::Give(_) => Ok(0),
        }
    }
}

impl<'client> Write for Execute<'client> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.client.stream {
            Stream::None => unreachable!(),
            Stream::Raw(ref mut s) => s.write(buf),
            Stream::Give(ref mut s) => s.write(buf),
            _ => Ok(0),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.client.stream {
            Stream::None => unreachable!(),
            Stream::Raw(ref mut s) => s.flush(),
            Stream::Give(ref mut s) => s.flush(),
            _ => Err(io::Error::new(io::ErrorKind::WriteZero, "not writable")),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum DotState {
    BeginLine,
    Dot,
    DotCr,
    Cr,
    Data,
    Eof,
}

#[derive(Debug)]
struct DotReader<R: Read> {
    inner: R,
    state: DotState,
    saved: Option<u8>,
}

impl<R: Read> DotReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner: inner,
            state: DotState::BeginLine,
            saved: None
        }
    }

    pub fn into_inner(self) -> R { self.inner }
}

impl<R: BufRead> BufRead for DotReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt)
    }
}

impl<R: Read> Read for DotReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut n = 0;
        while n < buf.len() && self.state != DotState::Eof {
            let mut c = if let Some(c) = self.saved.take() {
                c
            } else {
                let mut c = [0];
                self.inner.read_exact(&mut c)?;
                c[0]
            };
            self.state = match (self.state, c) {
                (DotState::BeginLine, b'.') => DotState::Dot,
                (DotState::BeginLine, b'\r') => DotState::Cr,
                (DotState::BeginLine, _) => DotState::Data,
                (DotState::Dot, b'\r') => DotState::DotCr,
                (DotState::Dot, b'\n') => DotState::Eof,
                (DotState::Dot, _) => DotState::Data,
                (DotState::DotCr, b'\n') => DotState::Eof,
                (DotState::Cr, b'\n') => DotState::BeginLine,
                (DotState::Data, b'\r') => DotState::Cr,
                (DotState::Data, b'\n') => DotState::BeginLine,
                (DotState::Data, _) => DotState::Data,
                (DotState::DotCr, _) | (DotState::Cr, _) => {
                    self.saved = Some(c);
                    c = b'\r';
                    DotState::Data
                },
                _ => unreachable!(),
            };
            if self.state == DotState::Data || self.state == DotState::BeginLine {
                buf[n] = c;
                n += 1;
            }
        }
        Ok(n)
    }
}

#[derive(Debug)]
struct Give<T> {
    inner: T,
    limit: u64,
}

impl<T> Give<T> {
    pub fn new(inner: T, limit: u64) -> Give<T> {
        Give {
            inner: inner,
            limit: limit,
        }
    }

    pub fn limit(&self) -> u64 { self.limit }
    pub fn into_inner(self) -> T { self.inner }
}

impl<T: Write> Write for Give<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.limit == 0 {
            return Ok(0);
        }
        let max = ::std::cmp::min(buf.len() as u64, self.limit) as usize;
        let n = self.inner.write(&buf[..max])?;
        self.limit -= n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[test]
fn test_read_response() {
    assert_eq!(read_response(&mut io::Cursor::new("200- OK\r\n"), StatusCode::Ok, "").unwrap(), (StatusCode::Ok, "OK".to_owned()));
}

#[test]
fn test_dot_reader() {
    let mut s = String::new();
    DotReader::new(io::Cursor::new("foo\r\n.\r\n")).read_to_string(&mut s).unwrap();
    assert_eq!(s, "foo\n");
}

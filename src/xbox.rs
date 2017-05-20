use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::option;
use std::time::Duration;

use super::{MAX_NAME_LENGTH, PORT_360, PORT_CLASSIC};

const RESOLVE_TIMEOUT_MILLIS: u64 = 300;
const MAX_PACKET_LENGTH: usize = MAX_NAME_LENGTH + 2;

/// Describes an Xbox Development Kit found by a discover or resolve operation.
#[derive(Debug)]
pub struct Xbox {
    ip: Ipv4Addr,
    port: u16,
    name: String,
}

impl Xbox {
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(self.ip), self.port)
    }
    pub fn ip(&self) -> Ipv4Addr { self.ip }
    pub fn port(&self) -> u16 { self.port }
    pub fn name(&self) -> &str { &self.name }
    pub fn is_360(&self) -> bool { self.port == PORT_360 }
    pub fn is_classic(&self) -> bool { self.port == PORT_CLASSIC }
}

impl ToSocketAddrs for Xbox {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(Some(self.socket_addr()).into_iter())
    }
}

fn parse_reply(data: &[u8], src: SocketAddr) -> Option<Xbox> {
    if data.len() < 3 || data[0] != 2 || data[1] == 0 {
        return None
    }
    if src.port() != PORT_360 && src.port() != PORT_CLASSIC {
        return None
    }
    Some(Xbox {
        ip: match src.ip() {
            IpAddr::V4(ip) => ip,
            _ => return None,
        },
        port: src.port(),
        name: match String::from_utf8(data[2..(data[1] as usize) + 2].into()) {
            Ok(s) => s,
            Err(_) => return None,
        },
    })
}

/// An iterator over `Xbox` instances returned from a discover operation.
pub struct Discover {
    socket: UdpSocket,
}

impl Iterator for Discover {
    type Item = Xbox;
    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = [0; MAX_PACKET_LENGTH];
        loop {
            let (n, src) = match self.socket.recv_from(&mut buf) {
                Ok(x) => x,
                Err(_) => break,
            };
            if let Some(xbox) = parse_reply(&buf[..n], src) {
                return Some(xbox)
            }
        }
        None
    }
}

/// Discover active Xbox Development Kits on the local network.
pub fn discover() -> io::Result<Discover> {
    let ip = Ipv4Addr::new(255, 255, 255, 255);
    let pkt = [3, 0];
    let timeout = Some(Duration::from_millis(300));
    let socket = UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 0))?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(timeout)?;
    socket.set_write_timeout(timeout)?;
    socket.send_to(&pkt, (ip, PORT_360))?;
    socket.send_to(&pkt, (ip, PORT_CLASSIC))?;
    Ok(Discover { socket: socket })
}

/// Resolve the Xbox debug name or IP address specified by `host`
/// as an `Xbox` instance.
pub fn resolve(host: &str) -> io::Result<Option<Xbox>> {
    match host.parse() {
        Ok(ip) => resolve_ip(ip),
        _ => resolve_name(host),
    }
}

fn is_timeout(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut
}

/// Resolve the IP address specified by `ip` as an `Xbox` instance.
pub fn resolve_ip(ip: Ipv4Addr) -> io::Result<Option<Xbox>> {
    let mut buf = [0; MAX_PACKET_LENGTH];
    buf[0] = 3;
    buf[1] = 0;
    let timeout = Some(Duration::from_millis(RESOLVE_TIMEOUT_MILLIS));
    let socket = UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 0))?;
    socket.set_read_timeout(timeout)?;
    socket.set_write_timeout(timeout)?;
    socket.send_to(&buf[..2], (ip, PORT_360))?;
    socket.send_to(&buf[..2], (ip, PORT_CLASSIC))?;
    loop {
        let (n, src) = match socket.recv_from(&mut buf) {
            Ok(x) => x,
            Err(ref e) if is_timeout(e) => break,
            Err(e) => return Err(e),
        };
        if let Some(xbox) = parse_reply(&buf[..n], src) {
            if xbox.ip == ip {
                return Ok(Some(xbox));
            }
        }
    }
    Ok(None)
}

/// Resolve the Xbox debug name specified by `name` as an `Xbox` instance.
pub fn resolve_name(name: &str) -> io::Result<Option<Xbox>> {
    if name.len() == 0 {
        return Ok(None)
    } else if name.len() > MAX_NAME_LENGTH {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput, "name is too long"))
    }

    let timeout = Some(Duration::from_millis(RESOLVE_TIMEOUT_MILLIS));
    let socket = UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 0))?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(timeout)?;
    socket.set_write_timeout(timeout)?;

    let mut buf = &mut [0; MAX_PACKET_LENGTH][..name.len()+2];
    buf[0] = 1;
    buf[1] = name.len() as u8;
    buf[2..].copy_from_slice(name.as_bytes());

    let ip = Ipv4Addr::new(255, 255, 255, 255);
    socket.send_to(&buf, (ip, PORT_360))?;
    socket.send_to(&buf, (ip, PORT_CLASSIC))?;

    loop {
        let (n, src) = match socket.recv_from(&mut buf) {
            Ok(x) => x,
            Err(ref e) if is_timeout(e) => break,
            Err(e) => return Err(e),
        };
        if let Some(xbox) = parse_reply(&buf[..n], src) {
            if xbox.name == name {
                return Ok(Some(xbox));
            }
        }
    }

    Ok(None)
}

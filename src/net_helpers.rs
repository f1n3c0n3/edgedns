use nix::fcntl::FcntlArg::F_SETFL;
use nix::fcntl::{fcntl, O_NONBLOCK};
use nix::sys::socket::{bind, listen, setsockopt, sockopt, AddressFamily, SockFlag, SockType,
                       SockLevel, SockAddr, socket, InetAddr};
use std::net::{self, SocketAddr, UdpSocket};
use std::io;
use std::os::unix::io::{RawFd, FromRawFd};
use std::str::FromStr;
use super::{TCP_BACKLOG, UDP_BUFFER_SIZE};

#[inline]
pub fn socket_tcp_v4() -> io::Result<RawFd> {
    let socket_fd = try!(socket(AddressFamily::Inet,
                                SockType::Stream,
                                SockFlag::empty(),
                                SockLevel::Tcp as i32));
    Ok(socket_fd)
}

#[inline]
pub fn socket_tcp_v6() -> io::Result<RawFd> {
    let socket_fd = try!(socket(AddressFamily::Inet6,
                                SockType::Stream,
                                SockFlag::empty(),
                                SockLevel::Tcp as i32));
    Ok(socket_fd)
}

pub fn socket_tcp_bound(addr: &str) -> io::Result<net::TcpListener> {
    let actual: SocketAddr = FromStr::from_str(addr).expect("Invalid address");
    let nix_addr = SockAddr::Inet(InetAddr::from_std(&actual));
    let socket_fd = match actual {
        SocketAddr::V4(_) => try!(socket_tcp_v4()),
        SocketAddr::V6(_) => try!(socket_tcp_v6()),
    };
    let _ = setsockopt(socket_fd, sockopt::ReuseAddr, &true);
    let _ = setsockopt(socket_fd, sockopt::ReusePort, &true);
    let _ = setsockopt(socket_fd, sockopt::TcpNoDelay, &true);
    bind(socket_fd, &nix_addr).expect("Unable to bind a TCP socket");
    listen(socket_fd, TCP_BACKLOG).expect("Unable to listen to the TCP socket");
    let socket = unsafe { net::TcpListener::from_raw_fd(socket_fd) };
    Ok(socket)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn socket_udp_set_buffer_size(socket_fd: RawFd) {
    let _ = setsockopt(socket_fd, sockopt::SndBufForce, &UDP_BUFFER_SIZE);
    let _ = setsockopt(socket_fd, sockopt::RcvBufForce, &UDP_BUFFER_SIZE);
}

#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub fn socket_udp_set_buffer_size(socket_fd: RawFd) {
    let _ = setsockopt(socket_fd, sockopt::SndBuf, &UDP_BUFFER_SIZE);
    let _ = setsockopt(socket_fd, sockopt::RcvBuf, &UDP_BUFFER_SIZE);
}

#[inline]
pub fn socket_udp_v4() -> io::Result<RawFd> {
    let socket_fd = try!(socket(AddressFamily::Inet,
                                SockType::Datagram,
                                SockFlag::empty(),
                                SockLevel::Udp as i32));
    Ok(socket_fd)
}

#[inline]
pub fn socket_udp_v6() -> io::Result<RawFd> {
    let socket_fd = try!(socket(AddressFamily::Inet6,
                                SockType::Datagram,
                                SockFlag::empty(),
                                SockLevel::Udp as i32));
    Ok(socket_fd)
}

pub fn socket_udp_bound(addr: &str) -> io::Result<UdpSocket> {
    let actual: SocketAddr = FromStr::from_str(addr).expect("Invalid address");
    let nix_addr = SockAddr::Inet(InetAddr::from_std(&actual));
    let socket_fd = match actual {
        SocketAddr::V4(_) => try!(socket_udp_v4()),
        SocketAddr::V6(_) => try!(socket_udp_v6()),
    };
    let _ = setsockopt(socket_fd, sockopt::ReuseAddr, &true);
    let _ = setsockopt(socket_fd, sockopt::ReusePort, &true);
    socket_udp_set_buffer_size(socket_fd);
    bind(socket_fd, &nix_addr).expect("Unable to bind a UDP socket");
    let socket = unsafe { UdpSocket::from_raw_fd(socket_fd) };
    Ok(socket)
}

#[inline]
pub fn set_nonblock(sock: RawFd) -> io::Result<()> {
    try!(fcntl(sock, F_SETFL(O_NONBLOCK)));
    Ok(())
}

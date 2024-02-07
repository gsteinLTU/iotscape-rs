
#[cfg(feature = "std")]
extern crate std;

use alloc::{
    collections::VecDeque,
    string::{String, ToString},
    vec::Vec, 
    format,
};
use core::time::Duration;

#[cfg(feature = "std")]
use std::net::SocketAddr;

#[cfg(not(feature = "std"))]
use no_std_net::SocketAddr;

#[cfg(feature = "std")]
use std::net::UdpSocket as StdUdpSocket;

#[cfg(feature = "tokio")]
use tokio::net::UdpSocket as TokioUdpSocket;


/// Trait to allow various socket types to be used with IoTScapeService
pub trait SocketTrait : Sized {
    fn bind(addrs: &[SocketAddr]) -> Result<Self, String>;
    fn send_to(&self, buf: &[u8], addr: SocketAddr) -> Result<usize, String>;
    fn recv(&self, buf: &mut [u8]) -> Result<usize, String>;
    fn set_read_timeout(&self, timeout: Option<Duration>) -> Result<(), String>;
    fn set_write_timeout(&self, timeout: Option<Duration>) -> Result<(), String>;
}

#[cfg(feature = "tokio")]
pub trait SocketTraitAsync : Sized {
    fn bind(addr: &SocketAddr) -> impl std::future::Future<Output = Result<Self, std::io::Error>> + Send;
    fn send_to(&self, buf: &[u8], addr: SocketAddr) -> impl std::future::Future<Output = Result<usize, std::io::Error>> + Send;
    fn recv(&self, buf: &mut [u8]) -> impl std::future::Future<Output = Result<usize, std::io::Error>> + Send;
}

#[cfg(feature = "std")]
impl SocketTrait for StdUdpSocket {
    fn bind(addrs: &[SocketAddr]) -> Result<Self, String> {
        let socket = StdUdpSocket::bind(addrs.iter().map(|s| s.to_string().parse().unwrap()).collect::<Vec<std::net::SocketAddr>>().as_slice());
        if socket.is_err() {
            return socket.map_err(|e| format!("{}", e))
        } else {
            let socket = socket.unwrap();
            if let Err(e) = socket.set_nonblocking(true) {
                return Err(format!("{}", e));
            }
            return Ok(socket);
        }
    }

    fn send_to(&self, buf: &[u8], addr: SocketAddr) -> Result<usize, String> {
        StdUdpSocket::send_to(self, buf, addr).map_err(|e| e.to_string())
    }

    fn recv(&self, buf: &mut [u8]) -> Result<usize, String> {
        StdUdpSocket::recv(self, buf).map_err(|e| e.to_string())
    }

    fn set_read_timeout(&self, timeout: Option<Duration>) -> Result<(), String> {
        StdUdpSocket::set_read_timeout(self, timeout).map_err(|e| e.to_string())
    }

    fn set_write_timeout(&self, timeout: Option<Duration>) -> Result<(), String> {
        StdUdpSocket::set_write_timeout(self, timeout).map_err(|e| e.to_string())
    }
}


#[cfg(feature = "tokio")]
impl SocketTraitAsync for TokioUdpSocket {
    async fn bind(addr: &SocketAddr) -> Result<Self, std::io::Error> {
        TokioUdpSocket::bind(addr).await
    }

    async fn send_to(&self, buf: &[u8], addr: SocketAddr) -> Result<usize, std::io::Error> {
        TokioUdpSocket::send_to(&self, buf, addr).await
    }

    async fn recv(&self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        TokioUdpSocket::recv(self, buf).await
    }
}

/// SocketTrait impl with an internal message queue for testing purposes
pub struct MockSocket {
    pub data: core::cell::RefCell<VecDeque<Vec<u8>>>,
}

impl SocketTrait for MockSocket {
    fn bind(_addrs: &[SocketAddr]) -> Result<Self, String> {
        Ok(MockSocket{ data: core::cell::RefCell::new(VecDeque::new()) })
    }

    fn send_to(&self, buf: &[u8], _addr: SocketAddr) -> Result<usize, String> {
        let mut i: usize = 0; 

        while i < buf.len() {
            if buf[i] == 0 {
                break;
            }

            i += 1;
        }

        Ok(i)
    }

    fn recv(&self, buf: &mut [u8]) -> Result<usize, String> {
        if self.data.borrow().len() > 0 {
            let packet = self.data.borrow_mut().pop_front().unwrap();
            buf.copy_from_slice(packet.as_slice());
            return Ok(packet.len());
        }

        Err("No packets".into())
    }

    fn set_read_timeout(&self, _timeout: Option<Duration>) -> Result<(), String> {
        Ok(())
    }

    fn set_write_timeout(&self, _timeout: Option<Duration>) -> Result<(), String> {
        Ok(())
    }
}

/// SocketTrait impl which does nothing
pub struct NullSocket {}

impl SocketTrait for NullSocket {
    fn bind(_addrs: &[SocketAddr]) -> Result<Self, String> {
        Ok(NullSocket{})
    }

    fn send_to(&self, buf: &[u8], _addr: SocketAddr) -> Result<usize, String> {
        let mut i: usize = 0; 

        while i < buf.len() {
            if buf[i] == 0 {
                break;
            }

            i += 1;
        }

        Ok(i)
    }

    fn recv(&self, _buf: &mut [u8]) -> Result<usize, String> {
        Ok(0)
    }

    fn set_read_timeout(&self, _timeout: Option<Duration>) -> Result<(), String> {
        Ok(())
    }

    fn set_write_timeout(&self, _timeout: Option<Duration>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(feature = "tokio")]
impl SocketTraitAsync for NullSocket {
    async fn bind(_: &SocketAddr) -> Result<Self, std::io::Error> {
        Ok(NullSocket{})
    }

    async fn send_to(&self, buf: &[u8], _: SocketAddr) -> Result<usize, std::io::Error> {
        let mut i: usize = 0; 

        while i < buf.len() {
            if buf[i] == 0 {
                break;
            }

            i += 1;
        }

        Ok(i)
    }

    async fn recv(&self, _: &mut [u8]) -> Result<usize, std::io::Error> {
        Ok(0)
    }
}
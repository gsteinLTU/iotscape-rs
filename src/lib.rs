#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use core::time::Duration;

#[cfg(feature = "std")]
use std::net::SocketAddr;

#[cfg(not(feature = "std"))]
use no_std_net::SocketAddr;


#[cfg(feature = "std")]
use std::net::UdpSocket;

use alloc::{
    borrow::ToOwned,
    collections::{BTreeMap, VecDeque},
    string::{String, ToString},
    vec::Vec, 
    format,
};

use log::{error, trace};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A request sent from the NetsBlox server
#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub service: String,
    pub device: String,
    pub function: String,
    pub params: Vec<serde_json::Value>,
}

/// A response to be sent to the NetsBlox server
#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub request: String,
    pub service: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<EventResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Data for an event response to be sent to the server
#[derive(Debug, Serialize, Deserialize)]
pub struct EventResponse {
    pub r#type: Option<String>,
    pub args: Option<BTreeMap<String, String>>,
}

/// Definition of an IoTScape service, to be serialized and set to NetsBlox server
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceDefinition {
    pub id: String,
    pub methods: BTreeMap<String, MethodDescription>,
    pub events: BTreeMap<String, EventDescription>,
    #[serde(rename = "service")]
    pub description: IoTScapeServiceDescription,
}

/// Service meta-data for an IoTScape Service
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct IoTScapeServiceDescription {
    pub description: Option<String>,
    pub externalDocumentation: Option<String>,
    pub termsOfService: Option<String>,
    pub contact: Option<String>,
    pub license: Option<String>,
    pub version: String,
}

/// Describes a method belonging to an IoTScape service
#[derive(Debug, Serialize, Deserialize)]
pub struct MethodDescription {
    pub documentation: Option<String>,
    pub params: Vec<MethodParam>,
    pub returns: MethodReturns,
}

/// Describes a parameter of a method in an IoTScape service
#[derive(Debug, Serialize, Deserialize)]
pub struct MethodParam {
    pub name: String,
    pub documentation: Option<String>,
    pub r#type: String,
    pub optional: bool,
}

/// Describes a return value of a method in an IoTScape service
#[derive(Debug, Serialize, Deserialize)]
pub struct MethodReturns {
    pub documentation: Option<String>,
    pub r#type: Vec<String>,
}

/// Describes an event type in an IoTScape service
#[derive(Debug, Serialize, Deserialize)]
pub struct EventDescription {
    pub params: Vec<String>,
}

/// Trait to allow various socket types to be used with IoTScapeService
pub trait SocketTrait : Sized {
    fn bind(addrs: &[SocketAddr]) -> Result<Self, String>;
    fn send_to(&self, buf: &[u8], addr: SocketAddr) -> Result<usize, String>;
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize, String>;
    fn set_read_timeout(&self, timeout: Option<Duration>) -> Result<(), String>;
    fn set_write_timeout(&self, timeout: Option<Duration>) -> Result<(), String>;
}

#[cfg(feature = "std")]
impl SocketTrait for UdpSocket {
    fn bind(addrs: &[SocketAddr]) -> Result<Self, String> {
        let socket = UdpSocket::bind(addrs.iter().map(|s| s.to_string().parse().unwrap()).collect::<Vec<std::net::SocketAddr>>().as_slice());
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
        UdpSocket::send_to(self, buf, addr).map_err(|e| e.to_string())
    }

    fn recv(&mut self, buf: &mut [u8]) -> Result<usize, String> {
        UdpSocket::recv(self, buf).map_err(|e| e.to_string())
    }

    fn set_read_timeout(&self, timeout: Option<Duration>) -> Result<(), String> {
        UdpSocket::set_read_timeout(self, timeout).map_err(|e| e.to_string())
    }

    fn set_write_timeout(&self, timeout: Option<Duration>) -> Result<(), String> {
        UdpSocket::set_write_timeout(self, timeout).map_err(|e| e.to_string())
    }
}

/// SocketTrait impl with an internal message queue for testing purposes
pub struct MockSocket {
    pub data: VecDeque<Vec<u8>>,
}

impl SocketTrait for MockSocket {
    fn bind(_addrs: &[SocketAddr]) -> Result<Self, String> {
        Ok(MockSocket{ data: VecDeque::new() })
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

    fn recv(&mut self, buf: &mut [u8]) -> Result<usize, String> {
        if self.data.len() > 0 {
            let packet = self.data.pop_front().unwrap();
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

    fn recv(&mut self, _buf: &mut [u8]) -> Result<usize, String> {
        Ok(0)
    }

    fn set_read_timeout(&self, _timeout: Option<Duration>) -> Result<(), String> {
        Ok(())
    }

    fn set_write_timeout(&self, _timeout: Option<Duration>) -> Result<(), String> {
        Ok(())
    }
}




/// An IoTScape service and socket setup to send/receive messages
#[cfg(not(feature = "std"))]
pub struct IoTScapeService<SocketType: SocketTrait> {
    pub definition: ServiceDefinition,
    pub name: String,
    server: SocketAddr,
    socket: SocketType,
    pub next_msg_id: u64,
    pub rx_queue: VecDeque<Request>,
    pub tx_queue: VecDeque<Response>,
}

#[cfg(feature = "std")]
pub struct IoTScapeService<SocketType: SocketTrait = UdpSocket> {
    pub definition: ServiceDefinition,
    cached_definition: Option<String>,
    pub name: String,
    server: SocketAddr,
    socket: SocketType,
    pub next_msg_id: u64,
    pub rx_queue: VecDeque<Request>,
    pub tx_queue: VecDeque<Response>,
}

#[cfg(feature = "std")]
pub type IoTScapeServiceUdp = IoTScapeService<UdpSocket>;

impl<SocketType: SocketTrait> IoTScapeService<SocketType> {
    pub fn new(name: &str, definition: ServiceDefinition, server: SocketAddr) -> Self {
        let addrs = [
            SocketAddr::from(([0, 0, 0, 0], 0)),
            SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], 0)),
        ];
        let socket = SocketType::bind(&addrs[..]).unwrap();
        Self {
            name: name.to_owned(),
            definition,
            cached_definition: None,
            socket,
            server,
            rx_queue: VecDeque::<Request>::new(),
            tx_queue: VecDeque::<Response>::new(),
            next_msg_id: 0,
        }
    }

    /// Send the service description to the server
    pub fn announce(&mut self) -> Result<usize, String> {
        // Serialize definition if not already cached
        let mut definition_string = self.cached_definition.as_ref();
        if definition_string.is_none() {
            self.cached_definition = Some(serde_json::to_string(&BTreeMap::from([(
                self.name.to_owned(),
                &self.definition,
            )]))
            .unwrap());
            definition_string = self.cached_definition.as_ref();
        }
        let definition_string = definition_string.unwrap();

        // Send to server
        trace!("Announcing {:?}", definition_string);
        self.socket
            .send_to(definition_string.as_bytes(), self.server)
    }

    /// Handle rx/tx
    pub fn poll(&mut self, timeout: Option<Duration>) {
        self.socket
            .set_read_timeout(timeout.or(Some(Duration::from_millis(15))))
            .unwrap();
        self.socket
            .set_write_timeout(timeout.or(Some(Duration::from_millis(15))))
            .unwrap();

        // Get incoming messages
        loop {
            let mut buf = [0u8; 65_535];
            match self.socket.recv(&mut buf) {
                Ok(size) => {
                    let content = &buf[..size];

                    match serde_json::from_slice::<Request>(content) {
                        Ok(msg) => {
                            // Handle heartbeat immediately
                            if msg.function == "heartbeat" {
                                self.send_response(Response {
                                    id: self.definition.id.clone(),
                                    request: msg.id,
                                    service: msg.service,
                                    response: Some(alloc::vec![]),
                                    event: None,
                                    error: None,
                                }).unwrap();
                                self.next_msg_id += 1;
                            } else {
                                self.rx_queue.push_back(msg);
                            }
                        }
                        Err(e) => {
                            error!("Error parsing request: {}", e);
                        }
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }

        // Send queued messages
        while !self.tx_queue.is_empty() {
            let next_msg = self.tx_queue.pop_front().unwrap();
            if let Err(e) = self.send_response(next_msg) {
                error!("Error sending response: {}", e);
            }
        }
    }

    /// Create a response to an Request and enqueue it for sending
    pub fn enqueue_response_to(
        &mut self,
        request: Request,
        params: Result<Vec<Value>, String>,
    ) -> Result<usize, String> {
        let mut response = None;
        let mut error = None;

        match params {
            Ok(p) => {
                response = Some(p);
            }
            Err(e) => {
                error = Some(e);
            }
        }

        self.send_response(Response {
            id: self.definition.id.clone(),
            request: request.id.to_owned(),
            service: request.service,
            response,
            event: None,
            error,
        }).and_then(|r| { self.next_msg_id += 1; Ok(r) })
    }

    /// Set an event message to be sent
    pub fn send_event(&mut self, call_id: &str, event_type: &str, args: BTreeMap<String, String>) -> Result<usize, String> {
        self.send_response(Response {
            id: self.definition.id.clone(),
            request: call_id.to_owned(),
            service: self.name.to_owned(),
            response: None,
            event: Some(EventResponse {
                r#type: Some(event_type.to_owned()),
                args: Some(args),
            }),
            error: None,
        })
    }

    /// Sends an Response to ther server
    fn send_response(&mut self, response: Response) -> Result<usize, String>{
        let as_string = serde_json::to_string(&response).unwrap();
        trace!("Sending response {:?}", as_string);
        self.socket
            .send_to(as_string.as_bytes(), self.server)
    }
}

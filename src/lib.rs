#![no_std]
#![forbid(unsafe_code)]

mod socket;

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use core::time::Duration;

use alloc::{
    borrow::ToOwned,
    collections::{BTreeMap, VecDeque},
    string::String,
    vec::Vec,
};

use log::{error, trace};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use socket::SocketTrait;

#[cfg(feature = "std")]
use std::net::SocketAddr;

#[cfg(not(feature = "std"))]
use no_std_net::SocketAddr;


#[cfg(feature = "std")]
use std::net::UdpSocket;


/// A request sent from the NetsBlox server
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub id: String,
    pub service: String,
    pub device: String,
    pub function: String,
    pub params: Vec<serde_json::Value>,
    #[serde(rename = "clientId")]
    pub client_id: Option<String>,
}

/// A response to be sent to the NetsBlox server
#[derive(Debug, Serialize, Deserialize, Clone)]
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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventResponse {
    pub r#type: Option<String>,
    pub args: Option<BTreeMap<String, String>>,
}

/// Definition of an IoTScape service, to be serialized and set to NetsBlox server
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceDefinition {
    pub id: String,
    pub methods: BTreeMap<String, MethodDescription>,
    pub events: BTreeMap<String, EventDescription>,
    #[serde(rename = "service")]
    pub description: IoTScapeServiceDescription,
}

/// Service meta-data for an IoTScape Service
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IoTScapeServiceDescription {
    pub description: Option<String>,
    pub externalDocumentation: Option<String>,
    pub termsOfService: Option<String>,
    pub contact: Option<String>,
    pub license: Option<String>,
    pub version: String,
}

/// Describes a method belonging to an IoTScape service
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MethodDescription {
    pub documentation: Option<String>,
    pub params: Vec<MethodParam>,
    pub returns: MethodReturns,
}

/// Describes a parameter of a method in an IoTScape service
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MethodParam {
    pub name: String,
    pub documentation: Option<String>,
    pub r#type: String,
    pub optional: bool,
}

/// Describes a return value of a method in an IoTScape service
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MethodReturns {
    pub documentation: Option<String>,
    pub r#type: Vec<String>,
}

/// Describes an event type in an IoTScape service
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventDescription {
    pub params: Vec<String>,
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

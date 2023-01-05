#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use core::time::Duration;

#[cfg(feature = "std")]
use std::net::{SocketAddr, UdpSocket};

use alloc::{
    borrow::ToOwned,
    collections::{BTreeMap, VecDeque},
    string::String,
    vec::Vec,
};
use serde::{Deserialize, Serialize};

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
    pub response: Option<Vec<String>>,
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

#[cfg(feature = "std")]
/// An IoTScape service and socket setup to send/receive messages
pub struct IoTScapeService {
    pub definition: ServiceDefinition,
    name: String,
    server: SocketAddr,
    socket: UdpSocket,
    pub next_msg_id: u64,
    pub rx_queue: VecDeque<Request>,
    pub tx_queue: VecDeque<Response>,
}

#[cfg(feature = "std")]
impl IoTScapeService {
    pub fn new(name: &str, definition: ServiceDefinition, server: SocketAddr) -> Self {
        Self {
            name: name.to_owned(),
            definition,
            socket: UdpSocket::bind("127.0.0.1:0").unwrap(),
            server,
            rx_queue: VecDeque::<Request>::new(),
            tx_queue: VecDeque::<Response>::new(),
            next_msg_id: 0,
        }
    }

    /// Send the service description to the server
    pub fn announce(&mut self) -> std::io::Result<usize> {
        let definition_string =
            serde_json::to_string(&BTreeMap::from([(
                self.name.to_owned(),
                &self.definition,
            )]))
            .unwrap();

        // Send to server
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
            let mut buf = [0u8; 2048];
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
                                });
                                self.next_msg_id += 1;
                            } else {
                                self.rx_queue.push_back(msg);
                            }
                        }
                        Err(e) => {
                            std::println!("Error parsing request: {}", e);
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
            self.send_response(next_msg);
        }
    }

    /// Create a response to an Request and enqueue it for sending
    pub fn enqueue_response_to(
        &mut self,
        request: Request,
        params: Result<Vec<String>, String>,
    ) {
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
        });

        self.next_msg_id += 1;
    }

    // Set an event message to be sent
    pub fn send_event(&mut self, call_id: &str, event_type: &str, args: BTreeMap<String, String>) {
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
        });
    }

    /// Sends an Response to ther server
    fn send_response(&mut self, response: Response) {
        let as_string = serde_json::to_string(&response).unwrap();
        std::println!("{:?}", as_string);
        self.socket
            .send_to(as_string.as_bytes(), self.server)
            .expect("Error sending response");
    }
}

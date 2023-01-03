#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use core::time::Duration;

#[cfg(feature = "std")]
use std::net::{UdpSocket, SocketAddr};

use alloc::{borrow::ToOwned, collections::{BTreeMap, VecDeque}, string::{String, ToString}, vec::Vec};
use serde::{Deserialize, Serialize};

/// A request sent from the NetsBlox server
#[derive(Debug, Serialize, Deserialize)]
pub struct IoTScapeRequest {
    pub id: String,
    pub service: String,
    pub device: String,
    pub function: String,
    pub params: Vec<serde_json::Value>
}

/// A response to be sent to the NetsBlox server
#[derive(Debug, Serialize, Deserialize)]
pub struct IoTScapeResponse {
    pub id: String,
    pub request: String,
    pub service: String,
    pub response: Option<Vec<String>>,
    pub event: Option<IoTScapeEventResponse>,
    pub error: Option<String>
}

/// Data for an event response to be sent to the server
#[derive(Debug, Serialize, Deserialize)]
pub struct IoTScapeEventResponse {
    pub r#type: Option<String>,
    pub args: Option<Vec<String>>
}

/// Definition of an IoTScape service, to be serialized and set to NetsBlox server
#[derive(Debug, Serialize, Deserialize)]
pub struct IoTScapeServiceDefinition {
    pub id: String,
    pub methods: BTreeMap<String, IoTScapeMethodDescription>,
    pub events: BTreeMap<String, IoTScapeEventDescription>,
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
    pub version: String
}

/// Describes a method belonging to an IoTScape service
#[derive(Debug, Serialize, Deserialize)]
pub struct IoTScapeMethodDescription {
    pub documentation: Option<String>,
    pub params: Vec<IoTScapeMethodParam>,
    pub returns: IoTScapeMethodReturns
}

/// Describes a parameter of a method in an IoTScape service
#[derive(Debug, Serialize, Deserialize)]
pub struct IoTScapeMethodParam {
    pub name: String,
    pub documentation: Option<String>,
    pub r#type: String,
    pub optional: bool
}

/// Describes a return value of a method in an IoTScape service
#[derive(Debug, Serialize, Deserialize)]
pub struct IoTScapeMethodReturns {
    pub documentation: Option<String>,
    pub r#type: Vec<String>
}

/// Describes an event type in an IoTScape service
#[derive(Debug, Serialize, Deserialize)]
pub struct IoTScapeEventDescription {
    pub params: Vec<String>
}

#[cfg(feature = "std")]
/// An IoTScape service and socket setup to send/receive messages
pub struct IoTScapeService {
    pub definition: IoTScapeServiceDefinition,
    name: String,
    server: SocketAddr,
    socket: UdpSocket,
    pub next_msg_id: u64,
    pub rx_queue: VecDeque<IoTScapeRequest>,
    pub tx_queue: VecDeque<IoTScapeResponse>
}

#[cfg(feature = "std")]
impl IoTScapeService {
    pub fn new(name: &str, definition: IoTScapeServiceDefinition, server: SocketAddr ) -> Self { 
        Self { 
            name: name.to_owned(), 
            definition, 
            socket: UdpSocket::bind("127.0.0.1:0").unwrap(),
            server, 
            rx_queue: VecDeque::<IoTScapeRequest>::new(),
            tx_queue: VecDeque::<IoTScapeResponse>::new(),
            next_msg_id: 0
        }
    }

    /// Send the service description to the server
    pub fn announce(&mut self) -> Result<usize, std::io::Error> {
        
        let definition_string = serde_json::to_string(&BTreeMap::<String, &IoTScapeServiceDefinition>::from([(self.name.to_owned(), &self.definition)])).unwrap();

        // Send to server
        self.socket.send_to(definition_string.as_bytes(), self.server)
    }

    /// Handle rx/tx
    pub fn poll(&mut self, timeout: Option<Duration>) {
        self.socket.set_read_timeout(timeout.or(Some(Duration::from_millis(15)))).unwrap();
        self.socket.set_write_timeout(timeout.or(Some(Duration::from_millis(15)))).unwrap();

        // Get incoming messages
        loop {
            let mut buf: [u8; 2048] = [0; 2048];
            match self.socket.recv(&mut buf) {
                Ok(size) => {
                    let content = buf.split_at(size).0;

                    match serde_json::from_slice::<IoTScapeRequest>(content) {
                        Ok(msg) => {
                            // Handle heartbeat immediately
                            if msg.function == "heartbeat" {
                                self.send_response(IoTScapeResponse {
                                    id: self.next_msg_id.to_string(),
                                    request: msg.id,
                                    service: msg.service,
                                    response: None,
                                    event: None,
                                    error: None,
                                });
                                self.next_msg_id += 1;
                            } else {
                                self.rx_queue.push_back(msg);
                            }
                        },
                        Err(e) => {
                            std::println!("Error parsing request: {}", e);
                        }
                    }
                },
                Err(_) => {
                    break;
                }
            }
        }

        // Send queued messages
        while self.tx_queue.len() > 0 {
            let next_msg = self.tx_queue.pop_front().unwrap();
            self.send_response(next_msg);
        }
    }

    /// Create a response to an IoTScapeRequest and enqueue it for sending
    pub fn enqueue_response_to(&mut self, request: IoTScapeRequest, params: Result<Vec<String>, String>) {

        let mut response = None;
        let mut error = None;

        match params {
            Ok(p) => {
                response = Some(p);
            },
            Err(e) => {
                error = Some(e);
            }
        }
        
        self.send_response(IoTScapeResponse {
            id: self.next_msg_id.to_string(),
            request: request.id.to_owned(),
            service: request.service.to_owned(),
            response,
            event: None,
            error 
        });

        self.next_msg_id += 1;
    }

    /// Sends an IoTScapeResponse to ther server
    fn send_response(&mut self, response: IoTScapeResponse) {
        self.socket.send_to(serde_json::to_string(&response).unwrap().as_bytes(), self.server);
    }
}


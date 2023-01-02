#![no_std]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::net::{UdpSocket, SocketAddr};

use alloc::borrow::ToOwned;
use alloc::{collections::BTreeMap, vec};
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// A request sent from the NetsBlox server
#[derive(Debug, Serialize, Deserialize)]
pub struct IoTScapeRequest {
    pub id: String,
    pub service: Vec<String>,
    pub device: String,
    pub function: Option<String>,
    pub paramsList: Vec<String>
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
    pub paramsList: Vec<String>
}

#[cfg(feature = "std")]
/// An IoTScape service and socket setup to send/receive messages
pub struct IoTScapeService {
    pub definition: IoTScapeServiceDefinition,
    name: String,
    server: SocketAddr,
    socket: UdpSocket
}

#[cfg(feature = "std")]
impl IoTScapeService {
    pub fn new(name: &str, definition: IoTScapeServiceDefinition, server: SocketAddr ) -> Self { 
        Self { name: name.to_owned(), definition, socket: UdpSocket::bind("127.0.0.1:0").unwrap(), server}
    }

    pub fn announce(&mut self) {
        let definition_string = serde_json::to_string(&BTreeMap::<String, &IoTScapeServiceDefinition>::from([(self.name.to_owned(), &self.definition)])).unwrap();

        // Send to server
        let result = self.socket.send_to(definition_string.as_bytes(), self.server);
    }
}


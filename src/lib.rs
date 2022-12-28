#![no_std]
extern crate alloc;
use alloc::{collections::BTreeMap, vec};
use alloc::string::String;
use alloc::vec::Vec;
use smoltcp::storage::PacketMetadata;
use smoltcp::wire::IpEndpoint;
use smoltcp::{socket::UdpSocket, storage::PacketBuffer};
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
    pub name: String,
    pub id: String,
    pub methods: BTreeMap<String, IoTScapeMethodDescription>,
    pub events: BTreeMap<String, IoTScapeEventDescription>,
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
    pub paramsList: Vec<IoTScapeMethodParam>,
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

/// An IoTScape service and socket setup to send/receive messages
pub struct IoTScapeService<'a> {
    pub definition: IoTScapeServiceDefinition,
    server: IpEndpoint,
    socket: UdpSocket<'a>
}

impl<'a> IoTScapeService<'a> {
    pub fn new(definition: IoTScapeServiceDefinition, server: IpEndpoint ) -> Self { 
        let mut rx_buffer = PacketBuffer::<'a, _>::new(vec![PacketMetadata::EMPTY, PacketMetadata::EMPTY],vec![0; 65535]);
        let mut tx_buffer = PacketBuffer::<'a, _>::new(vec![PacketMetadata::EMPTY, PacketMetadata::EMPTY], vec![0; 65535]);
        Self { definition, socket: UdpSocket::<'a>::new(rx_buffer, tx_buffer), server } 
    }

    pub fn announce(&self) {

    }
}


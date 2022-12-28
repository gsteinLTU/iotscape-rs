#![no_std]
extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug)]
pub struct IoTScapeRequest {
    id: String,
    service: Vec<String>,
    device: String,
    function: Option<String>,
    paramsList: Vec<String>
}

#[derive(Debug)]
pub struct IoTScapeResponse {
    id: String,
    request: String,
    service: String,
    response: Option<Vec<String>>,
    event: Option<IoTScapeEventResponse>,
    error: Option<String>
}

#[derive(Debug)]
pub struct IoTScapeEventResponse {
    r#type: Option<String>,
    args: Option<Vec<String>>
}

#[derive(Debug)]
pub struct IoTScapeService {
    service_name: String
}

#[derive(Debug)]
pub struct IoTScapeServiceDefinition {
    name: String,
    id: String,
    methods: BTreeMap<String, IoTScapeMethodDescription>,
    events: BTreeMap<String, IoTScapeEventDescription>,
    description: IoTScapeServiceDescription,
}

#[derive(Debug)]
pub struct IoTScapeServiceDescription {
    description: Option<String>,
    externalDocumentation: Option<String>,
    termsOfService: Option<String>,
    contact: Option<String>,
    license: Option<String>,
    version: Option<String>
}

#[derive(Debug)]
pub struct IoTScapeMethodDescription {
    documentation: Option<String>,
    paramsList: Vec<IoTScapeMethodParam>,
    returns: IoTScapeMethodReturns
}

#[derive(Debug)]
pub struct IoTScapeMethodParam {
    name: String,
    documentation: Option<String>,
    r#type: String,
    optional: bool
}

#[derive(Debug)]
pub struct IoTScapeMethodReturns {
    documentation: Option<String>,
    r#type: Vec<String>
}

#[derive(Debug)]
pub struct IoTScapeEventDescription {
    paramsList: Vec<String>
}

#[derive(Debug)]
pub struct IoTScapeServer {

}
use std::{collections::BTreeMap};

use iotscape::*;
use smoltcp::wire::IpAddress;

fn main() {
    let mut definition = IoTScapeServiceDefinition { 
        name: "ExampleService".to_owned(), 
        id: "rs1".to_owned(), 
        methods: BTreeMap::new(), 
        events: BTreeMap::new(), 
        description: IoTScapeServiceDescription { 
            description: Some("Test IoTScape service.".to_owned()), 
            externalDocumentation: None, 
            termsOfService: None, 
            contact: Some("gstein@ltu.edu".to_owned()), 
            license: None, version: "1".to_owned() 
        }};

    let service: IoTScapeService = IoTScapeService::new(definition, smoltcp::wire::IpEndpoint { addr: IpAddress::v4(127, 0, 0, 1), port: 1975 });
    service.announce();
}
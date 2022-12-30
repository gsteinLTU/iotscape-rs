use std::{collections::BTreeMap, vec, net::{Ipv4Addr, SocketAddr, IpAddr}};

use iotscape::*;

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

    definition.methods.insert("helloWorld".to_owned(), IoTScapeMethodDescription { documentation: Some("Says \"Hello, World!\"".to_owned()), paramsList: vec![], returns: IoTScapeMethodReturns { documentation: Some("The text \"Hello, World!\"".to_owned()), r#type: vec!["string".to_owned()] } });

    let mut service: IoTScapeService = IoTScapeService::new(definition, SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1975));
    service.announce();
}
use std::{collections::BTreeMap, vec, net::{Ipv4Addr, SocketAddr, IpAddr}, time::{Duration, Instant}};

use iotscape::*;

fn main() {
    // Create definition struct
    let mut definition = IoTScapeServiceDefinition { 
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

    // Define methods
    definition.methods.insert("helloWorld".to_owned(), 
    IoTScapeMethodDescription { 
        documentation: Some("Says \"Hello, World!\"".to_owned()), 
        params: vec![], 
        returns: IoTScapeMethodReturns { 
            documentation: Some("The text \"Hello, World!\"".to_owned()), 
            r#type: vec!["string".to_owned()] 
        } 
    });
    definition.methods.insert("add".to_owned(), 
        IoTScapeMethodDescription { 
            documentation: Some("Adds two numbers".to_owned()), 
            params: vec![
                IoTScapeMethodParam { name: "a".to_owned(), documentation: Some("First number".to_owned()), r#type: "number".to_owned(), optional: false },
                IoTScapeMethodParam { name: "b".to_owned(), documentation: Some("Second number".to_owned()), r#type: "number".to_owned(), optional: false }
            ], 
            returns: IoTScapeMethodReturns { 
                documentation: Some("The sum of a and b".to_owned()), 
                r#type: vec!["number".to_owned()] 
            } 
        });

    let mut service: IoTScapeService = IoTScapeService::new(
        "ExampleService", 
        definition, 
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1975));
    
    service.announce().expect("Could not announce to server");
    
    let mut last_announce = Instant::now();
    let announce_period = Duration::from_secs(60);

    loop {
        service.poll(Some(Duration::from_secs(1)));
        
        // Re-announce to server regularly
        if Instant::now() - last_announce > announce_period {
            service.announce().expect("Could not announce to server");
            last_announce = Instant::now();
        }

        // Handle requests
        while service.rx_queue.len() > 0 {
            let next_msg = service.rx_queue.pop_front().unwrap();

            println!("Handling message {:?}", next_msg);

            // Request handlers
            match next_msg.function.as_str() {
                "helloWorld" => {
                    service.enqueue_response_to(next_msg, Ok(vec!["Hello, World!".to_owned()]));
                }
                "add" => {
                    let result: f64 = next_msg.params.iter().map(|v| v.as_f64().unwrap_or_default()).sum();
                    service.enqueue_response_to(next_msg, Ok(vec![result.to_string()]));
                }
                _ => {}
            }
        }
    }
}
use std::{collections::BTreeMap, vec, net::{Ipv4Addr, SocketAddr, IpAddr}, time::{Duration, Instant}};

use iotscape::*;

fn main() {
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

    definition.methods.insert("helloWorld".to_owned(), IoTScapeMethodDescription { documentation: Some("Says \"Hello, World!\"".to_owned()), params: vec![], returns: IoTScapeMethodReturns { documentation: Some("The text \"Hello, World!\"".to_owned()), r#type: vec!["string".to_owned()] } });

    let mut service: IoTScapeService = IoTScapeService::new("ExampleService", definition, SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1975));
    
    service.announce().expect("Could not announce to server");
    
    let mut last_announce = Instant::now();
    let announce_period = Duration::from_secs(60);

    loop {
        service.poll(Some(Duration::from_secs(1)));
        
        if Instant::now() - last_announce > announce_period {
            service.announce().expect("Could not announce to server");
            last_announce = Instant::now();
        }

        // Handle requests
        while service.rx_queue.len() > 0 {
            let next_msg = service.rx_queue.pop_front().unwrap();

            println!("Handling message {:?}", next_msg);

            match next_msg.function.as_str() {
                "helloWorld" => {
                    service.enqueue_response_to(next_msg, Ok(vec!["Hello, World!".to_owned()]));
                }
                _ => {}
            }
        }
    }
}
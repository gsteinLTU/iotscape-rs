use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
    vec,
};

use iotscape::*;

#[tokio::main]
async fn main() {
    // Create definition struct
    let mut definition = ServiceDefinition {
        id: "rs1".to_owned(),
        methods: BTreeMap::new(),
        events: BTreeMap::new(),
        description: IoTScapeServiceDescription {
            description: Some("Test IoTScape service.".to_owned()),
            externalDocumentation: None,
            termsOfService: None,
            contact: Some("gstein@ltu.edu".to_owned()),
            license: None,
            version: "1".to_owned(),
        },
    };

    // Define methods
    definition.methods.insert(
        "helloWorld".to_owned(),
        MethodDescription {
            documentation: Some("Says \"Hello, World!\"".to_owned()),
            params: vec![],
            returns: MethodReturns {
                documentation: Some("The text \"Hello, World!\"".to_owned()),
                r#type: vec!["string".to_owned()],
            },
        },
    );
    definition.methods.insert(
        "add".to_owned(),
        MethodDescription {
            documentation: Some("Adds two numbers".to_owned()),
            params: vec![
                MethodParam {
                    name: "a".to_owned(),
                    documentation: Some("First number".to_owned()),
                    r#type: "number".to_owned(),
                    optional: false,
                },
                MethodParam {
                    name: "b".to_owned(),
                    documentation: Some("Second number".to_owned()),
                    r#type: "number".to_owned(),
                    optional: false,
                },
            ],
            returns: MethodReturns {
                documentation: Some("The sum of a and b".to_owned()),
                r#type: vec!["number".to_owned()],
            },
        },
    );
    definition.methods.insert(
        "timer".to_owned(),
        MethodDescription {
            documentation: Some("Sends timer event on a delay".to_owned()),
            params: vec![MethodParam {
                name: "msec".to_owned(),
                documentation: Some("Amount of time to wait, in ms".to_owned()),
                r#type: "number".to_owned(),
                optional: false,
            }],
            returns: MethodReturns {
                documentation: Some("Response after delay".to_owned()),
                r#type: vec!["event timer".to_owned()],
            },
        },
    );
    definition.methods.insert(
        "returnComplex".to_owned(),
        MethodDescription {
            documentation: Some("Complex response to method".to_owned()),
            params: vec![],
            returns: MethodReturns {
                documentation: Some("Complex object".to_owned()),
                r#type: vec!["string".to_owned(), "string".to_owned()],
            },
        },
    );

    definition.events.insert(
        "timer".to_owned(),
        EventDescription { params: vec![] },
    );
    
    //let server = std::env::var("IOTSCAPE_SERVER").unwrap_or("52.73.65.98:1975".to_string());
    let server = std::env::var("IOTSCAPE_SERVER").unwrap_or("127.0.0.1:1978".to_string());
    let service: Arc<Mutex<IoTScapeService>> = Arc::from(Mutex::new(IoTScapeService::new(
        "ExampleService",
        definition,
        server.parse().unwrap(),
    )));

    service
        .lock()
        .unwrap()
        .announce()
        .expect("Could not announce to server");

    let mut last_announce = Instant::now();
    let announce_period = Duration::from_secs(30);

    let service_clone = Arc::clone(&service);

    tokio::task::spawn(async move {
        let service = service_clone;
        loop {
            tokio::time::sleep(Duration::from_millis(1)).await;
            service.lock().unwrap().poll(Some(Duration::from_millis(1)));

            // Re-announce to server regularly
            if last_announce.elapsed() > announce_period {
                service
                    .lock()
                    .unwrap()
                    .announce()
                    .expect("Could not announce to server");
                last_announce = Instant::now();
            }

            // Handle requests
            loop {
                if service.lock().unwrap().rx_queue.len() == 0 {
                    break;
                }

                let next_msg = service.lock().unwrap().rx_queue.pop_front().unwrap();

                println!("Handling message {:?}", next_msg);

                // Request handlers
                match next_msg.function.as_str() {
                    "helloWorld" => {
                        service
                            .lock()
                            .unwrap()
                            .enqueue_response_to(next_msg, Ok(vec!["Hello, World!".to_owned().into()])).unwrap();
                    },
                    "add" => {
                        let result: f64 = next_msg
                            .params
                            .iter()
                            .map(|v| v.as_f64().unwrap_or_default())
                            .sum();
                        
                        service
                            .lock()
                            .unwrap()
                            .enqueue_response_to(next_msg, Ok(vec![result.to_string().into()])).unwrap();
                    },
                    "timer" => {
                        let ms = next_msg
                            .params
                            .get(0).and_then(|x| x.as_u64())
                            .unwrap_or(0);
                        tokio::spawn(delayed_event(
                            Arc::clone(&service),
                            ms,
                            next_msg.id,
                            "timer",
                            BTreeMap::new(),
                        ));
                    },
                    "returnComplex" => {
                        service
                            .lock()
                            .unwrap()
                            .enqueue_response_to(next_msg, Ok(vec![vec![Into::<serde_json::Value>::into("test"), vec![1, 2, 3].into()].into()])).unwrap();                    
                    },
                    "_requestedKey" => {
                        println!("Received key: {:?}", next_msg.params);
                        service
                            .lock()
                            .unwrap()
                            .enqueue_response_to(next_msg, Ok(vec![])).unwrap();      
                    },
                    t => {
                        println!("Unrecognized function {}", t);
                    }
                }
            }
        }
    });

    loop {
        tokio::time::sleep(Duration::from_millis(1)).await;

        // Get console input
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        // Parse input
        let mut parts = input.split_whitespace();
        let command = parts.next().unwrap_or_default();
        let args = parts.collect::<Vec<&str>>();
        
        match command {
            "getkey" => {
                let mut s = service.lock().unwrap();
                let next_msg_id = s.next_msg_id.to_string();
                s.send_event(&next_msg_id, "_requestKey", BTreeMap::default()).unwrap();
                s.next_msg_id += 1;                
            },
            "reset" => {
                let mut s = service.lock().unwrap();
                let next_msg_id = s.next_msg_id.to_string();
                s.send_event(&next_msg_id, "_reset", BTreeMap::default()).unwrap();
                s.next_msg_id += 1;
            },
            "help" => {
                println!("Commands:");
                println!("  getkey - request a key from the server");
                println!("  reset - reset the encryption settings on the server");
                println!("  quit - exit the program");
            },
            "quit" => {
                break;
            },
            _ => {
                println!("Unrecognized command {}", command);
            }
        }
    }
}

async fn delayed_event(
    service: Arc<Mutex<IoTScapeService>>,
    delay: u64,
    call_id: String,
    event_type: &str,
    args: BTreeMap<String, String>,
) {
    tokio::time::sleep(Duration::from_millis(delay)).await;
    service
        .lock()
        .unwrap()
        .send_event(call_id.as_str(), event_type, args).unwrap();
}

use std::{
    collections::BTreeMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
    vec,
};

use iotscape::*;

#[tokio::main]
async fn main() {
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
            license: None,
            version: "1".to_owned(),
        },
    };

    // Define methods
    definition.methods.insert(
        "helloWorld".to_owned(),
        IoTScapeMethodDescription {
            documentation: Some("Says \"Hello, World!\"".to_owned()),
            params: vec![],
            returns: IoTScapeMethodReturns {
                documentation: Some("The text \"Hello, World!\"".to_owned()),
                r#type: vec!["string".to_owned()],
            },
        },
    );
    definition.methods.insert(
        "add".to_owned(),
        IoTScapeMethodDescription {
            documentation: Some("Adds two numbers".to_owned()),
            params: vec![
                IoTScapeMethodParam {
                    name: "a".to_owned(),
                    documentation: Some("First number".to_owned()),
                    r#type: "number".to_owned(),
                    optional: false,
                },
                IoTScapeMethodParam {
                    name: "b".to_owned(),
                    documentation: Some("Second number".to_owned()),
                    r#type: "number".to_owned(),
                    optional: false,
                },
            ],
            returns: IoTScapeMethodReturns {
                documentation: Some("The sum of a and b".to_owned()),
                r#type: vec!["number".to_owned()],
            },
        },
    );
    definition.methods.insert(
        "timer".to_owned(),
        IoTScapeMethodDescription {
            documentation: Some("Sends timer event on a delay".to_owned()),
            params: vec![IoTScapeMethodParam {
                name: "msec".to_owned(),
                documentation: Some("Amount of time to wait, in ms".to_owned()),
                r#type: "number".to_owned(),
                optional: false,
            }],
            returns: IoTScapeMethodReturns {
                documentation: Some("Response after delay".to_owned()),
                r#type: vec!["event timer".to_owned()],
            },
        },
    );

    definition.events.insert(
        "timer".to_owned(),
        IoTScapeEventDescription { params: vec![] },
    );

    let service: Arc<Mutex<IoTScapeService>> = Arc::from(Mutex::new(IoTScapeService::new(
        "ExampleService",
        definition,
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1975),
    )));

    service
        .lock()
        .unwrap()
        .announce()
        .expect("Could not announce to server");

    let mut last_announce = Instant::now();
    let announce_period = Duration::from_secs(60);

    loop {
        service.lock().unwrap().poll(Some(Duration::from_millis(1)));

        // Re-announce to server regularly
        if Instant::now() - last_announce > announce_period {
            service
                .lock()
                .unwrap()
                .announce()
                .expect("Could not announce to server");
            last_announce = Instant::now();
        }

        // Handle requests
        while service.lock().unwrap().rx_queue.len() > 0 {
            let next_msg = service.lock().unwrap().rx_queue.pop_front().unwrap();

            println!("Handling message {:?}", next_msg);

            // Request handlers
            match next_msg.function.as_str() {
                "helloWorld" => {
                    service
                        .lock()
                        .unwrap()
                        .enqueue_response_to(next_msg, Ok(vec!["Hello, World!".to_owned()]));
                }
                "add" => {
                    let result: f64 = next_msg
                        .params
                        .iter()
                        .map(|v| v.as_f64().unwrap_or_default())
                        .sum();
                    service
                        .lock()
                        .unwrap()
                        .enqueue_response_to(next_msg, Ok(vec![result.to_string()]));
                }
                "timer" => {
                    let ms = next_msg
                        .params
                        .get(0)
                        .unwrap_or(&serde_json::Value::Number(
                            serde_json::Number::from_f64(0.0).unwrap(),
                        ))
                        .as_u64()
                        .unwrap();
                    tokio::spawn(delayed_event(
                        Arc::clone(&service),
                        ms,
                        next_msg.id,
                        "timer",
                        BTreeMap::new(),
                    ));
                }
                _ => {}
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
        .send_event(call_id.as_str(), event_type, args);
}

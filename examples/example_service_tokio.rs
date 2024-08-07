use std::sync::LazyLock;
#[cfg(feature = "tokio")]
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
    vec,
};
#[cfg(feature = "tokio")]
use std::str::FromStr;

#[cfg(feature = "tokio")]
use iotscape::*;
#[cfg(feature = "tokio")]
use log::info;
#[cfg(feature = "tokio")]
use tokio::spawn;

//static SERVER: LazyLock<String> = LazyLock::new(|| std::env::var("IOTSCAPE_SERVER").unwrap_or("52.73.65.98:1978".to_string()));
static SERVER: LazyLock<String> = LazyLock::new(|| std::env::var("IOTSCAPE_SERVER").unwrap_or("127.0.0.1:1978".to_string()));
//static ANNOUNCE_ENDPOINT: LazyLock<String> = LazyLock::new(|| std::env::var("IOTSCAPE_ANNOUNCE_ENDPOINT").unwrap_or("https://services.netsblox.org/routes/iotscape/announce".to_string()));
static ANNOUNCE_ENDPOINT: LazyLock<String> = LazyLock::new(|| std::env::var("IOTSCAPE_ANNOUNCE_ENDPOINT").unwrap_or("http://localhost:8080/routes/iotscape/announce".to_string()));
// static RESPONSE_ENDPOINT: LazyLock<String> = LazyLock::new(|| std::env::var("IOTSCAPE_RESPONSE_ENDPOINT").unwrap_or("http://services.netsblox.org/routes/iotscape/response".to_string()));
static RESPONSE_ENDPOINT: LazyLock<String> = LazyLock::new(|| std::env::var("IOTSCAPE_RESPONSE_ENDPOINT").unwrap_or("http://localhost:8080/routes/iotscape/response".to_string()));

#[cfg(feature = "tokio")]
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

    let service: Arc<IoTScapeServiceAsync> = Arc::from(IoTScapeServiceAsync::new(
        "ExampleService",
        definition,
        SERVER.parse().unwrap(),
    ).await);

    service
        .announce()
        .await
        .expect("Could not announce to server");

    let mut last_announce = Instant::now();
    let announce_period = Duration::from_secs(30);

    let service_clone = service.clone();

    tokio::task::spawn(async move {
        let service = service_clone;
        loop {
            tokio::time::sleep(Duration::from_millis(10)).await;
            service.poll().await;

            // Re-announce to server regularly
            if last_announce.elapsed() > announce_period {
                service
                    .announce()
                    .await
                    .expect("Could not announce to server");
                last_announce = Instant::now();
            }

            // Handle requests
            while let Some(next_msg) = service.rx_queue.lock().unwrap().pop_front() {
                println!("Handling message {:?}", next_msg);

                let service = service.clone();
                spawn(async move { 
                    // Request handlers
                    match next_msg.function.as_str() {
                        "helloWorld" => {
                                service.enqueue_response_to(next_msg, Ok(vec!["Hello, World!".to_owned().into()])).await.expect("Could not enqueue response");
                        },
                        "add" => {
                            let result: f64 = next_msg
                                .params
                                .iter()
                                .map(|v| 
                                    match v {
                                        serde_json::Value::Number(n) => n.as_f64().unwrap_or_default(),
                                        serde_json::Value::String(s) => f64::from_str(&s).unwrap_or_default(),
                                        _ => 0.0,
                                    })
                                .sum(); 
                                service
                                    .enqueue_response_to_http(&RESPONSE_ENDPOINT, next_msg, Ok(vec![result.to_string().into()])).await.expect("Could not enqueue response");
                        },
                        "timer" => {
                            info!("Received timer request {:?}", next_msg);
                            let ms = next_msg
                                .params
                                .get(0).and_then(|x| u64::from_str_radix(&x.to_string(), 10).ok())
                                .unwrap_or(0);
                            spawn(delayed_event(
                                service.clone(),
                                ms,
                                next_msg.id.clone(),
                                "timer",
                                BTreeMap::new(),
                            ));
                            service
                                .enqueue_response_to(next_msg, Ok(vec![])).await.expect("Could not enqueue response");    
                        },
                        "returnComplex" => {
                            // Load image
                            let image = std::fs::read("examples/figure.png").expect("Could not read image file");
                            let image = "<costume  name=\"costume\" collabId=\"\" center-x=\"43.5\" center-y=\"62\" image=\"data:image/png;base64,".to_string() + base64::encode(&image).as_str() + "\"/>";
                            service
                                .enqueue_response_to_http(&RESPONSE_ENDPOINT, next_msg, Ok(vec![vec![Into::<serde_json::Value>::into("test"), vec![1, 2, 3].into(), vec![image].into()].into()])).await.expect("Could not enqueue response");
                        },
                        "_requestedKey" => {
                            println!("Received key: {:?}", next_msg.params);
                            service
                                .enqueue_response_to(next_msg, Ok(vec![])).await.expect("Could not enqueue response");      
                        },
                        t => {
                            println!("Unrecognized function {}", t);
                        }
                    }
                });
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
        let _args = parts.collect::<Vec<&str>>();
        
        match command {
            "getkey" => {
                let next_msg_id = service.next_msg_id.load(std::sync::atomic::Ordering::Relaxed).to_string();
                service.send_event(&next_msg_id, "_requestKey", BTreeMap::default()).await.expect("Could not send event");
            },
            "reset" => {
                let next_msg_id = service.next_msg_id.load(std::sync::atomic::Ordering::Relaxed).to_string();
                service.send_event(&next_msg_id, "_reset", BTreeMap::default()).await.expect("Could not send event");
            },
            "announce" => {
                service.announce().await.expect("Could not announce to server");
            },
            "announcehttp" => {
                service.announce_http(&ANNOUNCE_ENDPOINT).await.expect("Could not announce to server");
            },
            "help" => {
                println!("Commands:");
                println!("  announce - send a new announce to the server");
                println!("  announcehttp - send a new announce to the server over HTTP");
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

#[cfg(feature = "tokio")]
async fn delayed_event(
    service: Arc<IoTScapeServiceAsyncUdp>,
    delay: u64,
    call_id: String,
    event_type: &str,
    args: BTreeMap<String, String>,
) {
    tokio::time::sleep(Duration::from_millis(delay)).await;
    println!("Sending event {} with args {:?} after {} ms", event_type, args, delay);
    service.clone()
        .send_event(call_id.as_str(), event_type, args).await.expect("Could not send event");
}

#[cfg(not(feature = "tokio"))]
fn main() {
    panic!("This example requires the 'tokio' feature to be enabled.");
}
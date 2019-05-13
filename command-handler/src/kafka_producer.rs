use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{thread, env};

use log::{info, warn};
use avro_rs::types::Value;
use futures::Future;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{DeliveryFuture, FutureProducer, FutureRecord};
use schema_registry_converter::schema_registry::SubjectNameStrategy;
use schema_registry_converter::Encoder;

pub struct RecordProducer {
    producer: FutureProducer,
    encoder: Encoder,
    sender: Sender<DeliveryFuture>,
}

impl RecordProducer {
    pub fn send(
        &mut self,
        topic: &str,
        key: Option<&str>,
        values: Vec<(&'static str, Value)>,
        strategy: &SubjectNameStrategy
    ) {
        let payload = match self.encoder.encode(values, &strategy) {
            Ok(v) => v,
            Err(e) => panic!("Error getting payload: {}", e),
        };
        let fr = FutureRecord {
            topic,
            partition: None,
            payload: Some(&payload),
            key,
            timestamp: None,
            headers: None,
        };
        match self.sender.send(self.producer.send(fr, 0)) {
            Ok(_) => info!("successfully send message"),
            Err(e) => panic!("Could not send record: {}", e),
        };
    }
}

pub fn get_producer() -> RecordProducer {
    let brokers = match env::var("KAFKA_BROKERS") {
        Ok(val) => val,
        Err(_e) => "127.0.0.1:9092".to_string(),
    };
    let schema_registry_url = match env::var("SCHEMA_REGISTRY_URL") {
        Ok(val) => val,
        Err(_e) => "http://localhost:8081".to_string(),
    };
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", brokers.as_str())
        .set("linger.ms", "100")
        .create()
        .expect("Producer creation error");
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || finish_loop(&receiver));
    let encoder = Encoder::new(schema_registry_url);
    RecordProducer {
        producer,
        encoder,
        sender,
    }
}

fn finish_loop(receiver: &Receiver<DeliveryFuture>) {
    loop {
        let df = match receiver.recv() {
            Ok(v) => v,
            Err(e) => panic!("Error reading future from receiver: {}", e),
        };
        let result = match df.wait() {
            Ok(v) => v,
            Err(e) => panic!("Error waiting for future: {}", e),
        };
        match result {
            Ok(v) => info!("Successfully stored message in partition {} at offset {}", v.0, v.1),
            Err(e) => warn!("Error storing message: {:?}, with error: {}", e.1, e.0),
        };
    }
}

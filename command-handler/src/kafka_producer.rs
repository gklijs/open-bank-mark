use log::info;
use avro_rs::types::Value;
use schema_registry_converter::schema_registry::SubjectNameStrategy;
use schema_registry_converter::Encoder;
use kafka::producer::{Producer, Record};
use std::time::Duration;
use kafka::client::RequiredAcks;
use std::env;

pub struct AvroProducer {
    producer: Producer,
    encoder: Encoder,
}

impl AvroProducer {
    pub fn send(
        &mut self,
        topic: &str,
        key: String,
        values: Vec<(&'static str, Value)>,
        strategy: &SubjectNameStrategy
    ) {
        let value = match self.encoder.encode(values, &strategy) {
            Ok(v) => v,
            Err(e) => panic!("Error getting payload: {}", e),
        };
        let record = Record::from_key_value(topic, key, value);
        match self.producer.send(&record){
            Ok(v) => info!("successfully send message {:#?}", v),
            Err(e) => panic!("Error sending message: {}", e),
        }
    }
}

pub fn get_producer() -> AvroProducer {
    let brokers = match env::var("KAFKA_BROKERS") {
        Ok(val) => val.split(',').map(String::from).collect(),
        Err(_e) => vec!("127.0.0.1:9092".to_string()),
    };
    let schema_registry_url = match env::var("SCHEMA_REGISTRY_URL") {
        Ok(val) => val,
        Err(_e) => "http://localhost:8081".to_string(),
    };
    let producer = match Producer::from_hosts(brokers)
        .with_ack_timeout(Duration::from_secs(1))
        .with_required_acks(RequiredAcks::All)
        .create(){
        Ok(p) => p,
        Err(e) => panic!("Error creating producer: {}", e),
    };
    let encoder = Encoder::new(schema_registry_url);
    AvroProducer{
        producer,
        encoder
    }
}

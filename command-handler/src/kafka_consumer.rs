use log::{info, warn};
use futures::stream::Stream;
use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{Consumer, ConsumerContext, Rebalance};
use rdkafka::error::KafkaResult;
use rdkafka::message::Message;
use schema_registry_converter::Decoder;
use avro_rs::types::Value;
use std::{env, thread};
use std::thread::JoinHandle;

// A context can be used to change the behavior of producers and consumers by adding callbacks
// that will be executed by librdkafka.
// This particular context sets up custom callbacks to log rebalancing events.
pub struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        info!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        info!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(
        &self,
        _result: KafkaResult<()>,
        _offsets: *mut rdkafka_sys::RDKafkaTopicPartitionList,
    ) {
        info!("Storing processed offsets");
    }
}

type ProcessingConsumer = StreamConsumer<CustomContext>;

pub trait ValuesProcessor {
    fn process(&mut self, values: &[(String, Value)]);
}

pub fn consume(
    group_id: &'static str,
    topic: &'static str,
    mut values_processor: Box<ValuesProcessor + Send>,
) -> JoinHandle<()> {
    let brokers = match env::var("KAFKA_BROKERS") {
        Ok(val) => val,
        Err(_e) => "127.0.0.1:9092".to_string(),
    };
    let schema_registry_url = match env::var("SCHEMA_REGISTRY_URL") {
        Ok(val) => val,
        Err(_e) => "http://localhost:8081".to_string(),
    };

    thread::spawn(move || {
        let consumer = get_consumer(brokers.as_str(), group_id, topic);
        let message_stream = consumer.start();
        let mut decoder = Decoder::new(schema_registry_url);
        for message in message_stream.wait() {
            match message {
                Err(e) => warn!("Error while reading from stream: {:#?}", e),
                Ok(Ok(m)) => {
                    match decoder.decode(m.payload()) {
                        Ok(v) => {
                            match v {
                                Value::Record(v) => values_processor.as_mut().process(&v),
                                _ => panic!("Not a record, while only those expected"),
                            }
                        }
                        Err(e) => warn!("Error decoding value of record with error: {:?}", e),
                    }
                    consumer.store_offset(&m).unwrap();
                }
                Ok(Err(e)) => warn!("Kafka error: {}", e),
            };
        }
    })
}

fn get_consumer(brokers: &str, group_id: &str, topic: &str) -> ProcessingConsumer {
    let context = CustomContext;
    let consumer: ProcessingConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.auto.commit", "true")
        .set("enable.auto.offset.store", "false")
        .set("statistics.interval.ms", "0")
        .set("fetch.error.backoff.ms", "1")
        .set("auto.offset.reset", "earliest")
        .set_log_level(RDKafkaLogLevel::Info)
        .create_with_context(context)
        .expect("Consumer creation failed");
    consumer
        .subscribe(&[topic])
        .expect("Can't subscribe to specified topics");
    consumer
}

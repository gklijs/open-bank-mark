use log::{info, warn};
use schema_registry_converter::Decoder;
use avro_rs::types::Value;
use std::{env, thread};
use std::thread::JoinHandle;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};

pub trait ValuesProcessor {
    fn process(&mut self, values: &[(String, Value)]);
}

pub fn consume(
    group_id: &'static str,
    topic: &'static str,
    mut values_processor: Box<ValuesProcessor + Send>,
) -> JoinHandle<()> {
    let brokers = match env::var("KAFKA_BROKERS") {
        Ok(val) => val.split(',').map(String::from).collect(),
        Err(_e) => vec!("127.0.0.1:9092".to_string()),
    };
    let schema_registry_url = match env::var("SCHEMA_REGISTRY_URL") {
        Ok(val) => val,
        Err(_e) => "http://localhost:8081".to_string(),
    };
    thread::spawn(move || {
        let mut consumer = get_consumer(brokers, group_id, topic);
        let mut decoder = Decoder::new(schema_registry_url);
        loop{
            let mss = match consumer.poll(){
                Ok(v) => v,
                Err(e) => panic!("Quit because of problem doing consumer poll {}", e)
            };
            if mss.is_empty() {
                info!("No messages available right now.");
            };
            for ms in mss.iter() {
                for m in ms.messages() {
                    info!("{}:{}@{}: {:?}", ms.topic(), ms.partition(), m.offset, m.value);
                    match decoder.decode(Some(m.value)) {
                        Ok(v) => {
                            match v {
                                Value::Record(v) => values_processor.as_mut().process(&v),
                                _ => panic!("Not a record, while only those expected"),
                            }
                        }
                        Err(e) => warn!("Error decoding value of record with error: {:?}", e),
                    }
                }
                match consumer.consume_messageset(ms){
                    Ok(v) => info!("Successfully stored offets internally {:#?}", v),
                    Err(e) => panic!("Quit because of problem storing offsets {}", e)
                }
            }
            match consumer.commit_consumed(){
                Ok(v) => info!("Consumer offset successful committed {:#?}", v),
                Err(e) => panic!("Quit because of problem committing consumer offsets {}", e)
            };
        }
    })
}

fn get_consumer(brokers: Vec<String>, group: &str, topic: &str) -> Consumer {
    match Consumer::from_hosts(brokers)
        .with_topic(topic.to_string())
        .with_group(group.to_string())
        .with_fallback_offset(FetchOffset::Earliest)
        .with_offset_storage(GroupOffsetStorage::Kafka)
        .create(){
        Ok(c) => c,
        Err(e) => panic!("Error creating consumer {}", e)
    }
}

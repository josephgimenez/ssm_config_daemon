use {
    crate::config::{Config, Parameter, Service},
    rusoto_core::Region,
    rusoto_kinesis::{KinesisClient, Kinesis, DescribeStreamInput, GetShardIteratorInput, GetRecordsInput},
    std::io::{Error, ErrorKind},
    serde_json::Value,
    std::{str, thread::sleep, time::Duration },
};
use core::borrow::Borrow;

pub struct KinesisConsumerClient {
    client: KinesisClient,
    config: Config,
    service_parameters: Vec<Service>,
    shard_iterator: String,
    stream_name: String,
}

impl KinesisConsumerClient {
    pub fn new(config: Config, service_parameters: Vec<Service>) -> KinesisConsumerClient {
        KinesisConsumerClient {
            client: KinesisClient::new(Region::UsEast1),
            config,
            service_parameters,
            stream_name: "parameter-store-template".to_owned(),
            shard_iterator: String::new(),
        }
    }

    fn get_shard_id(&mut self) -> Result<(), Box<std::error::Error>> {
        let describe_output = self.client.describe_stream(
            DescribeStreamInput {
                stream_name: self.stream_name.clone(),
                ..Default::default()
            }
        ).sync()?;

        let shard_id = describe_output.stream_description.shards[0].shard_id.clone();

        let shard_iterator_output = self.client.get_shard_iterator(
            GetShardIteratorInput {
                stream_name: self.stream_name.clone(),
                shard_id,
                shard_iterator_type: "LATEST".to_owned(),
                ..Default::default()
            }
        ).sync()?;

        self.shard_iterator = shard_iterator_output.shard_iterator.unwrap();


        Ok(())
    }

    pub fn start_event_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        self.get_shard_id()?;

        loop {
            let records_output = self.client.get_records( GetRecordsInput{
                limit: Some(10),
                shard_iterator: self.shard_iterator.clone()
            }).sync()?;

            for record in &records_output.records {
                let ps_event: Value = serde_json::from_str(str::from_utf8(record.data.as_slice())?)?;
                println!("Got update at path: {} with new value: {}", ps_event["requestParameters"]["name"], ps_event["requestParameters"]["value"]);

                for service in &mut self.service_parameters {
                    match service
                        .parameters
                        .iter_mut()
                        .find(|parameter| parameter.ps_path == ps_event["requestParameters"]["name"]) {
                        Some(parameter) => {
                            parameter.value = ps_event["requestParameters"]["value"].clone();
                            crate::config::render_config(&vec![service.clone()], &self.config)?;
                        },
                        None => continue
                    }
                }

            }

            match records_output.next_shard_iterator {
                Some(iterator) => self.shard_iterator = iterator,
                None => break
            }

            sleep(Duration::from_secs(5))
        }

        Ok(())
    }
}







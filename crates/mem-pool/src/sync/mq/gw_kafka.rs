use anyhow::Result;
use gw_types::{
    bytes::Bytes,
    packed::{RefreshMemBlockMessage, RefreshMemBlockMessageUnion},
    prelude::{Builder, Entity, Reader},
};
use rdkafka::{
    consumer::{BaseConsumer, CommitMode, Consumer as RdConsumer},
    message::ToBytes,
    producer::{BaseProducer, BaseRecord},
    ClientConfig, Message,
};

use crate::sync::subscribe::SubscribeMemPoolService;

use super::{Consume, Produce};
pub(crate) struct Producer {
    producer: rdkafka::producer::BaseProducer,
    topic: String,
}

impl Producer {
    pub(crate) fn connect(hosts: Vec<String>, topic: String) -> Result<Self> {
        let brokers = hosts.join(",");
        let producer: BaseProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("enable.idempotence", "true")
            .create()?;
        Ok(Self { producer, topic })
    }
}

impl Produce for Producer {
    type Msg = RefreshMemBlockMessageUnion;

    fn send(&mut self, message: Self::Msg) -> Result<()> {
        let msg = RefreshMemBlockMessage::new_builder().set(message).build();
        let bytes = msg.as_bytes();
        log::trace!("Producer send msg: {:?}", &bytes.to_vec());
        let message = RefreshMemBlockMessageFacade(bytes);
        if let Err((err, _)) = self
            .producer
            .send(BaseRecord::to(&self.topic).key("").payload(&message))
        {
            log::error!("[kafka] send message failed: {:?}", &err);
        }
        Ok(())
    }
}

pub(crate) struct Consumer {
    consumer: BaseConsumer,
    subscribe: SubscribeMemPoolService,
    topic: String,
}

impl Consumer {
    pub(crate) fn start(
        hosts: Vec<String>,
        topic: String,
        group: String,
        subscribe: SubscribeMemPoolService,
    ) -> Result<Self> {
        let brokers = hosts.join(",");
        let consumer: BaseConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .set("group.id", group)
            .create()?;
        Ok(Self {
            consumer,
            topic,
            subscribe,
        })
    }
}

impl Consume for Consumer {
    fn poll(&mut self) -> Result<()> {
        self.consumer.subscribe(&[&self.topic])?;
        for message in self.consumer.iter() {
            match message {
                Ok(msg) => {
                    let topic = msg.topic();
                    let partition = msg.partition();
                    let offset = msg.offset();
                    let payload = msg.payload();
                    log::trace!(
                        "Recv kafka msg: {}:{}@{}: {:?}",
                        topic,
                        partition,
                        offset,
                        &payload
                    );

                    if let Some(payload) = payload {
                        let refresh_msg = RefreshMemBlockMessage::from_slice(payload)?;
                        let reader = refresh_msg.as_reader();
                        let refresh_msg = reader.to_enum();
                        match &refresh_msg {
                            gw_types::packed::RefreshMemBlockMessageUnionReader::NextL2Transaction(
                                next,
                            ) => {
                                if let Err(err) = self.subscribe.next_tx(next.to_entity()) {
                                    log::error!("[Subscribe tx] error: {:?}", err);
                                }
                            }
                            gw_types::packed::RefreshMemBlockMessageUnionReader::NextMemBlock(
                                next,
                            ) => {
                                match self.subscribe.next_mem_block(next.to_entity()) {
                                    Ok(None) => {
                                        log::debug!(
                                            "Invalid tip. Wait for syncing to the new tip."
                                        );
                                        break;
                                    }
                                    Ok(Some(block_number)) => {
                                        log::debug!("Refresh mem pool to {}", block_number);
                                    }
                                    Err(err) => {
                                        log::error!("[Refresh mem pool] error: {:?}", err);
                                    }
                                };
                            }
                        };
                        self.consumer.commit_message(&msg, CommitMode::Async)?;
                        log::trace!("Kafka commit offset: {}", offset);
                    };
                }
                Err(err) => {
                    log::error!("Receive error from kafka: {:?}", err);
                }
            }
        }

        Ok(())
    }
}

struct RefreshMemBlockMessageFacade(Bytes);
impl ToBytes for RefreshMemBlockMessageFacade {
    fn to_bytes(&self) -> &[u8] {
        &self.0
    }
}

use std::io::StdoutLock;
use std::io::Write;

use anyhow::Context;

use anyhow::Ok;
use anyhow::Result;

pub mod msg_protocol {
    use anyhow::Result;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Message {
        pub src: Option<String>,
        pub dest: Option<String>,
        pub body: Body,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Body {
        pub msg_id: Option<i64>,
        pub in_reply_to: Option<i64>,

        #[serde(flatten)]
        pub body: MessageBodyType,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type")]
    #[serde(rename_all = "snake_case")]
    pub enum MessageBodyType {
        Init {
            node_id: String,
            node_ids: Vec<String>,
        },
        InitOk {},
        EchoOk {
            echo: String,
        },
        Echo {
            echo: String,
        },
        Generate {},
        GenerateOk {
            id: i64,
        },
    }

    pub trait Processor {
        fn process(&mut self, msg: Message) -> Result<Option<Message>>;
    }
}
pub mod runner {
    use super::*;
    use msg_protocol::*;

    fn serialize(maybe_reply: Option<Message>, out: &mut StdoutLock) -> Result<()> {
        if let Some(reply) = maybe_reply {
            serde_json::to_writer(&mut *out, &reply).context("Serialize reply message")?;
            // writing a new line to flush the line writer
            out.write_all(b"\n")?;
        }
        Ok(())
    }

    pub fn run<P: Processor>(processor: &mut P) -> anyhow::Result<()> {
        let stdin = std::io::stdin().lock();
        let deserializer = serde_json::Deserializer::from_reader(stdin);
        let mut stdout = std::io::stdout().lock();
        deserializer
            .into_iter::<Message>()
            .for_each(|msg| match msg {
                std::result::Result::Ok(msg) => {
                    let maybe_msg_result =
                        processor.process(msg).context("Error processing message");
                    if let Result::Ok(maybe_msg) = maybe_msg_result {
                        serialize(maybe_msg, &mut stdout).unwrap();
                    };
                }
                Err(e) => {
                    println!("Unknown message : {}", e);
                }
            });
        Ok(())
    }
}

pub mod shared_fixtures {
    use super::msg_protocol::*;

    pub fn init_ok_msg() -> Message {
        Message {
            src: Some("dest".to_string()),
            dest: Some("src".to_string()),
            body: Body {
                msg_id: Some(1),
                in_reply_to: Some(1),
                body: MessageBodyType::InitOk {},
            },
        }
    }
    pub fn echo_msg() -> Message {
        Message {
            src: Some("src".to_string()),
            dest: Some("dest".to_string()),
            body: Body {
                msg_id: Some(1),
                in_reply_to: None,
                body: MessageBodyType::Echo {
                    echo: "echo".to_string(),
                },
            },
        }
    }
}
#[cfg(test)]
mod tests {

    use serde_json::from_str;
    use serde_json::to_string;

    use crate::shared_fixtures;

    use super::msg_protocol::*;

    #[test]
    fn test_serde_msg_echo() {
        let msg = shared_fixtures::echo_msg();
        let msg_serialized = to_string(&msg).unwrap();
        let msg_round_trip = from_str::<Message>(&msg_serialized).unwrap();
        assert_eq!(msg, msg_round_trip);
    }
    #[test]
    fn test_serde_msg_init_ok() {
        let msg = shared_fixtures::init_ok_msg();
        let msg_serialized = to_string(&msg).unwrap();
        let msg_round_trip = from_str::<Message>(&msg_serialized).unwrap();
        assert_eq!(msg, msg_round_trip);
    }
}

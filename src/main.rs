use std::io::StdoutLock;
use std::io::Write;

use anyhow::anyhow;
use anyhow::Context;

use anyhow::Ok;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Message {
    src: Option<String>,
    dest: Option<String>,
    body: Body,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Body {
    msg_id: Option<usize>,
    in_reply_to: Option<usize>,

    #[serde(flatten)]
    body: MessageBodyType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum MessageBodyType {
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
}

struct MessageProcessor {
    id: usize,
}
impl MessageProcessor {
    fn new() -> Self {
        MessageProcessor { id: 1 }
    }
    fn process(&mut self, msg: Message) -> Result<Option<Message>> {
        match msg.body.body {
            MessageBodyType::Init {
                node_id: _,
                node_ids: _,
            } => {
                let reply = Ok(Some(Message {
                    src: msg.dest,
                    dest: msg.src,
                    body: Body {
                        msg_id: Some(self.id),
                        in_reply_to: msg.body.msg_id,
                        body: MessageBodyType::InitOk {},
                    },
                }));
                self.id += 1;
                reply
            }

            MessageBodyType::Echo { echo } => {
                let reply = Message {
                    src: msg.dest,
                    dest: msg.src,
                    body: Body {
                        msg_id: Some(self.id),
                        in_reply_to: msg.body.msg_id,
                        body: MessageBodyType::EchoOk { echo },
                    },
                };
                self.id += 1;
                Ok(Some(reply))
            }

            _ => Err(anyhow!("Received unknown message: {:?}", msg)),
        }
    }
}

fn serialize(maybe_reply: Option<Message>, out: &mut StdoutLock) -> Result<()> {
    if let Some(reply) = maybe_reply {
        serde_json::to_writer(&mut *out, &reply).context("Serialize reply message")?;
        // writing a new line to flush the line writer
        out.write_all(b"\n")?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut processor = MessageProcessor::new();
    let stdin = std::io::stdin().lock();
    let deserializer = serde_json::Deserializer::from_reader(stdin);
    let mut stdout = std::io::stdout().lock();
    deserializer
        .into_iter::<Message>()
        .for_each(|msg| match msg {
            std::result::Result::Ok(msg) => {
                let maybe_msg_result = processor.process(msg).context("Error processing message");
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

#[cfg(test)]
mod tests {

    use super::*;

    use serde_json::from_str;
    use serde_json::to_string;

    mod fixtures {
        use super::*;

        pub fn init_msg() -> Message {
            Message {
                src: Some("src".to_string()),
                dest: Some("dest".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: None,
                    body: MessageBodyType::Init {
                        node_id: "mynode1".to_string(),
                        node_ids: vec!["mynode1".to_string()],
                    },
                },
            }
        }

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
        pub fn echo_ok_msg() -> Message {
            Message {
                src: Some("dest".to_string()),
                dest: Some("src".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: Some(1),
                    body: MessageBodyType::EchoOk {
                        echo: "echo".to_string(),
                    },
                },
            }
        }
    }

    #[test]
    fn test_serde_msg_echo() {
        let msg = fixtures::echo_msg();
        let msg_serialized = to_string(&msg).unwrap();
        let msg_round_trip = from_str::<Message>(&msg_serialized).unwrap();
        assert_eq!(msg, msg_round_trip);
    }
    #[test]
    fn test_serde_msg_init_ok() {
        let msg = fixtures::init_ok_msg();
        let msg_serialized = to_string(&msg).unwrap();
        let msg_round_trip = from_str::<Message>(&msg_serialized).unwrap();
        assert_eq!(msg, msg_round_trip);
    }

    #[test]
    fn test_msg_processing_init() {
        let mut processor = MessageProcessor::new();
        let msg = fixtures::init_msg();
        let reply = processor.process(msg);
        assert_eq!(reply.unwrap(), Some(fixtures::init_ok_msg()));
    }

    #[test]
    fn test_msg_processing_echo() {
        let mut processor = MessageProcessor::new();
        let msg = fixtures::echo_msg();

        let reply = processor.process(msg);
        assert_eq!(reply.unwrap(), Some(fixtures::echo_ok_msg()));
    }

    #[test]

    fn test_msg_processing_unhandled_msg() {
        let mut processor = MessageProcessor::new();
        let msg = fixtures::echo_ok_msg();

        let expected_err_msg = format!("Received unknown message: {:?}", &msg);
        let result = processor.process(msg);
        match result {
            Err(err) => assert_eq!(format!("{}", err), expected_err_msg),
            _ => panic!("Expected error"),
        }
    }

    #[test]
    fn test_msg_processor_id_increments_on_every_msg() {
        let mut processor = MessageProcessor::new();

        fn assert_reply_to_msg(
            processor: &mut MessageProcessor,
            msg: Message,
            expected_reply: Option<Message>,
        ) {
            let maybe_reply = processor.process(msg).unwrap();
            assert_eq!(maybe_reply, expected_reply);
        }
        let base_msg = fixtures::echo_msg();
        let base_reply = fixtures::echo_ok_msg();

        let range = 1..5;
        range.for_each(|x| {
            let mut msg = base_msg.clone();
            msg.body.msg_id = Some(x);

            let mut expected_reply = base_reply.clone();
            expected_reply.body.msg_id = Some(x);
            expected_reply.body.in_reply_to = Some(x);
            assert_reply_to_msg(&mut processor, msg, Some(expected_reply))
        })
    }
}

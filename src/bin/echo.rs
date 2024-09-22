use anyhow::anyhow;
use anyhow::Ok;
use anyhow::Result;
use maelstrom_rust::msg_protocol::*;
use maelstrom_rust::runner::*;
pub struct EchoProcessor {
    pub id: i64,
}

impl EchoProcessor {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

impl Default for EchoProcessor {
    fn default() -> Self {
        Self::new(1)
    }
}
impl Processor for EchoProcessor {
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
fn main() -> anyhow::Result<()> {
    run(&mut EchoProcessor::default())
}

#[cfg(test)]
mod tests {

    use maelstrom_rust::msg_protocol::*;

    use crate::EchoProcessor;
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
    fn test_msg_processing_init() {
        let mut processor = EchoProcessor::default();
        let msg = fixtures::init_msg();
        let reply = processor.process(msg);
        assert_eq!(reply.unwrap(), Some(fixtures::init_ok_msg()));
    }

    #[test]
    fn test_msg_processing_echo() {
        let mut processor = EchoProcessor::default();
        let msg = fixtures::echo_msg();

        let reply = processor.process(msg);
        assert_eq!(reply.unwrap(), Some(fixtures::echo_ok_msg()));
    }

    #[test]

    fn test_msg_processing_unhandled_msg() {
        let mut processor: EchoProcessor = EchoProcessor::default();
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
        let mut processor = EchoProcessor::default();

        fn assert_reply_to_msg(
            processor: &mut EchoProcessor,
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

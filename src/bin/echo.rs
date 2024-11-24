use anyhow::anyhow;
use anyhow::Ok;
use anyhow::Result;
use maelstrom_rust::msg_protocol::*;
use maelstrom_rust::runner::*;
use serde::{Deserialize, Serialize};

struct EchoMaelstromNode {
    id: i64,
}

impl EchoMaelstromNode {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

impl Default for EchoMaelstromNode {
    fn default() -> Self {
        Self::new(1)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum EchoMessage {
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

impl Processor<EchoMessage> for EchoMaelstromNode {
    fn process(&mut self, msg: Message<EchoMessage>) -> Result<Option<Vec<Message<EchoMessage>>>> {
        match msg.body.body {
            EchoMessage::Init {
                node_id: _,
                node_ids: _,
            } => {
                let reply = Ok(Some(vec![Message {
                    src: msg.dest,
                    dest: msg.src,
                    body: Body {
                        msg_id: Some(self.id),
                        in_reply_to: msg.body.msg_id,
                        body: EchoMessage::InitOk {},
                    },
                }]));
                self.id += 1;
                reply
            }

            EchoMessage::Echo { echo } => {
                let reply = Message {
                    src: msg.dest,
                    dest: msg.src,
                    body: Body {
                        msg_id: Some(self.id),
                        in_reply_to: msg.body.msg_id,
                        body: EchoMessage::EchoOk { echo },
                    },
                };
                self.id += 1;
                Ok(Some(vec![reply]))
            }

            _ => Err(anyhow!("Received unknown message: {:?}", msg)),
        }
    }
}
fn main() -> anyhow::Result<()> {
    run(&mut EchoMaelstromNode::default())
}

#[cfg(test)]
mod tests {

    use crate::EchoMaelstromNode;
    use crate::EchoMessage;

    use maelstrom_rust::msg_protocol::*;

    use serde_json::from_str;
    use serde_json::to_string;
    mod fixtures {
        use super::*;

        pub fn init_msg() -> Message<EchoMessage> {
            Message {
                src: Some("src".to_string()),
                dest: Some("dest".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: None,
                    body: EchoMessage::Init {
                        node_id: "mynode1".to_string(),
                        node_ids: vec!["mynode1".to_string()],
                    },
                },
            }
        }
        pub fn init_ok_msg() -> Message<EchoMessage> {
            Message {
                src: Some("dest".to_string()),
                dest: Some("src".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: Some(1),
                    body: EchoMessage::InitOk {},
                },
            }
        }
        pub fn echo_msg() -> Message<EchoMessage> {
            Message {
                src: Some("src".to_string()),
                dest: Some("dest".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: None,
                    body: EchoMessage::Echo {
                        echo: "echo".to_string(),
                    },
                },
            }
        }
        pub fn echo_ok_msg() -> Message<EchoMessage> {
            Message {
                src: Some("dest".to_string()),
                dest: Some("src".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: Some(1),
                    body: EchoMessage::EchoOk {
                        echo: "echo".to_string(),
                    },
                },
            }
        }
    }
    #[test]
    fn test_msg_processing_init() {
        let mut processor = EchoMaelstromNode::default();
        let msg = fixtures::init_msg();
        let reply = processor.process(msg);
        assert_eq!(reply.unwrap(), Some(vec![fixtures::init_ok_msg()]));
    }

    #[test]
    fn test_msg_processing_echo() {
        let mut processor = EchoMaelstromNode::default();
        let msg = fixtures::echo_msg();

        let reply = processor.process(msg);
        assert_eq!(reply.unwrap(), Some(vec![fixtures::echo_ok_msg()]));
    }

    #[test]
    fn test_msg_processing_unhandled_msg() {
        let mut processor: EchoMaelstromNode = EchoMaelstromNode::default();
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
        let mut processor = EchoMaelstromNode::default();

        fn assert_reply_to_msg(
            processor: &mut EchoMaelstromNode,
            msg: Message<EchoMessage>,
            expected_reply: Option<Vec<Message<EchoMessage>>>,
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
            assert_reply_to_msg(&mut processor, msg, Some(vec![expected_reply]))
        })
    }
    #[test]
    fn test_serde_msg_echo() {
        let msg = fixtures::echo_msg();
        let msg_serialized = to_string(&msg).unwrap();
        let msg_round_trip = from_str::<Message<EchoMessage>>(&msg_serialized).unwrap();
        assert_eq!(msg, msg_round_trip);
    }
    #[test]
    fn test_serde_msg_init_ok() {
        let msg = fixtures::init_ok_msg();
        let msg_serialized = to_string(&msg).unwrap();
        let msg_round_trip = from_str::<Message<EchoMessage>>(&msg_serialized).unwrap();
        assert_eq!(msg, msg_round_trip);
    }
}

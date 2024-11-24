use anyhow::anyhow;
use anyhow::Result;
use maelstrom_rust::msg_protocol::*;
use maelstrom_rust::runner::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

trait IdGenerator {
    fn generate(&self) -> Uuid;
}
struct DefaultIdGenerator;
impl IdGenerator for DefaultIdGenerator {
    fn generate(&self) -> Uuid {
        Uuid::new_v4()
    }
}
struct UniqueIdGeneratorMaelstromNode {
    id: i64,
    uuid_generator: Box<dyn IdGenerator>,
}

impl UniqueIdGeneratorMaelstromNode {
    pub fn new(id: i64, uuid_generator: Box<dyn IdGenerator>) -> Self {
        Self { id, uuid_generator }
    }
}

impl Default for UniqueIdGeneratorMaelstromNode {
    fn default() -> Self {
        Self::new(1, Box::new(DefaultIdGenerator))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum UniqueIdMessage {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {},
    Generate {},
    GenerateOk {
        id: Uuid,
    },
}

impl Processor<UniqueIdMessage> for UniqueIdGeneratorMaelstromNode {
    fn process(
        &mut self,
        msg: Message<UniqueIdMessage>,
    ) -> Result<Option<Vec<Message<UniqueIdMessage>>>> {
        match msg.body.body {
            UniqueIdMessage::Init {
                node_id: _,
                node_ids: _,
            } => {
                // TODO fix repetition with other nodes
                let reply = Ok(Some(vec![Message {
                    src: msg.dest,
                    dest: msg.src,
                    body: Body {
                        msg_id: Some(self.id),
                        in_reply_to: msg.body.msg_id,
                        body: UniqueIdMessage::InitOk {},
                    },
                }]));
                self.id += 1;
                reply
            }
            UniqueIdMessage::Generate {} => {
                let reply = Ok(Some(vec![Message {
                    src: msg.dest,
                    dest: msg.src,
                    body: Body {
                        msg_id: Some(self.id),
                        in_reply_to: msg.body.msg_id,
                        body: UniqueIdMessage::GenerateOk {
                            id: self.uuid_generator.generate(),
                        },
                    },
                }]));
                self.id += 1;
                reply
            }
            _ => Err(anyhow!("Received unknown message: {:?}", msg)),
        }
    }
}
fn main() -> anyhow::Result<()> {
    run(&mut UniqueIdGeneratorMaelstromNode::default())
}

#[cfg(test)]
mod tests {
    use crate::IdGenerator;
    use crate::UniqueIdGeneratorMaelstromNode;
    use crate::UniqueIdMessage;
    use maelstrom_rust::msg_protocol::*;
    use serde_json::from_str;
    use serde_json::to_string;
    use uuid::Uuid;

    mod fixtures {
        use super::*;

        pub fn init_msg() -> Message<UniqueIdMessage> {
            Message {
                src: Some("src".to_string()),
                dest: Some("dest".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: None,
                    body: UniqueIdMessage::Init {
                        node_id: "mynode1".to_string(),
                        node_ids: vec!["mynode1".to_string()],
                    },
                },
            }
        }

        pub fn init_ok_msg() -> Message<UniqueIdMessage> {
            Message {
                src: Some("dest".to_string()),
                dest: Some("src".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: Some(1),
                    body: UniqueIdMessage::InitOk {},
                },
            }
        }

        pub fn generate_msg() -> Message<UniqueIdMessage> {
            Message {
                src: Some("src".to_string()),
                dest: Some("dest".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: None,
                    body: UniqueIdMessage::Generate {},
                },
            }
        }
        pub fn generate_ok_msg(uuid: Uuid) -> Message<UniqueIdMessage> {
            Message {
                src: Some("dest".to_string()),
                dest: Some("src".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: Some(1),
                    body: UniqueIdMessage::GenerateOk { id: uuid },
                },
            }
        }
    }

    mod stubs {
        use super::*;
        pub struct FakeIdGenerator {
            uuid: Uuid,
        }

        impl FakeIdGenerator {
            pub fn new(uuid: Uuid) -> Self {
                Self { uuid }
            }
        }
        impl IdGenerator for FakeIdGenerator {
            fn generate(&self) -> Uuid {
                self.uuid
            }
        }
    }

    #[test]
    fn test_msg_processing_init() {
        let mut processor = UniqueIdGeneratorMaelstromNode::default();
        let msg = fixtures::init_msg();
        let reply = processor.process(msg);
        assert_eq!(reply.unwrap(), Some(vec![fixtures::init_ok_msg()]));
    }

    #[test]
    fn test_serde_msg_generate() {
        let msg = fixtures::generate_msg();
        let msg_serialized = to_string(&msg).unwrap();
        let msg_round_trip = from_str::<Message<UniqueIdMessage>>(&msg_serialized).unwrap();
        assert_eq!(msg, msg_round_trip);
    }

    #[test]
    fn test_serde_msg_generate_ok() {
        let uuid = Uuid::new_v4();
        let msg = fixtures::generate_ok_msg(uuid);
        let msg_serialized = to_string(&msg).unwrap();
        let msg_round_trip = from_str::<Message<UniqueIdMessage>>(&msg_serialized).unwrap();
        assert_eq!(msg, msg_round_trip);
    }

    #[test]
    fn test_msg_processing_generate() {
        use stubs::FakeIdGenerator;

        let uuid = Uuid::new_v4();
        let id_generator = FakeIdGenerator::new(uuid);
        let mut processor = UniqueIdGeneratorMaelstromNode::new(1, Box::new(id_generator));
        let msg = fixtures::generate_msg();

        let reply = processor.process(msg);

        assert_eq!(reply.unwrap(), Some(vec![fixtures::generate_ok_msg(uuid)]));
    }

    #[test]
    fn test_msg_processing_unhandled_generate_ok() {
        let mut processor = UniqueIdGeneratorMaelstromNode::default();
        let msg = fixtures::generate_ok_msg(Uuid::new_v4());

        let expected_err_msg = format!("Received unknown message: {:?}", &msg);
        let result = processor.process(msg);
        match result {
            Err(err) => assert_eq!(format!("{}", err), expected_err_msg),
            _ => panic!("Expected error"),
        }
    }

    #[test]
    fn test_msg_processor_id_increments_on_every_msg() {
        use stubs::FakeIdGenerator;

        // TODO extract this type of test to shared test utils
        let uuid = Uuid::new_v4();
        let id_generator = FakeIdGenerator::new(uuid);
        let mut processor = UniqueIdGeneratorMaelstromNode::new(1, Box::new(id_generator));

        fn assert_reply_to_msg(
            processor: &mut UniqueIdGeneratorMaelstromNode,
            msg: Message<UniqueIdMessage>,
            expected_reply: Option<Vec<Message<UniqueIdMessage>>>,
        ) {
            let maybe_reply = processor.process(msg).unwrap();
            assert_eq!(maybe_reply, expected_reply);
        }

        let base_msg = fixtures::generate_msg();
        let base_reply = fixtures::generate_ok_msg(uuid);

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
}

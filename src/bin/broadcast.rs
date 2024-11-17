use anyhow::anyhow;
use anyhow::Ok;
use anyhow::Result;
use maelstrom_rust::msg_protocol::*;
use maelstrom_rust::runner::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

struct BroadcastMaelstromNode {
    id: i64,
    messages: Vec<i64>,
    node_id: Option<String>,
    // all nodes in the network minus current node
    node_ids: Vec<String>,
}

impl BroadcastMaelstromNode {
    pub fn new(
        id: i64,
        node_id: Option<String>,
        messages: Vec<i64>,
        node_ids: Vec<String>,
    ) -> Self {
        Self {
            id,
            node_id,
            messages,
            node_ids,
        }
    }
}

impl Default for BroadcastMaelstromNode {
    fn default() -> Self {
        Self::new(1, None, vec![], vec![])
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum BroadcastMessage {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {},
    Broadcast {
        message: i64,
    },
    BroadcastOk {},
    Read {},
    ReadOk {
        messages: Vec<i64>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk {},
}

impl Processor<BroadcastMessage> for BroadcastMaelstromNode {
    fn process(
        &mut self,
        msg: Message<BroadcastMessage>,
    ) -> Result<Option<Message<BroadcastMessage>>> {
        match msg.body.body {
            BroadcastMessage::Init { node_id, node_ids } => {
                let reply = Ok(Some(Message {
                    src: msg.dest,
                    dest: msg.src,
                    body: Body {
                        msg_id: Some(self.id),
                        in_reply_to: msg.body.msg_id,
                        body: BroadcastMessage::InitOk {},
                    },
                }));
                self.id += 1;
                self.node_id = Some(node_id.clone());
                self.node_ids = node_ids;
                // we keep only other nodes by removing the current one
                self.node_ids.retain(|n| n != &node_id);
                reply
            }
            BroadcastMessage::Broadcast { message } => {
                self.messages.push(message);
                let reply = Ok(Some(Message {
                    src: msg.dest,
                    dest: msg.src,
                    body: Body {
                        msg_id: Some(self.id),
                        in_reply_to: msg.body.msg_id,
                        body: BroadcastMessage::BroadcastOk {},
                    },
                }));
                self.id += 1;
                reply
            }
            BroadcastMessage::Read {} => {
                let reply = Ok(Some(Message {
                    src: msg.dest,
                    dest: msg.src,
                    body: Body {
                        msg_id: Some(self.id),
                        in_reply_to: msg.body.msg_id,
                        body: BroadcastMessage::ReadOk {
                            messages: self.messages.clone(),
                        },
                    },
                }));
                self.id += 1;
                reply
            }

            BroadcastMessage::Topology { topology } => {
                let reply = Ok(Some(Message {
                    src: msg.dest,
                    dest: msg.src,
                    body: Body {
                        msg_id: Some(self.id),
                        in_reply_to: msg.body.msg_id,
                        body: BroadcastMessage::TopologyOk {},
                    },
                }));

                // TODO update current node's topology, move it to node
                if let Some(node) = &self.node_id {
                    if let Some(node_ids) = topology.get(node) {
                        self.node_ids = node_ids.clone();
                    }
                };

                self.id += 1;

                reply
            }

            _ => Err(anyhow!("Received unknown message: {:?}", msg)),
        }
    }
}
fn main() -> anyhow::Result<()> {
    run(&mut BroadcastMaelstromNode::default())
}

#[cfg(test)]
mod tests {

    use crate::BroadcastMaelstromNode;
    use crate::BroadcastMessage;

    use maelstrom_rust::msg_protocol::*;

    use maplit::hashmap;
    use serde_json::from_str;
    use serde_json::to_string;
    mod fixtures {
        use super::*;

        use std::{collections::HashMap, vec};

        pub fn init_msg() -> Message<BroadcastMessage> {
            Message {
                src: Some("src".to_string()),
                dest: Some("dest".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: None,
                    body: BroadcastMessage::Init {
                        node_id: "node1".to_string(),
                        node_ids: vec!["node1".to_string(), "node2".to_string()],
                    },
                },
            }
        }
        pub fn init_ok_msg() -> Message<BroadcastMessage> {
            Message {
                src: Some("dest".to_string()),
                dest: Some("src".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: Some(1),
                    body: BroadcastMessage::InitOk {},
                },
            }
        }
        pub fn broadcast_msg() -> Message<BroadcastMessage> {
            Message {
                src: Some("src".to_string()),
                dest: Some("dest".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: None,
                    body: BroadcastMessage::Broadcast { message: 1 },
                },
            }
        }
        pub fn broadcast_ok_msg() -> Message<BroadcastMessage> {
            Message {
                src: Some("dest".to_string()),
                dest: Some("src".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: Some(1),
                    body: BroadcastMessage::BroadcastOk {},
                },
            }
        }

        pub fn read_msg() -> Message<BroadcastMessage> {
            Message {
                src: Some("src".to_string()),
                dest: Some("dest".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: None,
                    body: BroadcastMessage::Read {},
                },
            }
        }
        pub fn read_ok_msg(messages: Vec<i64>) -> Message<BroadcastMessage> {
            Message {
                src: Some("dest".to_string()),
                dest: Some("src".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: Some(1),
                    body: BroadcastMessage::ReadOk { messages },
                },
            }
        }
        pub fn topology_msg(
            maybe_provided_topo: Option<HashMap<String, Vec<String>>>,
        ) -> Message<BroadcastMessage> {
            let topology = maybe_provided_topo.unwrap_or(hashmap! {
                "node1".to_string() => vec!["node2".to_string(), "node3".to_string()],
                "node2".to_string() => vec!["node3".to_string()],
                "node3".to_string() => vec!["node2".to_string()]
            });
            Message {
                src: Some("src".to_string()),
                dest: Some("dest".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: None,
                    body: BroadcastMessage::Topology { topology },
                },
            }
        }
        pub fn topology_ok_msg() -> Message<BroadcastMessage> {
            Message {
                src: Some("dest".to_string()),
                dest: Some("src".to_string()),
                body: Body {
                    msg_id: Some(1),
                    in_reply_to: Some(1),
                    body: BroadcastMessage::TopologyOk {},
                },
            }
        }
    }
    #[test]
    fn test_msg_processing_init() {
        let mut processor = BroadcastMaelstromNode::default();
        let msg = fixtures::init_msg();
        let reply = processor.process(msg);
        assert_eq!(reply.unwrap(), Some(fixtures::init_ok_msg()));
        assert_eq!(processor.node_id, Some("node1".to_string()));
        assert_eq!(processor.node_ids, vec!["node2".to_string()]);
    }

    #[test]
    fn test_msg_processing_broadcast() {
        let mut processor = BroadcastMaelstromNode::default();
        let msg = fixtures::broadcast_msg();

        let reply = processor.process(msg);
        assert_eq!(reply.unwrap(), Some(fixtures::broadcast_ok_msg()));
        assert_eq!(processor.messages, vec![1]);
    }

    #[test]
    fn test_msg_processing_topology() {
        let mut processor = BroadcastMaelstromNode::new(
            1,
            Some("node1".to_string()),
            vec![],
            vec!["node1".to_string()],
        );
        let msg = fixtures::topology_msg(None);

        let reply = processor.process(msg);
        assert_eq!(reply.unwrap(), Some(fixtures::topology_ok_msg()));
        assert_eq!(processor.node_id, Some("node1".to_string()));
        assert_eq!(
            processor.node_ids,
            vec!["node2".to_string(), "node3".to_string()]
        );
    }

    #[test]
    fn test_msg_processing_topology_without_node_intialized_before() {
        let mut processor = BroadcastMaelstromNode::default();
        let msg = fixtures::topology_msg(None);

        let reply = processor.process(msg);
        assert_eq!(reply.unwrap(), Some(fixtures::topology_ok_msg()));
        // we shouldn't update with values from the topology message if current node_id hasn't even been initialized
        assert_eq!(processor.node_id, None);
        let expected_node_ids: Vec<String> = vec![];
        assert_eq!(processor.node_ids, expected_node_ids);
    }

    #[test]
    fn test_msg_processing_topology_without_current_nodeid_mapped() {
        let mut processor = BroadcastMaelstromNode::new(
            1,
            Some("node1".to_string()),
            vec![],
            vec!["node1".to_string()],
        );
        let msg = fixtures::topology_msg(Some(hashmap! {
            "node2".to_string() => vec!["node3".to_string()],
            "node3".to_string() => vec!["node2".to_string()]
        }));

        let reply = processor.process(msg);
        assert_eq!(reply.unwrap(), Some(fixtures::topology_ok_msg()));
        // we shouldn't update with values from the topology message if current node_id is not mapped
        assert_eq!(processor.node_id, Some("node1".to_string()));
        assert_eq!(processor.node_ids, vec!["node1".to_string(),]);
    }

    #[test]
    fn test_msg_processing_read() {
        let stored_messages = vec![1, 2];
        let topology = vec!["2".to_string()];

        let mut processor =
            BroadcastMaelstromNode::new(1, None, stored_messages.clone(), topology.clone());
        let msg = fixtures::read_msg();

        let reply = processor.process(msg);
        assert_eq!(
            reply.unwrap(),
            Some(fixtures::read_ok_msg(stored_messages.clone()))
        );
    }
    #[test]
    fn test_msg_processing_unhandled_msg() {
        let mut processor: BroadcastMaelstromNode = BroadcastMaelstromNode::default();
        let msg = fixtures::init_ok_msg();

        let expected_err_msg = format!("Received unknown message: {:?}", &msg);
        let result = processor.process(msg);
        match result {
            Err(err) => assert_eq!(format!("{}", err), expected_err_msg),
            _ => panic!("Expected error"),
        }
    }

    #[test]
    fn test_msg_processor_id_increments_on_every_msg() {
        let mut processor = BroadcastMaelstromNode::default();

        fn assert_reply_to_msg(
            processor: &mut BroadcastMaelstromNode,
            msg: Message<BroadcastMessage>,
            expected_reply: Option<Message<BroadcastMessage>>,
        ) {
            let maybe_reply = processor.process(msg).unwrap();
            assert_eq!(maybe_reply, expected_reply);
        }
        let base_msg = fixtures::broadcast_msg();
        let base_reply = fixtures::broadcast_ok_msg();

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
    #[test]
    fn test_serde_msg_broadcast() {
        assert_round_trip(fixtures::broadcast_msg());
    }

    #[test]
    fn test_serde_msg_broadcast_ok() {
        assert_round_trip(fixtures::broadcast_ok_msg());
    }

    #[test]
    fn test_serde_msg_read() {
        assert_round_trip(fixtures::read_msg());
    }

    #[test]
    fn test_serde_msg_read_ok() {
        assert_round_trip(fixtures::read_ok_msg(vec![1, 2]));
    }

    #[test]
    fn test_serde_msg_topology() {
        assert_round_trip(fixtures::topology_msg(None));
    }

    #[test]
    fn test_serde_msg_topology_ok() {
        assert_round_trip(fixtures::topology_ok_msg());
    }

    #[test]
    fn test_serde_msg_init() {
        assert_round_trip(fixtures::init_msg());
    }
    #[test]
    fn test_serde_msg_init_ok() {
        assert_round_trip(fixtures::init_ok_msg());
    }

    fn assert_round_trip(msg: Message<BroadcastMessage>) {
        let msg_serialized = to_string(&msg).unwrap();
        let msg_round_trip = from_str::<Message<BroadcastMessage>>(&msg_serialized).unwrap();
        assert_eq!(msg, msg_round_trip);
    }
}

use anyhow::anyhow;
use anyhow::Result;
use maelstrom_rust::msg_protocol::*;
use maelstrom_rust::runner::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

struct UniqueIdGeneratorMaelstromNode {
    id: i64,
}

impl UniqueIdGeneratorMaelstromNode {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

impl Default for UniqueIdGeneratorMaelstromNode {
    fn default() -> Self {
        Self::new(1)
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
    ) -> Result<Option<Message<UniqueIdMessage>>> {
        match msg.body.body {
            UniqueIdMessage::Init {
                node_id: _,
                node_ids: _,
            } => {
                // TODO fix repetition with other nodes
                let reply = Ok(Some(Message {
                    src: msg.dest,
                    dest: msg.src,
                    body: Body {
                        msg_id: Some(self.id),
                        in_reply_to: msg.body.msg_id,
                        body: UniqueIdMessage::InitOk {},
                    },
                }));
                self.id += 1;
                reply
            }
            UniqueIdMessage::Generate {} => {
                let reply = Ok(Some(Message {
                    src: msg.dest,
                    dest: msg.src,
                    body: Body {
                        msg_id: Some(self.id),
                        in_reply_to: msg.body.msg_id,
                        body: UniqueIdMessage::GenerateOk { id: Uuid::new_v4() },
                    },
                }));
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
mod tests {}

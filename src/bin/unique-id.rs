use anyhow::anyhow;
use anyhow::Result;
use maelstrom_rust::msg_protocol::{Message, Processor};
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
            UniqueIdMessage::Init { node_id, node_ids } => todo!(),
            UniqueIdMessage::InitOk {} => todo!(),
            UniqueIdMessage::Generate {} => todo!(),
            UniqueIdMessage::GenerateOk { id } => todo!(),
            _ => Err(anyhow!("Received unknown message: {:?}", msg)),
        }
    }
}
fn main() -> anyhow::Result<()> {
    run(&mut UniqueIdGeneratorMaelstromNode::default())
}

#[cfg(test)]
mod tests {}

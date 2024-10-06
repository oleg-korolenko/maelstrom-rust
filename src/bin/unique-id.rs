use anyhow::anyhow;
use anyhow::Result;
use maelstrom_rust::msg_protocol::{Message, MessageBodyType, Processor};
use maelstrom_rust::runner::*;
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

impl Processor for UniqueIdGeneratorMaelstromNode {
    fn process(&mut self, msg: Message) -> Result<Option<Message>> {
        match msg.body.body {
            MessageBodyType::Init { node_id, node_ids } => todo!(),
            MessageBodyType::InitOk {} => todo!(),
            MessageBodyType::Generate {} => todo!(),
            MessageBodyType::GenerateOk { id } => todo!(),
            _ => Err(anyhow!("Received unknown message: {:?}", msg)),
        }
    }
}
fn main() -> anyhow::Result<()> {
    run(&mut UniqueIdGeneratorMaelstromNode::default())
}

#[cfg(test)]
mod tests {}

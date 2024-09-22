use anyhow::Result;
use maelstrom_rust::msg_protocol::{Message, Processor};
use maelstrom_rust::runner::*;
struct IdGeneratorMaelstromNode {
    id: i64,
}

impl IdGeneratorMaelstromNode {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

impl Default for IdGeneratorMaelstromNode {
    fn default() -> Self {
        Self::new(1)
    }
}

impl Processor for IdGeneratorMaelstromNode {
    fn process(&mut self, msg: Message) -> Result<Option<Message>> {
        todo!()
    }
}
fn main() -> anyhow::Result<()> {
    run(&mut IdGeneratorMaelstromNode::default())
}

#[cfg(test)]
mod tests {}

use std::io::StdoutLock;
use std::io::Write;

use anyhow::Context;

use anyhow::Ok;
use anyhow::Result;

pub mod msg_protocol {
    use anyhow::Result;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Message<T> {
        pub src: Option<String>,
        pub dest: Option<String>,
        pub body: Body<T>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Body<T> {
        pub msg_id: Option<i64>,
        pub in_reply_to: Option<i64>,

        #[serde(flatten)]
        pub body: T,
    }

    pub trait Processor<MessageType> {
        fn process(
            &mut self,
            msg: Message<MessageType>,
        ) -> Result<Option<Vec<Message<MessageType>>>>;
    }
}
pub mod runner {
    use super::*;
    use msg_protocol::*;

    fn serialize<MessageType>(
        maybe_reply: Option<Vec<Message<MessageType>>>,
        out: &mut StdoutLock,
    ) -> Result<()>
    where
        MessageType: serde::Serialize,
    {
        if let Some(replies) = maybe_reply {
            for reply in replies {
                serde_json::to_writer(&mut *out, &reply).context("Serialize reply message")?;
                // writing a new line to flush the line writer
                out.write_all(b"\n")?;
            }
        }
        Ok(())
    }

    pub fn run<MessageType, P>(processor: &mut P) -> anyhow::Result<()>
    where
        MessageType: for<'de> serde::Deserialize<'de>,
        P: Processor<MessageType>,
        MessageType: serde::Serialize,
    {
        let stdin = std::io::stdin().lock();
        let deserializer = serde_json::Deserializer::from_reader(stdin);
        let mut stdout = std::io::stdout().lock();
        deserializer
            .into_iter::<Message<MessageType>>()
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

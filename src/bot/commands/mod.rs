pub mod parser;

use std::sync::Arc;

use lazy_static::lazy_static;
use serenity::{model::channel::Message, prelude::*};

lazy_static! {
    pub static ref COMMAND_LIST: [Arc<dyn Command>; 1] = [
        Arc::new(list::Command::default()),
    ];
}

mod list {
    use clap::Parser;

    /// List commands
    #[derive(Parser, Debug)]
    #[command(version, about, long_about = None)]
    pub struct Args;

    #[derive(Default)]
    pub struct Command;

    impl super::Command for Command {
        type Args = Args;

        fn name(&self) -> &'static str {
            "list"
        }
        fn run(
            &self,
            _args: Self::Args,
            _ctx: serenity::prelude::Context,
            _msg: serenity::model::prelude::Message,
        ) {
        }
    }
}

pub enum DataType {
    String(String),
}

pub trait Command: Send + Sync {
    type Args: clap::Parser
    where
        Self: Sized;

    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str {
        ""
    }
    fn init() -> Self where Self: Default { Self::default() }
    fn run(&self, args: Self::Args, ctx: Context, msg: Message)
    where
        Self: Sized;
}

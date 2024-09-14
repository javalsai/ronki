use std::{collections::HashMap, sync::Arc};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref COMMAND_LIST: [Arc<dyn super::Command>; 1] = [Arc::new(cmd_list::Command),];
    pub static ref COMMAND_MAP: HashMap<&'static str, Arc<dyn super::Command>> = {
        let mut m = HashMap::new();
        for cmd in COMMAND_LIST.iter() {
            m.insert(cmd.name(), Arc::clone(cmd));
        }
        m
    };
}

mod cmd_list {
    use clap::Parser;

    /// List commands
    #[derive(Parser, Debug)]
    #[command(version, about, long_about = None)]
    pub struct Args;

    #[derive(Default)]
    pub struct Command;

    impl super::super::Command for Command {
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

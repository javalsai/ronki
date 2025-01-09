use std::{collections::HashMap, sync::Arc};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref COMMAND_LIST: [&'static dyn super::Command; 1] = [&cmd_list::ListCommand,];
    pub static ref COMMAND_MAP: HashMap<&'static str, &'static dyn super::Command> = {
        let mut m = HashMap::with_capacity(COMMAND_LIST.len());
        for cmd in COMMAND_LIST.iter() {
            m.insert(cmd.name(), *cmd);
        }
        m
    };
}

mod cmd_list {
    use clap::Parser;

    use super::super::parser::{Environ, EnvironValue};

    use super::COMMAND_MAP;

    /// List commands
    #[derive(Parser, Debug)]
    #[command(version, about, long_about = None)]
    pub struct ListArgs;

    #[derive(Default)]
    pub struct ListCommand;

    impl super::super::Command for ListCommand {
        type Args = ListArgs;

        fn name(&self) -> &'static str {
            "list"
        }

        fn description(&self) -> &'static str {
            "lists all commands avaliable in the bot"
        }

        fn run(
            &self,
            _args: Vec<EnvironValue>,
            _ctx: &mut serenity::prelude::Context,
            _msg: &mut serenity::model::prelude::Message,
        ) -> super::super::parser::EnvironValue {
            let mut command_list = String::from("Commands:\n");
            for cmd in COMMAND_MAP.iter() {
                command_list += &format!(" '{}' - {}\n", cmd.0, cmd.1.description());
            }
            super::super::parser::EnvironValue::String(command_list.into())
        }
    }
}

pub mod list;
pub mod parser;

use crate::util::humanize::units::sizes;

use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    os::unix::ffi::OsStringExt,
};

use procfs::WithCurrentSystemInfo;
use serenity::{model::channel::Message, prelude::*};

pub type DefaultEnviron<'a> = HashMap<String, parser::EnvironValue>;
impl<'a> parser::Environ<'a> for DefaultEnviron<'a> {
    fn get(&self, key: &str) -> Option<&parser::EnvironValue> {
        self.get(key)
    }

    fn set(&mut self, key: String, value: parser::EnvironValue) -> Option<parser::EnvironValue> {
        self.insert(key, value)
    }

    fn entries(&self) -> impl Iterator<Item = (&str, &parser::EnvironValue)> {
        self.iter().map(|(k, v)| (k.as_str(), v))
    }
}

pub trait Command: Send + Sync {
    type Args: clap::Parser
    where
        Self: Sized;

    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str {
        ""
    }
    fn init() -> Self
    where
        Self: Default,
    {
        Self::default()
    }
    fn run(&self, args: Self::Args, ctx: Context, msg: Message)
    where
        Self: Sized;
}

#[derive(Default)]
pub struct HardcodedExecuter;

#[derive(Debug)]
pub enum HardcodedExecuterError {
    NoCommandName,
    ImproperEncoding,
    UnknownCommand,
    // TODO: this will be a enum for each cmd
    // ig they'll have a common trait for display
    // aaand, be a dyn prob
    CommandError(&'static str),
    NoStringCommandName,
    UnserializableValue,
}

impl parser::Executer<HardcodedExecuterError> for HardcodedExecuter {
    fn execute<'a>(
        &mut self,
        mut args: Vec<parser::EnvironValue>,
        env: &mut impl parser::Environ<'a>,
    ) -> Result<parser::EnvironValue, HardcodedExecuterError> {
        let parser::EnvironValue::String(cmd) =
            args.first().ok_or(HardcodedExecuterError::NoCommandName)?
        else {
            do yeet HardcodedExecuterError::NoStringCommandName;
        };
        let cmd = cmd
            .to_str()
            .ok_or(HardcodedExecuterError::ImproperEncoding)?;

        match cmd {
            "help" => Ok(concat!(
                "Hardcoded Executer:\n\n",
                "Commands:\n",
                "  'help': Display list of commands\n",
                "  'echo': Echo 👍\n",
                "  'env': List env variables\n",
                "  'let': Define an env variable\n",
                "  'printargs': Prints arguments\n",
                "  'memusage': Print memory usage\n",
                "  'music': Full separate music handler",
            )
            .into()),
            "echo" => Ok(parser::EnvironValue::String(
                args.into_iter()
                    .skip(1)
                    .map(|v| {
                        v.as_string()
                            .ok_or(HardcodedExecuterError::UnserializableValue)
                    })
                    .try_collect::<Vec<_>>()? // I hate this, but idk, no try_map and idwanna do
                    // mine
                    .iter()
                    .intersperse(&OsString::from(" "))
                    .fold(OsString::new(), |mut acc, v| {
                        acc.push(v);
                        acc
                    }),
            )),
            "env" => {
                let result = env
                    .entries()
                    .map(|(k, v)| format!("{k}={v:?}"))
                    .intersperse(String::from("\n"))
                    .fold(OsString::new(), |mut acc, new| {
                        acc.push(OsString::from(new));
                        acc
                    });

                Ok(parser::EnvironValue::String(result))
            }
            "let" => {
                if args.len() != 2 {
                    do yeet HardcodedExecuterError::CommandError("invalid arg count")
                }

                let expr = args
                    .remove(1)
                    .as_string()
                    .ok_or(HardcodedExecuterError::UnserializableValue)?;

                let (k, v) = expr
                    .as_encoded_bytes()
                    .split_once(|b| *b == b'=')
                    .ok_or(HardcodedExecuterError::CommandError("no '=' found"))?;

                let k = String::from_utf8(k.to_vec()).map_err(|_| {
                    HardcodedExecuterError::CommandError("key is not a utf8 sequence")
                })?;

                let v = OsString::from_vec(v.to_vec());

                env.set(k, parser::EnvironValue::String(v)); // no v=$(stuff) btw

                Ok(parser::EnvironValue::None)
            }
            "printargs" => {
                Ok(parser::EnvironValue::String(OsString::from(format!("{args:?}"))))
            }
            "memusage" => {
                let me = procfs::process::Process::myself().unwrap();
                let stat = me.stat().unwrap();

                Ok(parser::EnvironValue::String(OsString::from(format!(
                    "pid({}): rss({}) vsize({})",
                    stat.pid,
                    sizes::bytes_to_binary(stat.rss_bytes().get() as f64, 2),
                    sizes::bytes_to_binary(stat.vsize as f64, 2),
                ))))
            }
            "music" => {
                let Some(nice_args): Option<Vec<String>> = args[1..].into_iter().map(|osstr| {
                    let parser::EnvironValue::String(osstr) = osstr else { return None; };
                    osstr.clone().into_string().ok()
                }).try_collect() else {
                    return Ok(parser::EnvironValue::String(OsString::from("invalid encoding argument")))
                };
                let response = hardcoded_music_player::main_handler(nice_args.as_slice());
                Ok(parser::EnvironValue::String(OsString::from(response)))
            }
            _ => Err(HardcodedExecuterError::UnknownCommand),
        }
    }
}

pub mod hardcoded_music_player {
    pub fn main_handler(args: &[String]) -> String {
        String::new()
    }
}

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

pub type DefaultEnviron<'a> = HashMap<String, OsString>;
impl<'a> parser::Environ<'a> for DefaultEnviron<'a> {
    fn get(&self, key: &str) -> Option<&OsStr> {
        self.get(key).map(move |dat| dat.as_os_str())
    }

    fn set(&mut self, key: String, value: OsString) -> Option<OsString> {
        self.insert(key, value)
    }

    fn entries(&self) -> impl Iterator<Item = (&str, &OsStr)> {
        self.iter().map(|(k, v)| (k.as_str(), v.as_os_str()))
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
}

impl parser::Executer<HardcodedExecuterError> for HardcodedExecuter {
    fn execute<'a>(
        &mut self,
        args: Vec<OsString>,
        env: &mut impl parser::Environ<'a>,
    ) -> Result<OsString, HardcodedExecuterError> {
        let cmd = args.first().ok_or(HardcodedExecuterError::NoCommandName)?;
        let cmd = cmd
            .to_str()
            .ok_or(HardcodedExecuterError::ImproperEncoding)?;
        let args = &args[1..];

        match cmd {
            "help" => Ok(concat!(
                "Hardcoded Executer:\n\n",
                "Commands:\n",
                "  'help': Display list of commands\n",
                "  'echo': Echo 👍\n",
                "  'env': List env variables\n",
                "  'let': Define an env variable\n",
                "  'memusage': Print memory usage",
            )
            .into()),
            "echo" => Ok(args.iter().intersperse(&OsString::from(" ")).fold(
                OsString::new(),
                |mut acc, v| {
                    acc.push(v);
                    acc
                },
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

                Ok(result)
            }
            "let" => {
                if args.len() != 1 {
                    do yeet HardcodedExecuterError::CommandError("invalid arg count")
                }
                let Some(expr) = args.first() else {
                    do yeet HardcodedExecuterError::CommandError("wtf")
                };

                let (k, v) = expr
                    .as_encoded_bytes()
                    .split_once(|b| *b == b'=')
                    .ok_or(HardcodedExecuterError::CommandError("no '=' found"))?;

                let k = String::from_utf8(k.to_vec()).map_err(|_| {
                    HardcodedExecuterError::CommandError("key is not a utf8 sequence")
                })?;

                let v = OsString::from_vec(v.to_vec());

                env.set(k, v);

                Ok(OsString::new())
            }
            "memusage" => {
                let me = procfs::process::Process::myself().unwrap();
                let stat = me.stat().unwrap();

                Ok(OsString::from(format!(
                    "pid({}): rss({}) vsize({})",
                    stat.pid,
                    sizes::bytes_to_binary(stat.rss_bytes().get() as f64, 2),
                    sizes::bytes_to_binary(stat.vsize as f64, 2),
                )))
            }
            _ => Err(HardcodedExecuterError::UnknownCommand),
        }
    }
}

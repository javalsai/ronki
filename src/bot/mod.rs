pub mod commands;

use crate::util::humanize::units::sizes;

use std::ffi::OsString;

use procfs::WithCurrentSystemInfo;
use serenity::{async_trait, model::channel::Message, prelude::*};

struct Handler<T: surrealdb::Connection> {
    config: crate::config::Schema,
    #[allow(dead_code)]
    db: surrealdb::Surreal<T>,
}

#[derive(Default)]
struct DummyEnvironExecuter(Vec<String>);
impl commands::parser::Environ for DummyEnvironExecuter {
    fn get(&mut self, key: &str) -> Option<OsString> {
        let new_key = format!("/{key}\\");
        self.0
            .push(format!("env {key:?} asked for, returning ${new_key:?}"));
        Some(OsString::from(new_key))
    }
}
impl commands::parser::Executer for DummyEnvironExecuter {
    fn execute(
        &mut self,
        args: Vec<OsString>,
        _env: &mut impl commands::parser::Environ,
    ) -> OsString {
        self.0.push(format!("called with: {args:?}"));
        let mut cumulative = OsString::new();
        args.into_iter()
            .intersperse(OsString::from("-"))
            .for_each(|x| cumulative.push(x));
        self.0.push(format!("returning {cumulative:?}"));
        cumulative
    }
}

#[async_trait]
impl<T: surrealdb::Connection> EventHandler for Handler<T> {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!memusage" {
            let me = procfs::process::Process::myself().unwrap();
            let stat = me.stat().unwrap();
            let _ = msg
                .reply(
                    &ctx.http,
                    format!(
                        "pid({}): rss({}) vsize({})",
                        stat.pid,
                        sizes::bytes_to_binary(stat.rss_bytes().get()),
                        sizes::bytes_to_binary(stat.vsize),
                    ),
                )
                .await;
            return;
        } else if msg.content == "!panic" {
            let _ = msg.reply(&ctx.http, "panicking!").await;
            panic!("someone requested to panic");
        } else if msg.content == "!blocktest" {
            tokio::spawn(async move {
                let _ = msg.reply(&ctx.http, "sleeping for 5s").await;
                async_std::task::sleep(std::time::Duration::from_secs(5)).await;
                let _ = msg.reply(&ctx.http, "woke up!").await;
            });
            return;
        }

        let a =
            commands::parser::MsgParser::new(&self.config.prefix.to_string(), &msg.content).parse();

        match a {
            Ok(a) => {
                if !a.is_empty() {
                    let mut dmmy = DummyEnvironExecuter::default();
                    let _ = msg.reply(&ctx.http, format!("`{:?}`", a.clone())).await;
                    for cmd in a {
                        let mref = &mut dmmy as *mut _;
                        // TODO: temporal "fix" to not think this too much
                        let (mref1, mref2) = unsafe { (&mut *mref, &mut *mref) };
                        let result = cmd.resolve(mref1, mref2);
                        let _ = msg
                            .reply(
                                &ctx.http,
                                format!("`{result:?}`\n```rs\n{}\n```", dmmy.0.join("\n")),
                            )
                            .await;
                    }
                }
            }
            Err(a) => {
                let _ = msg.reply(&ctx.http, format!("**err**: `{a:?}`")).await;
            }
        };
    }
}

pub async fn load(
    config: crate::config::Schema,
    db: surrealdb::Surreal<impl surrealdb::Connection>,
) {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&config.token, intents)
        .event_handler(Handler { config, db })
        .await
        .expect("Error creating client");

    if let Err(err) = client.start().await {
        println!("Error on Client {err:?}");
    };
}

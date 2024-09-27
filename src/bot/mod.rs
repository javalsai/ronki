pub mod commands;

use std::ffi::OsString;

use serenity::{async_trait, model::channel::Message, prelude::*};

struct Handler<T: surrealdb::Connection> {
    config: crate::config::Schema,
    #[allow(dead_code)]
    db: surrealdb::Surreal<T>,
}

#[async_trait]
impl<T: surrealdb::Connection> EventHandler for Handler<T> {
    async fn message(&self, ctx: Context, msg: Message) {
        let maybe_cmds =
            commands::parser::MsgParser::new(&self.config.prefix.to_string(), &msg.content).parse();

        match maybe_cmds {
            Ok(cmds) => {
                if !cmds.is_empty() {
                    // TODO: make debug/trace log macro
                    //let _ = msg.reply(&ctx.http, format!("`{:?}`", a.clone())).await;

                    let mut environ = commands::DefaultEnviron::default();
                    environ.insert(
                        String::from("USER"),
                        commands::parser::EnvironValue::String(OsString::from(&msg.author.name)),
                    );
                    environ.insert(
                        String::from("USERID"),
                        commands::parser::EnvironValue::UNumber(msg.author.id.get() as u128)
                    );

                    let mut executer = commands::HardcodedExecuter;

                    let mut output = String::new();
                    for cmd in cmds {
                        match cmd.resolve(&mut environ, &mut executer) {
                            Ok(cmd_output) => {
                                let Some(cmd_output) = cmd_output.as_string() else {
                                    let _ = msg.reply(&ctx.http, "**err**: Unserializable Output");
                                    return;
                                };
                                if !cmd_output.is_empty() {
                                    output += cmd_output.to_string_lossy().as_ref();
                                    output += "\n";
                                }
                            }
                            Err(err) => {
                                let _ = msg
                                    .reply(&ctx.http, format!("**execution error**: `{err:?}`"))
                                    .await;
                                return;
                            }
                        };
                    }
                    let _ = msg.reply(&ctx.http, format!("```\n{output}\n```")).await;
                }
            }
            Err(err) => {
                let _ = msg.reply(&ctx.http, format!("**err**: `{err:?}`")).await;
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

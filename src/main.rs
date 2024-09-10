#![feature(
    generic_arg_infer,
    iterator_try_collect,
    iter_intersperse,
    yeet_expr,
)]

pub mod args;
pub mod bot;
pub mod config;
pub mod consts;
pub mod util;

use std::{fs::File, io::Read};

use args::Args;
use config::Schema;

use clap::Parser;
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let str_path = args.config.to_str().expect("Invalid config path");

    if args.reset_config || !args.config.exists() {
        println!("'{}' copying default config", str_path);
        std::fs::write(&args.config, consts::DEFAULT_CONFIG)?;
        return Ok(());
    }

    let mut conf_file = File::open(&args.config)?;

    let mut config = String::new();
    conf_file
        .read_to_string(&mut config)
        .expect("Error reading file");
    let config: Schema = toml::from_str(&config).expect("Invalid config");

    let db = Surreal::new::<Ws>(&config.surrealdb.address).await?;

    db.signin(Root {
        username: &config.surrealdb.username,
        password: &config.surrealdb.password,
    })
    .await?;

    bot::load(config, db).await;
    Ok(())
}

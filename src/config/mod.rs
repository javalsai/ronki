use serde::Deserialize;
use serenity::model::id::{UserId, GuildId};

#[derive(Deserialize, Debug)]
pub struct SurrealDB {
    pub address: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct Schema {
    pub token: String,
    #[serde(default = "__default_prefix")]
    pub prefix: char,
    #[serde(default)]
    pub owners: Vec<UserId>,
    #[serde(default)]
    pub servers: Vec<GuildId>,
    pub surrealdb: SurrealDB,
}

fn __default_prefix() -> char { '!' }

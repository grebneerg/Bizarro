use rand::Rng;
use serde_json::json;
use serenity::{
    framework::StandardFramework,
    http,
    model::{channel::Message, gateway::Ready, id::ChannelId, webhook::Webhook},
    prelude::*,
};

use std::fs;
use std::path::PathBuf;

use markov::Chain;

mod chains;
use chains::*;

mod commands;
mod config;

use config::Config;

fn webhook(cid: ChannelId, name: String) -> Result<Webhook, serenity::Error> {
    http::create_webhook(*cid.as_u64(), &json!({ "name": name }))
}

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        println!("Online");
    }

    fn message(&self, ctx: Context, msg: Message) {
        if msg.webhook_id.is_some() {
            return;
        }

        if let Some(gid) = msg.guild_id {
            let author_id = msg.author.id;
            ctx.data
                .lock()
                .get_mut::<UserChains>()
                .unwrap()
                .feed(&author_id, &msg.content);
            if let Some(guild_id) = msg.guild_id {
                if msg.mentions.len() > 0 || msg.mention_roles.len() > 0 || msg.mention_everyone {
                    let mentions: Vec<_> = ctx
                        .data
                        .lock()
                        .get::<UserChains>()
                        .expect("No chains loaded")
                        .user_ids()
                        .iter()
                        .map(|id| id.to_user().expect("couldn't retrieve user"))
                        .filter(|user| {
                            msg.mentions.contains(&user)
                                || msg
                                    .mention_roles
                                    .iter()
                                    .any(|role| user.has_role(guild_id, role))
                        })
                        .collect();

                    let hook = webhook(msg.channel_id, "wide hook".to_owned())
                        .expect("could not make webhook");
                    let mut rng = rand::thread_rng();

                    for user in mentions {
                        let name = user.nick_in(gid).unwrap_or(user.name.clone());
                        let a_url = user.avatar_url().unwrap_or("https://crates.io/assets/Cargo-Logo-Small-c39abeb466d747f3be442698662c5260.png".to_string());
                        for _ in 0..rng.gen_range(1, 5) {
                            let res = ctx
                                .data
                                .lock()
                                .get::<UserChains>()
                                .unwrap()
                                .make_message(&user.id)
                                .unwrap();
                            hook.execute(false, |w| {
                                w.username(&name).avatar_url(&a_url).content(&res)
                            });
                        }
                    }
                    if msg.mention_everyone {
                        ctx.data
                            .lock()
                            .get::<EveryoneChain>()
                            .unwrap()
                            .str_iter_for(rng.gen_range(1, 5))
                            .for_each(|m| {
                                hook.execute(false, |w| w.username("Everyone").content(&m));
                            });
                    }

                    hook.delete();
                }
            }
        }
    }
}

fn main() {
    let config: Config =
        toml::from_str(&fs::read_to_string("Bizarro.toml").expect("Didn't find Bizarro.toml"))
            .expect("Invalid Bizarro.toml");

    let mut client = Client::new(&config.discord_token, Handler).expect("Error creating client");
    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix(&config.prefix))
            .cmd("ping", commands::ping)
            .cmd("regen", commands::regenerate)
            .cmd("save", commands::save),
    );

    let chains = UserChains::load(&config.chain_storage_dir).expect("couldn't load chains");

    let mut everyone_path = PathBuf::from(&config.chain_storage_dir);
    everyone_path.push("everyone.mkc");
    let everychain = Chain::load(everyone_path).expect("Could not load everyone chain");

    {
        let mut data = client.data.lock();
        data.insert::<UserChains>(chains);
        data.insert::<EveryoneChain>(everychain);
        data.insert::<Config>(config);
    }

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

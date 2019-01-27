use rand::Rng;
use serde_derive::Deserialize;
use serde_json::json;
use serenity::{
    http,
    model::{
        channel::Message,
        gateway::Ready,
        guild::*,
        id::{ChannelId, GuildId, UserId},
        webhook::Webhook,
    },
    prelude::*,
};
use typemap::Key;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;

use markov::{Chain, SizedChainStringIterator};

#[derive(Deserialize)]
struct Config {
    discord_token: String,
    chain_storage_dir: String,
}

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
            if msg.mentions.len() > 0 || msg.mention_everyone {
                let hook = webhook(msg.channel_id, "wide hook".to_owned())
                    .expect("could not make webhook");
                let mut rng = rand::thread_rng();
                for user in msg.mentions {
                    let name = user.nick_in(gid).unwrap_or(user.name.clone());
                    let a_url = user.avatar_url().unwrap_or("https://crates.io/assets/Cargo-Logo-Small-c39abeb466d747f3be442698662c5260.png".to_string());
                    for _ in (0..rng.gen_range(1, 5)) {
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

fn main() {
    let config: Config =
        toml::from_str(&fs::read_to_string("Bizarro.toml").expect("Didn't find Bizarro.toml"))
            .expect("Invalid Bizarro.toml");

    let mut client = Client::new(&config.discord_token, Handler).expect("Error creating client");

    let chains =
        UserChains::load(PathBuf::from(&config.chain_storage_dir)).expect("couldn't load chains");

    let mut everyone_path = PathBuf::from(&config.chain_storage_dir);
    everyone_path.push("everyone.mkc");
    let everychain = Chain::load(everyone_path).expect("Could not load everyone chain");

    {
        let mut data = client.data.lock();
        data.insert::<UserChains>(chains);
        data.insert::<EveryoneChain>(everychain);
    }

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

struct UserChains(HashMap<UserId, Chain<String>>);

impl UserChains {
    fn generate(guild: &GuildId) -> Self {
        let mut map = HashMap::new();
        let none: Option<UserId> = None;
        for member in guild
            .members(Some(10), none)
            .expect("Could not get guild members")
        {
            map.insert(member.user_id(), Chain::new());
        }

        for channel in guild.channels().unwrap() {
            let messages = channel
                .0
                .messages(|g| g.most_recent().limit(100))
                .expect(&format!(
                    "Could not retrieve messages from {}",
                    channel.0.as_u64()
                ));
            let mut last = messages.last().cloned();

            messages.iter().for_each(|m| {
                if let Some(chain) = map.get_mut(&m.author.id) {
                    chain.feed_str(m.content.as_ref());
                }
            });

            while let Some(last_message) = last {
                let messages = channel
                    .0
                    .messages(|g| g.before(last_message).limit(100))
                    .expect("could not get messages");
                last = messages.last().cloned();

                messages.iter().for_each(|m| {
                    if let Some(chain) = map.get_mut(&m.author.id) {
                        chain.feed_str(m.content.as_ref());
                    }
                });
            }
        }

        Self(map)
    }

    // fn users(&self) -> Vec<UserId>

    fn make_message(&self, user: &UserId) -> Option<String> {
        if let Some(chain) = self.0.get(user) {
            Some(chain.generate_str())
        } else {
            None
        }
    }

    fn message_iter(&self, user: &UserId, length: usize) -> Option<SizedChainStringIterator> {
        if let Some(chain) = self.0.get(user) {
            Some(chain.str_iter_for(length))
        } else {
            None
        }
    }

    fn feed(&mut self, user: &UserId, string: &str) {
        if let Some(chain) = self.0.get_mut(user) {
            chain.feed_str(string);
        }
    }

    fn save(&self, path: PathBuf) -> io::Result<()> {
        if !path.is_dir() {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Specified path does not point to an existing directory.",
            ))
        } else {
            for (uid, chain) in self.0.iter() {
                let mut p = path.clone();
                p.push(format!("{}.mkc", uid));
                chain.save(p)?;
            }

            Ok(())
        }
    }

    fn load(path: PathBuf) -> io::Result<Self> {
        if !path.is_dir() {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Specified path does not point to an existing directory.",
            ))
        } else {
            let mut map = HashMap::new();
            for file in path.read_dir()? {
                let path = file?.path();
                if let Some(name) = path.file_stem() {
                    if let Some(name) = name.to_str() {
                        if let Ok(uid) = u64::from_str(name) {
                            map.insert(UserId(uid), Chain::load(path)?);
                        }
                    }
                }
            }

            Ok(Self(map))
        }
    }

    fn count_users(&self) -> usize {
        self.0.keys().count()
    }
}

impl Key for UserChains {
    type Value = Self;
}

fn generate_everyone_chain(guild: &GuildId) -> Chain<String> {
    let mut chain = Chain::new();
    for channel in guild.channels().unwrap() {
        let messages = channel
            .0
            .messages(|g| g.most_recent().limit(100))
            .expect(&format!(
                "Could not retrieve messages from {}",
                channel.0.as_u64()
            ));
        let mut last = messages.last().cloned();

        messages.iter().for_each(|m| {
            chain.feed_str(m.content.as_ref());
        });

        while let Some(last_message) = last {
            let messages = channel
                .0
                .messages(|g| g.before(last_message).limit(100))
                .expect("could not get messages");
            last = messages.last().cloned();

            messages.iter().for_each(|m| {
                chain.feed_str(m.content.as_ref());
            });
        }
    }
    chain
}

struct EveryoneChain;

impl Key for EveryoneChain {
    type Value = Chain<String>;
}

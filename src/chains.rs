use std::{collections::HashMap, io, path::PathBuf, str::FromStr};

use serenity::model::id::{GuildId, UserId};

use markov::{Chain, SizedChainStringIterator};
use typemap::Key;

use crate::config::GenerationParams;

pub struct UserChains(HashMap<UserId, Chain<String>>);

impl UserChains {
    pub fn generate(guild: &GuildId, params: &GenerationParams) -> Self {
        let mut map = HashMap::new();
        let none: Option<UserId> = None;
        for member in guild
            .members(Some(10), none) // TODO: make maximum configurable
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

            messages
                .iter()
                .filter(|m| {
                    let l = m.content.split_whitespace().collect::<Vec<_>>().len();
                    l >= params.min_words && !(params.include_tag_only && l == 1 && m.content.starts_with("<@!"))
                })
                .for_each(|m| {
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

                messages
                    .iter()
                    .filter(|m| {
                        let l = m.content.split_whitespace().collect::<Vec<_>>().len();
                        l >= params.min_words && !(params.include_tag_only && l == 1 && m.content.starts_with("<@!"))
                    })
                    .for_each(|m| {
                        if let Some(chain) = map.get_mut(&m.author.id) {
                            chain.feed_str(m.content.as_ref());
                        }
                    });
            }
        }

        Self(map)
    }

    pub fn user_ids(&self) -> Vec<&UserId> {
        self.0.keys().collect()
    }

    pub fn make_message(&self, user: &UserId) -> Option<String> {
        if let Some(chain) = self.0.get(user) {
            Some(chain.generate_str())
        } else {
            None
        }
    }

    pub fn message_iter(&self, user: &UserId, length: usize) -> Option<SizedChainStringIterator> {
        if let Some(chain) = self.0.get(user) {
            Some(chain.str_iter_for(length))
        } else {
            None
        }
    }

    pub fn feed(&mut self, user: &UserId, string: &str) {
        if let Some(chain) = self.0.get_mut(user) {
            chain.feed_str(string);
        }
    }

    pub fn save(&self, path: &PathBuf) -> io::Result<()> {
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

    pub fn load(path: &PathBuf) -> io::Result<Self> {
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

    pub fn count_users(&self) -> usize {
        self.0.keys().count()
    }
}

impl Key for UserChains {
    type Value = Self;
}

pub fn generate_everyone_chain(guild: &GuildId) -> Chain<String> {
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

pub struct EveryoneChain;

impl Key for EveryoneChain {
    type Value = Chain<String>;
}

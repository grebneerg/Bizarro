use std::{path::PathBuf, str::FromStr};

use serenity::command;
use crate::chains::UserChains;
use crate::config::Config;

command!(ping(ctx, msg) {
    msg.channel_id.say("pong")?;
});

command!(say(ctx, msg) {
    msg.channel_id.say(format!("msg: {}", msg.content))?;
});

command!(regenerate(ctx, msg) {
    let mut status = msg.channel_id.say("Beginning generation...")?;
    let new_chains = UserChains::generate(&msg.guild_id.expect("No guild id"));
    status.edit(|m| m.content("Generation completed. Loading new chains..."))?;
    ctx.data.lock().insert::<UserChains>(new_chains);
    status.edit(|m| m.content("New chains loaded"))?;
});

command!(save(ctx, msg) {
    let data = ctx.data.lock();
    let config = data.get::<Config>().expect("No configuration loaded");
    match data.get::<UserChains>() {
        Some(chains) => match chains.save(&config.chain_storage_dir) {
            Ok(_) => msg.channel_id.say("Saved successfully")?,
            Err(_) => msg.channel_id.say("Failed to save")?,
        },
        None => msg.channel_id.say("No chains appear to be loaded currently")?,
    };
});

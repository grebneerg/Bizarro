use std::{path::PathBuf, str::FromStr};

use crate::chains::UserChains;
use crate::config::Config;
use serenity::command;

use log::{info, warn};

command!(ping(ctx, msg) {
    msg.channel_id.say("pong")?;
});

command!(regenerate(ctx, msg) {
    info!("Regenerating chains");
    let mut status = msg.channel_id.say("Beginning generation...")?;
    let new_chains = UserChains::generate(
        &msg.guild_id.expect("No guild id"),
        &ctx.data.lock().get::<Config>().expect("No configuration loaded").generation
    );
    status.edit(|m| m.content("Generation completed. Loading new chains..."))?;
    ctx.data.lock().insert::<UserChains>(new_chains);
    status.edit(|m| m.content("New chains loaded"))?;
    info!("New chain generation complete");
});

command!(save(ctx, msg) {
    info!("Saving chains");
    let data = ctx.data.lock();
    let config = data.get::<Config>().expect("No configuration loaded");
    match data.get::<UserChains>() {
        Some(chains) => match chains.save(&config.chain_storage_dir) {
            Ok(_) => {
                msg.channel_id.say("Saved successfully")?;
                info!("Chains saved successfully");
            },
            Err(_) => {
                msg.channel_id.say("Failed to save")?;
                warn!("Failed to save chains");
            },
        },
        None => {
            msg.channel_id.say("No chains appear to be loaded currently")?;
            warn!("No chains appear to be loaded currently");
        },
    };
});

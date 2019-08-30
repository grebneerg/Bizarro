use std::{path::PathBuf, str::FromStr};

use crate::chains::UserChains;
use crate::config::Config;
use serenity::framework::standard::{
    macros::{command, group},
    CommandError, CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;

use log::{info, warn};

group!({
    name: "admin",
    options: {},
    commands: [ping, regenerate, save],
});

#[command]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    match msg.channel_id.say(&ctx, "pong") {
        Ok(_) => Ok(()),
        Err(_) => Err(CommandError("Didn't work.".to_string())),
    }
}

#[command]
fn regenerate(ctx: &mut Context, msg: &Message) -> CommandResult {
    info!("Regenerating chains");
    let mut status = msg.channel_id.say(&ctx, "Beginning generation...")?;
    let new_chains = UserChains::generate(
        &ctx,
        &msg.guild_id.expect("No guild id"),
        &ctx.data
            .read()
            .get::<Config>()
            .expect("No configuration loaded")
            .generation,
    );
    status.edit(&ctx, |m| {
        m.content("Generation completed. Loading new chains...")
    })?;
    ctx.data.write().insert::<UserChains>(new_chains);
    status.edit(&ctx, |m| m.content("New chains loaded"))?;
    info!("New chain generation complete");

    Ok(())
}

#[command]
fn save(ctx: &mut Context, msg: &Message) -> CommandResult {
    info!("Saving chains");
    let data = ctx.data.read();
    let config = data.get::<Config>().expect("No configuration loaded");
    match data.get::<UserChains>() {
        Some(chains) => match chains.save(&config.chain_storage_dir) {
            Ok(_) => {
                msg.channel_id.say(&ctx, "Saved successfully")?;
                info!("Chains saved successfully");
                Ok(())
            }
            Err(_) => {
                msg.channel_id.say(&ctx, "Failed to save")?;
                warn!("Failed to save chains");
                Err(CommandError("Failed to save chains".to_string()))
            }
        },
        None => {
            msg.channel_id
                .say(&ctx, "No chains appear to be loaded currently")?;
            warn!("No chains appear to be loaded currently");
            Err(CommandError(
                "No chains appear to be loaded currently".to_string(),
            ))
        }
    }
}

use serenity::all::{Context, CreateCommand, GuildId};
use crate::core::application::commands::command::DiscordBotCommand::{
    AboutProject, Help, Join, Leave, Play, Stop, Found};
use crate::core::application::commands::command::DiscordBotCommand;


pub async fn register_commands(ctx: &Context, guild_id: GuildId) {
    let _ = guild_id.create_command(&ctx.http, build_join()).await;
    let _ = guild_id.create_command(&ctx.http, build_help()).await;
    let _ = guild_id.create_command(&ctx.http, build_leave()).await;
    let _ = guild_id.create_command(&ctx.http, build_play()).await;
    let _ = guild_id.create_command(&ctx.http, build_stop()).await;
    let _ = guild_id.create_command(&ctx.http, build_found()).await;
    let _ = guild_id.create_command(&ctx.http, build_aboutproject()).await;
}

// ========================DISCORD COMMANDS BUILDER========================

pub fn build_help() -> CreateCommand {
    CreateCommand::new(DiscordBotCommand::as_text(Help))
        .description("Show bot help")
}

pub fn build_join() -> CreateCommand {
    CreateCommand::new(DiscordBotCommand::as_text(Join))
        .description("Join your current voice channel")
}

pub fn build_leave() -> CreateCommand {
    CreateCommand::new(DiscordBotCommand::as_text(Leave))
        .description("Leave from voice channel")
}

pub fn build_play() -> CreateCommand {
    CreateCommand::new(DiscordBotCommand::as_text(Play))
        .description("Play music")
}
pub fn build_stop() -> CreateCommand {
    CreateCommand::new(DiscordBotCommand::as_text(Stop))
        .description("Stop playing music")
}

pub fn build_found() -> CreateCommand {
    CreateCommand::new(DiscordBotCommand::as_text(Found))
        .description("Found the music")
}

pub fn build_aboutproject() -> CreateCommand {
    CreateCommand::new(DiscordBotCommand::as_text(AboutProject))
        .description("About 'Zeit' project" )
}
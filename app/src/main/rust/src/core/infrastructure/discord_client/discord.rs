use crate::core::application::commands::command::DiscordBotCommand;
use crate::core::application::commands::command::DiscordBotCommand::{
    Help, Join, Leave, Play, Stop, Found, AboutProject
};
use crate::core::application::commands::handler::DiscordBotCommandHandler;
use crate::core::infrastructure::discord_client::command_regestration::register_commands;
use serenity::{
    async_trait
    ,
    model::{
        application::Interaction,
        gateway::Ready,
        guild::Guild
        ,
    },
    prelude::*,
};
use songbird::{SerenityInit, Songbird};
use std::sync::Arc;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Bot connected as {}", ready.user.name);

        for guild in ready.guilds {
            register_commands(&ctx, guild.id).await;
        }
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        if is_new.unwrap_or(false) {
            register_commands(&ctx, guild.id).await;
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Command(command) = interaction else {
            return;
        };

        match DiscordBotCommand::from_text(command.data.name.as_str()) {
            Join => DiscordBotCommandHandler::handle_join(&ctx, &command).await,
            Help => DiscordBotCommandHandler::handle_help(&ctx, &command).await,
            Leave => DiscordBotCommandHandler::handle_leave(&ctx, &command).await,
            Play => DiscordBotCommandHandler::handle_play(&ctx, &command).await,
            Stop => DiscordBotCommandHandler::handle_stop(&ctx, &command).await,
            Found => DiscordBotCommandHandler::handle_found(&ctx, &command).await,
            AboutProject => DiscordBotCommandHandler::handle_aboutproject(&ctx, &command).await,
            _ => {}
        }
    }
}

pub async fn start(token: String) {
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;
    let songbird: Arc<Songbird> = Songbird::serenity();

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .type_map_insert::<songbird::SongbirdKey>(songbird.clone())
        .register_songbird()
        .await
        .expect("Client error");

    if let Err(err) = client.start().await {
        eprintln!("Client error: {err:?}");
    }
}
//! Minimal Discord voice bot with /join and /help.

use serenity::{
    async_trait,
    builder::{
        CreateCommand,
        CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
    model::{
        application::{CommandInteraction, Interaction},
        gateway::Ready,
        guild::Guild,
        id::{ChannelId, GuildId, UserId},
    },
    prelude::*,
};
use songbird::{Call, Songbird, SerenityInit};
use std::sync::Arc;
use songbird::error::JoinResult;
// ================= COMMANDS =================

const JOIN_COMMAND_NAME: &str = "join";
const HELP_COMMAND_NAME: &str = "help";

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

        match command.data.name.as_str() {
            JOIN_COMMAND_NAME => handle_join(&ctx, &command).await,
            HELP_COMMAND_NAME => handle_help(&ctx, &command).await,
            _ => {}
        }
    }
}

// ================= START =================

pub async fn start(token: String) {
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;
    let songbird = Songbird::serenity();

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

// ================= COMMAND REGISTRATION =================

fn build_join() -> CreateCommand {
    CreateCommand::new(JOIN_COMMAND_NAME)
        .description("Join your current voice channel")
}

fn build_help() -> CreateCommand {
    CreateCommand::new(HELP_COMMAND_NAME)
        .description("Show bot help")
}

async fn register_commands(ctx: &Context, guild_id: GuildId) {
    let _ = guild_id.create_command(&ctx.http, build_join()).await;
    let _ = guild_id.create_command(&ctx.http, build_help()).await;

    println!("Commands registered for guild {guild_id}");
}

// ================= COMMAND HANDLERS =================

async fn handle_join(ctx: &Context, command: &CommandInteraction) {
    let Some(guild_id) = command.guild_id else {
        respond(command, ctx, "Use this command in a server.").await;
        return;
    };

    let user_id = command.user.id;

    let Some(channel_id) = voice_channel(ctx, guild_id, user_id) else {
        respond(command, ctx, "Join a voice channel first.").await;
        return;
    };

    match join(ctx, guild_id, channel_id).await {
        Ok(_) => respond(command, ctx, "Joined voice channel").await,
        Err(_) => respond(command, ctx, "Error with joining").await,
    }
}

async fn handle_help(ctx: &Context, command: &CommandInteraction) {
    let text = "**Available commands:**\n\
    /join — join your voice channel\n\
    /help — show this message";

    respond(command, ctx, text).await;
}

// ================= VOICE =================

fn voice_channel(ctx: &Context, guild_id: GuildId, user_id: UserId) -> Option<ChannelId> {
    guild_id
        .to_guild_cached(&ctx.cache)?
        .voice_states
        .get(&user_id)?
        .channel_id
}

async fn join(
    ctx: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> JoinResult<Arc<Mutex<Call>>> {
    let manager = ctx
        .data
        .read()
        .await
        .get::<songbird::SongbirdKey>()
        .unwrap()
        .clone();

    manager.join(guild_id, channel_id).await
}

// ================= RESPONSE =================

async fn respond(command: &CommandInteraction, ctx: &Context, text: &str) {
    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(text)
            .ephemeral(true),
    );

    let _ = command.create_response(&ctx.http, response).await;
}
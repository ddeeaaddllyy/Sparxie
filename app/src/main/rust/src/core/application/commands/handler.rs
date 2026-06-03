use super::command::TelegramBotCommand;
use serenity::all::{
    CommandInteraction,
    Context,
    UserId
};
use crate::core::infrastructure::discord_client::discord_response::{
    respond,
    public_response
};
use crate::core::utils::voice_module::{
    voice_channel,
    join
};

pub fn handle_telegram_command(command: TelegramBotCommand) -> String {
    match command {
        TelegramBotCommand::Start => {
            "HII".to_string()
        },
        TelegramBotCommand::Help => {
            "Hi this is new bot".to_string()
        },
        TelegramBotCommand::Music => {
            "In Process".to_string()
        },
        TelegramBotCommand::AboutProject => {
            "This is project created by ddeeaaddllyy \
            for git progect Zeit. Follow me in git".to_string()
        }
        TelegramBotCommand::Unknown => {
            "".to_string()
        },
    }
}

pub struct DiscordBotCommandHandler;

impl DiscordBotCommandHandler {

    pub async fn handle_help(ctx: &Context, command: &CommandInteraction) {
        let text: &str = "**Available commands:**\n\
                         /join — join your voice channel\n\
                         /help — show this message";

        respond(command, ctx, text).await;
    }

    pub async fn handle_join(ctx: &Context, command: &CommandInteraction) {
        let Some(guild_id) = command.guild_id else {
            respond(command, ctx, "Use this command in a server.").await;
            return;
        };

        let user_id: UserId = command.user.id;

        let Some(channel_id) = voice_channel(ctx, guild_id, user_id) else {
            respond(command, ctx, "Join a voice channel first.").await;
            return;
        };

        match join(ctx, guild_id, channel_id).await {
            Ok(_) => respond(command, ctx, "Joined voice channel").await,
            Err(e) => {
                println!("Join error: {:?}", e);
                respond(command, ctx, "Error with joining").await
            }
        }
    }

    pub async fn handle_leave(ctx: &Context, command: &CommandInteraction) {
        println!("Leaving voice channel");
    }

    pub async fn handle_play(ctx: &Context, command: &CommandInteraction) {}

    pub async fn handle_stop(ctx: &Context, command: &CommandInteraction) {}

    pub async fn handle_found(ctx: &Context, command: &CommandInteraction) {}

    pub async fn handle_aboutproject(ctx: &Context, command: &CommandInteraction) {
        let text = "this is new project. Enjoy sweetheart";

        public_response(command, ctx, text).await;
    }
}
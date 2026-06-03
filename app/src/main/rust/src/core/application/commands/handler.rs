use super::command::TelegramBotCommand;
use crate::core::infrastructure::discord_client::discord_response::{
    local_response,
    public_response
};
use crate::core::utils::voice_module::voice_channel;
use songbird::get as songbird_get;
use serenity::all::{
    CommandInteraction
    ,Context
    ,CreateInteractionResponse
    ,CreateInteractionResponseMessage
};
use serenity::builder::EditInteractionResponse;

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

        local_response(command, ctx, text).await;
    }

    pub async fn handle_join(ctx: &Context, command: &CommandInteraction) {
        let Some(guild_id) = command.guild_id else {
            local_response(command, ctx, "Use this command in a server.").await;
            return;
        };

        let _ = command.create_response(
            &ctx.http,
            CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true)
            )
        ).await;

        let user_id = command.user.id;

        let Some(channel_id) = voice_channel(ctx, guild_id, user_id) else {
            let _ = command.edit_response(
                &ctx.http,
                EditInteractionResponse::new().content("Сначала зайди в голосовой канал")
            ).await;
            return;
        };

        let Some(manager) = songbird_get(ctx).await else {
            let _ = command.edit_response(
                &ctx.http,
                EditInteractionResponse::new().content("Voice system not initialized")
            ).await;
            return;
        };

        if manager.get(guild_id).is_some() {
            let _ = command.edit_response(
                &ctx.http,
                EditInteractionResponse::new().content("Already in a voice channel")
            ).await;
            return;
        }

        let result = manager.join(guild_id, channel_id).await;

        match result {
            Ok(_) => {
                let _ = command.edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().content("Join")
                ).await;
            }
            Err(e) => {
                println!("Join error: {:?}", e);
                let _ = command.edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().content("Error")
                ).await;
            }
        }
    }

    pub async fn handle_leave(ctx: &Context, command: &CommandInteraction) {
        let Some(guild_id) = command.guild_id else {
            local_response(command, ctx, "Use this command in a server.").await;
            return;
        };

        let manager = songbird_get(ctx).await;

        let Some(manager) = manager else {
            local_response(command, ctx, "Songbird not initialized").await;
            return;
        };

        if manager.get(guild_id).is_some() {
            match manager.remove(guild_id).await {
                Ok(_) => local_response(command, ctx, "Left voice channel").await,
                Err(e) => {
                    local_response(command, ctx, &format!("Error leaving: {:?}", e)).await
                }
            }
        } else {
            local_response(command, ctx, "Not in a voice channel").await;
        }
    }

    pub async fn handle_play(_ctx: &Context, _command: &CommandInteraction) {}

    pub async fn handle_stop(_ctx: &Context, _command: &CommandInteraction) {}

    pub async fn handle_found(_ctx: &Context, _command: &CommandInteraction) {}

    pub async fn handle_aboutproject(ctx: &Context, command: &CommandInteraction) {
        let text: &str = "lol";

        public_response(command, ctx, text).await;
    }
}
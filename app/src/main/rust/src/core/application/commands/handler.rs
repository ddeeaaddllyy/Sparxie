use super::command::TelegramBotCommand;
use crate::core::infrastructure::discord_client::discord_response::{
    local_response, public_response,
};
use crate::core::utils::voice_module::voice_channel;
use reqwest::Client as ReqwestClient;
use serenity::all::{
    CommandDataOptionValue, CommandInteraction, Context, CreateInteractionResponse,
    CreateInteractionResponseMessage, EditInteractionResponse, GuildId,
};
use songbird::events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};
use songbird::input::{AuxMetadata, Compose, YoutubeDl};
use songbird::{get as songbird_get, Call};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Handles legacy Telegram commands.
///
/// This function is intentionally separate from the Discord slash-command
/// handlers below. Telegram commands are parsed from plain text, while Discord
/// commands arrive as strongly typed interactions with options and response
/// tokens.
pub fn handle_telegram_command(command: TelegramBotCommand) -> String {
    match command {
        TelegramBotCommand::Start => "HII".to_string(),
        TelegramBotCommand::Help => "Hi this is new bot".to_string(),
        TelegramBotCommand::Music => "In Process".to_string(),
        TelegramBotCommand::AboutProject => "This is project created by ddeeaaddllyy \
            for git project Zeit. Follow me in git"
            .to_string(),
        TelegramBotCommand::Unknown => "".to_string(),
    }
}

/// Namespace for Discord slash-command handlers.
///
/// The struct has no fields because all command state is stored either in
/// Discord/Songbird runtime objects or in local variables for a single
/// interaction. This keeps command handling predictable across reconnects.
pub struct DiscordBotCommandHandler;

impl DiscordBotCommandHandler {
    /// Shows a short command list to the user who invoked `/help`.
    pub async fn handle_help(ctx: &Context, command: &CommandInteraction) {
        let text = "**Available commands:**\n\
            /join - join your current voice channel\n\
            /play <query> - play a YouTube URL or search result\n\
            /stop - stop playback and clear the queue\n\
            /leave - leave the voice channel";

        local_response(command, ctx, text).await;
    }

    /// Connects the bot to the command author's current voice channel.
    ///
    /// `/join` does not start playback. It only creates or moves the Songbird
    /// voice connection so that `/play` can later enqueue audio.
    pub async fn handle_join(ctx: &Context, command: &CommandInteraction) {
        let Some(guild_id) = command.guild_id else {
            local_response(command, ctx, "Use this command in a server.").await;
            return;
        };

        if !defer_ephemeral(command, ctx).await {
            return;
        }

        let Some(channel_id) = voice_channel(ctx, guild_id, command.user.id) else {
            edit_response(command, ctx, "First, join a voice channel.").await;
            return;
        };

        let Some(manager) = songbird_get(ctx).await else {
            edit_response(command, ctx, "Voice system is not initialized.").await;
            return;
        };

        match manager.join(guild_id, channel_id).await {
            Ok(_) => edit_response(command, ctx, "Joined your voice channel.").await,
            Err(err) => {
                edit_response(
                    command,
                    ctx,
                    &format!("Could not join the voice channel: {err:?}"),
                )
                .await;
            }
        }
    }

    /// Disconnects the bot from the current guild voice channel.
    pub async fn handle_leave(ctx: &Context, command: &CommandInteraction) {
        let Some(guild_id) = command.guild_id else {
            local_response(command, ctx, "Use this command in a server.").await;
            return;
        };

        let Some(manager) = songbird_get(ctx).await else {
            local_response(command, ctx, "Songbird is not initialized.").await;
            return;
        };

        if manager.get(guild_id).is_some() {
            match manager.remove(guild_id).await {
                Ok(_) => local_response(command, ctx, "Left the voice channel.").await,
                Err(err) => {
                    local_response(command, ctx, &format!("Error while leaving: {err:?}")).await
                }
            }
        } else {
            local_response(command, ctx, "I am not in a voice channel.").await;
        }
    }

    /// Plays a YouTube URL or search query in the guild voice channel.
    ///
    /// The command flow is deliberately defensive:
    /// 1. Defer the interaction immediately because `yt-dlp` can take longer
    ///    than Discord's short initial response window.
    /// 2. Reuse the existing Songbird call, or join the author's voice channel
    ///    automatically if the bot is not connected yet.
    /// 3. Ask `yt-dlp` for metadata before queueing. This catches missing
    ///    `yt-dlp`, unavailable videos, and empty searches before the user gets
    ///    a misleading "queued" message.
    /// 4. Queue with Songbird's built-in queue. `play_input` mixes tracks
    ///    together; `enqueue_input` plays one track after another.
    pub async fn handle_play(ctx: &Context, command: &CommandInteraction) {
        let Some(guild_id) = command.guild_id else {
            local_response(command, ctx, "Use this command in a server.").await;
            return;
        };

        let Some(query) = command_string_option(command, "query") else {
            local_response(
                command,
                ctx,
                "Pass a YouTube URL or search query after /play.",
            )
            .await;
            return;
        };

        let query = query.trim();
        if query.is_empty() {
            local_response(command, ctx, "The /play query cannot be empty.").await;
            return;
        }

        if !defer_public(command, ctx).await {
            return;
        }

        let call = match get_or_join_voice_call(ctx, command, guild_id).await {
            Ok(call) => call,
            Err(message) => {
                edit_response(command, ctx, &message).await;
                return;
            }
        };

        let mut source = youtube_source(query.to_owned());
        let metadata = match source.aux_metadata().await {
            Ok(metadata) => metadata,
            Err(err) => {
                edit_response(
                    command,
                    ctx,
                    &format!(
                        "Could not load this YouTube track. Make sure yt-dlp can access it. Details: {err:?}"
                    ),
                )
                .await;
                return;
            }
        };

        let title = track_title(&metadata, query);
        let position = {
            let mut handler = call.lock().await;
            let position = handler.queue().len() + 1;
            let track = handler.enqueue_input(source.into()).await;
            attach_track_diagnostics(&track, title.clone());
            position
        };

        edit_response(command, ctx, &format!("Queued #{position}: {title}")).await;
    }

    /// Stops playback and clears the Songbird queue for this guild.
    pub async fn handle_stop(ctx: &Context, command: &CommandInteraction) {
        let Some(guild_id) = command.guild_id else {
            local_response(command, ctx, "Use this command in a server.").await;
            return;
        };

        let Some(manager) = songbird_get(ctx).await else {
            local_response(command, ctx, "Songbird is not initialized.").await;
            return;
        };

        let Some(call) = manager.get(guild_id) else {
            local_response(command, ctx, "I am not in a voice channel.").await;
            return;
        };

        {
            let mut handler = call.lock().await;
            handler.queue().stop();
            handler.stop();
        }

        local_response(command, ctx, "Stopped playback and cleared the queue.").await;
    }

    /// Placeholder for the existing `/found` command.
    pub async fn handle_found(ctx: &Context, command: &CommandInteraction) {
        local_response(command, ctx, "This command is not implemented yet.").await;
    }

    /// Shows a short project message.
    pub async fn handle_aboutproject(ctx: &Context, command: &CommandInteraction) {
        public_response(command, ctx, "Zeit project Discord music bot.").await;
    }
}

/// Reads a string option from a Discord slash command.
///
/// Matching by option name is more robust than relying on positional order; it
/// keeps `/play` working even if more options are added later.
fn command_string_option<'a>(command: &'a CommandInteraction, name: &str) -> Option<&'a str> {
    command
        .data
        .options
        .iter()
        .find(|option| option.name == name)
        .and_then(|option| match &option.value {
            CommandDataOptionValue::String(value) => Some(value.as_str()),
            CommandDataOptionValue::Autocomplete { value, .. } => Some(value.as_str()),
            _ => None,
        })
}

/// Gets the existing Songbird call or joins the command author's voice channel.
///
/// This lets `/play` work both after `/join` and as a one-step command. If the
/// bot is not connected yet, the author must be in a voice channel so the target
/// channel can be inferred safely.
async fn get_or_join_voice_call(
    ctx: &Context,
    command: &CommandInteraction,
    guild_id: GuildId,
) -> Result<Arc<Mutex<Call>>, String> {
    let Some(manager) = songbird_get(ctx).await else {
        return Err("Songbird is not initialized.".to_string());
    };

    if let Some(call) = manager.get(guild_id) {
        return Ok(call);
    }

    let Some(channel_id) = voice_channel(ctx, guild_id, command.user.id) else {
        return Err("Join a voice channel first, then run /play.".to_string());
    };

    manager
        .join(guild_id, channel_id)
        .await
        .map_err(|err| format!("Could not join your voice channel: {err:?}"))
}

/// Creates a Songbird YouTube source for either a URL or a search query.
///
/// `YoutubeDl::new` treats its input as a direct URL. `YoutubeDl::new_search`
/// asks yt-dlp to search YouTube and pick the first result. Both forms are lazy:
/// the actual media stream is opened later by Songbird, while metadata can be
/// requested explicitly before queueing.
fn youtube_source(query: String) -> YoutubeDl<'static> {
    let client = ReqwestClient::builder()
        .timeout(std::time::Duration::from_secs(60))
        .pool_idle_timeout(None)
        .build()
        .unwrap();

    let source = if looks_like_url(&query) {
        YoutubeDl::new(client, query)
    } else {
        YoutubeDl::new_search(client, query)
    };

    source.user_args(vec![
        "--extractor-args".into(),
        "youtube:player_client=android".into(),

        "--force-ipv4".into(),

        "--format".into(),
        "bestaudio".into(),

        "--no-playlist".into(),

        "--no-warnings".into(),
    ])
}

/// Returns true when a `/play` query looks like a direct URL.
fn looks_like_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

/// Builds a human-readable track label from yt-dlp metadata.
fn track_title(metadata: &AuxMetadata, fallback: &str) -> String {
    metadata
        .title
        .as_deref()
        .or(metadata.track.as_deref())
        .unwrap_or(fallback)
        .to_string()
}

/// Defers an interaction with a private response.
async fn defer_ephemeral(command: &CommandInteraction, ctx: &Context) -> bool {
    let response =
        CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new().ephemeral(true));

    if let Err(err) = command.create_response(&ctx.http, response).await {
        eprintln!("Failed to defer ephemeral Discord response: {err:?}");
        return false;
    }

    true
}

/// Defers an interaction with a public response.
async fn defer_public(command: &CommandInteraction, ctx: &Context) -> bool {
    let response = CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new());

    if let Err(err) = command.create_response(&ctx.http, response).await {
        eprintln!("Failed to defer public Discord response: {err:?}");
        return false;
    }

    true
}

/// Edits the initial interaction response after a deafer.
async fn edit_response(command: &CommandInteraction, ctx: &Context, content: &str) {
    if let Err(err) = command
        .edit_response(&ctx.http, EditInteractionResponse::new().content(content))
        .await
    {
        eprintln!("Failed to edit Discord interaction response: {err:?}");
    }
}

/// Attaches console diagnostics to a queued Songbird track.
///
/// `enqueue_input` only means "the track was accepted by the queue". The actual
/// network stream is opened and parsed later by Songbird's audio worker. These
/// event hooks make that later phase visible in the console, which is essential
/// for debugging cases where Discord shows the bot in voice but no audio is
/// heard.
fn attach_track_diagnostics(track: &songbird::tracks::TrackHandle, title: String) {
    let playable_title = title.clone();
    if let Err(err) = track.add_event(
        Event::Track(TrackEvent::Playable),
        TrackLog {
            title: playable_title,
            event_name: "playable",
        },
    ) {
        eprintln!("Failed to attach Songbird playable event: {err:?}");
    }

    let error_title = title.clone();
    if let Err(err) = track.add_event(
        Event::Track(TrackEvent::Error),
        TrackLog {
            title: error_title,
            event_name: "error",
        },
    ) {
        eprintln!("Failed to attach Songbird error event: {err:?}");
    }

    if let Err(err) = track.add_event(
        Event::Track(TrackEvent::End),
        TrackLog {
            title,
            event_name: "end",
        },
    ) {
        eprintln!("Failed to attach Songbird end event: {err:?}");
    }
}

/// Small Songbird event logger for one queued track.
struct TrackLog {
    /// Human-readable track title resolved from yt-dlp metadata.
    title: String,
    /// Short event label printed to the console.
    event_name: &'static str,
}

#[serenity::async_trait]
impl VoiceEventHandler for TrackLog {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        match ctx {
            EventContext::Track(states) => {
                for (state, _) in *states {
                    println!(
                        "Songbird track {}: '{}' state={:?}",
                        self.event_name, self.title, state.playing
                    );
                }
            }
            other => {
                println!(
                    "Songbird track {}: '{}' context={other:?}",
                    self.event_name, self.title
                );
            }
        }

        Some(Event::Cancel)
    }
}

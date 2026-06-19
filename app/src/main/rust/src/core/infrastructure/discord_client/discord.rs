use crate::core::application::commands::command::DiscordBotCommand;
use crate::core::application::commands::command::DiscordBotCommand::{
    AboutProject, Found, Help, Join, Leave, Play, Stop,
};
use crate::core::application::commands::handler::DiscordBotCommandHandler;
use crate::core::infrastructure::discord_client::command_regestration::register_commands;
use serenity::{
    async_trait,
    model::{application::Interaction, gateway::Ready, guild::Guild},
    prelude::*,
};
use songbird::SerenityInit;

/// Discord gateway event handler.
///
/// The handler itself is intentionally stateless. Serenity owns the HTTP client,
/// gateway cache, and shared typemap, while Songbird stores its voice manager in
/// that typemap when [`SerenityInit::register_songbird`] is called in [`start`].
/// Keeping this type empty makes every command depend only on the fresh
/// [`Context`] that Discord gives us for each event.
pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    /// Registers slash commands when the bot is added to a new guild.
    ///
    /// Guild-scoped commands update almost immediately, which is much nicer for
    /// local testing than global commands that can take up to an hour to refresh
    /// in the Discord client.
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        if is_new.unwrap_or(false) {
            register_commands(&ctx, guild.id).await;
        }
    }

    /// Registers slash commands for every guild received in the READY payload.
    ///
    /// Discord sends this event after the gateway session is established. At
    /// this point the bot can safely create/update commands and accept voice
    /// state events required by `/join` and `/play`.
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Bot connected as {}", ready.user.name);

        for guild in ready.guilds {
            register_commands(&ctx, guild.id).await;
        }
    }

    /// Routes slash command interactions to their command-specific handler.
    ///
    /// The music commands are implemented in `commands::handler` so the Discord
    /// gateway adapter stays thin: it only translates Discord's command name
    /// into this project's [`DiscordBotCommand`] enum and then delegates.
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

/// Starts the Discord bot and blocks until the gateway client stops.
///
/// Required intents:
/// - `GUILDS` receives guild and slash command interaction events.
/// - `GUILD_VOICE_STATES` lets the cache identify the voice channel of the user
///   who invoked `/join` or `/play`.
///
/// `MESSAGE_CONTENT` is deliberately not requested because this bot uses slash
/// commands instead of reading text messages.
pub async fn start(token: String) {
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .register_songbird()
        .await
        .expect("Client error");

    if let Err(err) = (&mut client).start().await {
        eprintln!("Client error: {err:?}");
    }
}

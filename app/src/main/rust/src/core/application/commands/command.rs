
pub enum TelegramBotCommand {
    Start,
    Help,
    Music,
    AboutProject,
    Unknown
}

impl TelegramBotCommand {
    pub fn from_text(text: &str) -> Self {
        match text {
            "/start" => TelegramBotCommand::Start,
            "/help" => TelegramBotCommand::Help,
            "/music" => TelegramBotCommand::Music,
            "/aboutproject" => TelegramBotCommand::AboutProject,
            _ => TelegramBotCommand::Unknown
        }
    }
}

pub enum DiscordBotCommand {
    Help,
    Join,
    Leave,
    Play,
    Stop,
    Found,
    AboutProject,
    Unknown
}

impl DiscordBotCommand {
    pub fn from_text(text: &str) -> Self {
        match text {
            "/help" => DiscordBotCommand::Help,
            "join" => DiscordBotCommand::Join,
            "/leave" => DiscordBotCommand::Leave,
            "/play" => DiscordBotCommand::Play,
            "/stop" => DiscordBotCommand::Stop,
            "/found" => DiscordBotCommand::Found,
            "/aboutproject" => DiscordBotCommand::AboutProject,
            _ => DiscordBotCommand::Unknown
        }
    }
    pub fn as_text(command: DiscordBotCommand) -> &'static str {
        match command {
            DiscordBotCommand::Help => "help",
            DiscordBotCommand::Join => "join",
            DiscordBotCommand::Leave => "leave",
            DiscordBotCommand::Play => "play",
            DiscordBotCommand::Stop => "stop",
            DiscordBotCommand::Found => "found",
            DiscordBotCommand::AboutProject => "aboutproject",
            DiscordBotCommand::Unknown => "unknown"
        }
    }
}
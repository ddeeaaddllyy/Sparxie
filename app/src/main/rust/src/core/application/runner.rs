use std::io::{self, Write};
use crate::core::config::settings::Settings;
use crate::core::infrastructure::discord_client::discord;
use crate::core::infrastructure::telegram_client::telegram;
use crate::core::utils::debug::log;

pub async fn run() {
    dotenv::dotenv().ok();

    let settings: Settings = Settings::new();

    println!("1 - Telegram");
    println!("2 - Discord");
    println!("3 - Both");
    print!("Choose: ");
    io::stdout().flush().unwrap();

    let mut input: String = String::new();
    io::stdin().read_line(&mut input).unwrap();

    match input.trim() {
        "1" => {
            log("Starting Telegram...");
            telegram::start(settings.telegram_bot_token).await;
        }
        "2" => {
            log("Starting Discord...");
            discord::start(settings.discord_bot_token).await;
        }
        "3" => {
            log("Starting BOTH...");

            let t = tokio::spawn(telegram::start(settings.telegram_bot_token));
            let d = tokio::spawn(discord::start(settings.discord_bot_token));

            let _ = tokio::join!(t, d);
        }
        _ => {
            log("Invalid input");
        }
    }
}

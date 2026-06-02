use crate::core::utils::debug::log;
use crate::core::application::commands::{handler};
use crate::core::application::commands::command::TelegramBotCommand;
use crate::core::infrastructure::telegram_client::telegram_responce::{Response, Update};
use reqwest::Proxy;

pub async fn start(token: String) {
    let _proxy = Proxy::all("http://127.0.0.1:1080").unwrap();
    let client = reqwest::Client::builder()
        // .proxy(proxy)
        .build()
        .unwrap();
    let mut offset: i64 = 0;

    log("Telegram bot started");

    loop {
        let url = format!(
            "https://api.telegram.org/bot{}/getUpdates?timeout=10&offset={}",
            token, offset
        );

        let resp = client.get(&url).send().await;

        if resp.is_err() {
            let req_error = format!("Request error: {}", resp.err().unwrap());
            log(&*req_error);
            continue;
        }

        let text: String = resp.unwrap().text().await.unwrap();

        let parsed: Result<Response<Update>, _> = serde_json::from_str(&text);

        if parsed.is_err() {
            log("JSON parse error");
            continue;
        }

        let data: Response<Update> = parsed.unwrap();

        if !data.ok {
            log("Telegram API error");
            continue;
        }

        let updates: Vec<Update> = data.result.unwrap_or_default();

        for update in updates {
            offset = update.update_id + 1;

            if let Some(msg) = update.message {
                if let Some(text) = msg.text {
                    let command: TelegramBotCommand = TelegramBotCommand::from_text(&text);
                    let answer: String = handler::handle_telegram_command(command);

                    send_message(&client, &token, msg.chat.id, &answer).await;
                }
            }
        }
    }
}

async fn send_message(client: &reqwest::Client, token: &str, chat_id: i64, text: &str) {
    let url: String = format!("https://api.telegram.org/bot{}/sendMessage", token);

    let _ = client
        .post(&url)
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text": text
        }))
        .send()
        .await;
}
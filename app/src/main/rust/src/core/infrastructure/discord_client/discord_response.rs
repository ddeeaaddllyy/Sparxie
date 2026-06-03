use serenity::all::{CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage};

pub async fn respond(command: &CommandInteraction, ctx: &Context, text: &str) {
    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(text)
            .ephemeral(true),
    );

    let _ = command.create_response(&ctx.http, response).await;
}

pub async fn public_response(command: &CommandInteraction, ctx: &Context, text: &str) {
    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
        .content(text)
    );
    
    let _ = command.create_response(&ctx.http, response).await;
}
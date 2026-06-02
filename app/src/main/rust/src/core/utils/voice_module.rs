use std::sync::Arc;
use serenity::all::{ChannelId, Context, GuildId, UserId};
use songbird::Call;
use songbird::error::JoinResult;
use tokio::sync::Mutex;

pub fn voice_channel(ctx: &Context, guild_id: GuildId, user_id: UserId) -> Option<ChannelId> {
    guild_id
        .to_guild_cached(&ctx.cache)?
        .voice_states
        .get(&user_id)?
        .channel_id
}

pub async fn join(
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
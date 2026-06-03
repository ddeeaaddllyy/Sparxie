use serenity::all::{ChannelId, Context, GuildId, UserId};

pub fn voice_channel(ctx: &Context, guild_id: GuildId, user_id: UserId) -> Option<ChannelId> {
    guild_id
        .to_guild_cached(&ctx.cache)?
        .voice_states
        .get(&user_id)?
        .channel_id
}
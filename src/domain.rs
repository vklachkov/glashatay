use crate::vk_api;

#[derive(Clone, Copy, Debug)]
pub struct ChannelEntryId(pub i32);

#[derive(Clone, Debug)]
pub struct ChannelInfo {
    /// Идентификатор Telegram канала, куда будут отправляться посты.
    pub tg_channel: TelegramChannelId,

    /// Идентификатор стены ВК, откуда будут читаться публикации.
    pub vk_public_id: String,

    /// Интервал проверки на новые записи в секундах.
    pub poll_interval: chrono::Duration,

    /// Время последней проверки на новые публикации.
    pub last_poll_datetime: Option<chrono::DateTime<chrono::Utc>>,

    /// Идентификатор последней публикации на стене.
    pub vk_last_post: Option<vk_api::PostId>,
}

#[derive(Clone, Copy, Debug)]
pub struct TelegramChannelId(pub i64);

#[derive(Clone, Debug)]
pub struct TelegramPost {
    pub channel_id: TelegramChannelId,
    pub text: String,
}

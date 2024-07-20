use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelEntryId(pub i32);

#[derive(Clone, Debug)]
pub struct ChannelInfo {
    /// Идентификатор Telegram канала, куда будут отправляться посты.
    pub tg_channel: TelegramChannelId,

    /// Идентификатор стены ВК, откуда будут читаться публикации.
    pub vk_public_id: VkId,

    /// Интервал проверки на новые записи в секундах.
    pub poll_interval: chrono::Duration,

    /// Время последней проверки на новые публикации.
    pub last_poll_datetime: Option<chrono::DateTime<chrono::Utc>>,

    /// Время публикации последней записи на стене.
    pub last_post_datetime: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone, Debug)]
pub struct VkId(pub String);

impl Display for VkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("https://vk.com/")?;
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TelegramChannelId(pub i64);

impl Display for TelegramChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = self.0.to_string();
        let normal_id = id.strip_prefix("-100").unwrap_or(&id);

        f.write_str("https://t.me/c/")?;
        f.write_str(normal_id)
    }
}

pub struct TelegramPost {
    pub channel_id: TelegramChannelId,
    pub text: String,
    pub photos: Vec<TelegramPostPhoto>,
    pub is_pinned: bool,
}

#[derive(Clone)]
pub struct TelegramPostPhoto {
    pub bytes: Vec<u8>,
    pub description: String,
}

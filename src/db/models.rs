use crate::domain;
use diesel::prelude::*;

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = super::schema::channels)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewChannel {
    /// Идентификатор Telegram канала, куда будут отправляться посты.
    pub tg_channel_id: i64,

    /// Идентификатор стены ВК, откуда будут читаться публикации.
    pub vk_public_id: String,

    /// Интервал проверки на новые записи в секундах.
    pub poll_interval_secs: i32,

    /// Время последней проверки на новые публикации.
    pub last_poll_timestamp: Option<i64>,

    /// Время публикации последней записи на стене.
    pub last_post_timestamp: Option<i64>,
}

impl From<domain::ChannelInfo> for NewChannel {
    fn from(info: domain::ChannelInfo) -> Self {
        Self {
            tg_channel_id: info.tg_channel.0,
            vk_public_id: info.vk_public_id,
            poll_interval_secs: info
                .poll_interval
                .num_seconds()
                .try_into()
                .unwrap_or(i32::MAX),
            last_poll_timestamp: info.last_poll_datetime.map(|dt| dt.timestamp()),
            last_post_timestamp: info.last_post_datetime.map(|dt| dt.timestamp()),
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = super::schema::channels)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Channel {
    pub id: i32,

    /// Идентификатор Telegram канала, куда будут отправляться посты.
    pub tg_channel_id: i64,

    /// Идентификатор стены ВК, откуда будут читаться публикации.
    pub vk_public_id: String,

    /// Интервал проверки на новые записи в секундах.
    pub poll_interval_secs: i32,

    /// Время последней проверки на новые публикации.
    pub last_poll_timestamp: Option<i64>,

    /// Время публикации последней записи на стене.
    pub last_post_timestamp: Option<i64>,
}

impl From<Channel> for domain::ChannelInfo {
    fn from(ch: Channel) -> Self {
        Self {
            tg_channel: domain::TelegramChannelId(ch.tg_channel_id),
            vk_public_id: ch.vk_public_id,
            poll_interval: chrono::Duration::seconds(ch.poll_interval_secs.into()),
            last_poll_datetime: ch.last_poll_timestamp.map(|ts| {
                chrono::DateTime::from_timestamp(ts, 0)
                    .expect("last_poll_timestamp should be correct timestamp")
            }),
            last_post_datetime: ch.last_post_timestamp.map(|ts| {
                chrono::DateTime::from_timestamp(ts, 0)
                    .expect("last_post_timestamp should be correct timestamp")
            }),
        }
    }
}

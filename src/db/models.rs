use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::channels)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Post {
    /// Идентификатор Telegram канала, куда будут отправляться посты.
    pub tg_channel_id: i64,

    /// Идентификатор стены ВК, откуда будут читаться публикации.
    pub vk_public_id: String,

    /// Интервал проверки на новые записи в секундах.
    pub poll_interval_secs: i32,

    /// Время последней проверки на новые публикации.
    pub last_poll_timestamp: Option<i64>,

    /// Идентификатор последней публикации на стене.
    pub last_post_id: Option<i64>,
}

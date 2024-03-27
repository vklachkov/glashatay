CREATE TABLE channels (
    id INTEGER PRIMARY KEY NOT NULL,
    
    -- Идентификатор Telegram канала, куда будут отправляться посты.
    tg_channel_id BIGINT NOT NULL,
    
    -- Идентификатор стены ВК, откуда будут читаться публикации.
    vk_public_id TEXT NOT NULL,
    
    -- Интервал проверки на новые записи в секундах.
    poll_interval_secs INTEGER NOT NULL,
    
    -- Время последней проверки на новые публикации.
    last_poll_timestamp BIGINT,
    
    -- Идентификатор последней публикации на стене.
    last_post_id BIGINT
);
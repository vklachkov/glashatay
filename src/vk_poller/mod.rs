use crate::{
    db::Db,
    domain::{ChannelEntryId, ChannelInfo},
};
use std::{sync::Arc, time::Duration};

pub struct VkPoller {
    db: Arc<Db>,
}

impl VkPoller {
    /// Читает список каналов из базы данных и запускает процесс опроса.
    pub async fn new(db: Db) -> Self {
        let this = Self { db: Arc::new(db) };

        for (id, info) in this.db.get_channels().await {
            this.spawn_poller(id, info);
        }

        this
    }

    /// Сохраняет канал и запускает для него процесс опроса.
    pub async fn create(&self, info: ChannelInfo) {
        let id = self.db.new_channel(&info).await;
        self.spawn_poller(id, info);
    }

    fn spawn_poller(&self, database_id: ChannelEntryId, info: ChannelInfo) {
        tokio::spawn({
            let database = self.db.clone();
            Self::poller(database, database_id, info)
        });
    }

    async fn poller(database: Arc<Db>, id: ChannelEntryId, mut info: ChannelInfo) {
        loop {
            let should_poll = info
                .last_poll_datetime
                .map(|dt| info.poll_interval < (chrono::Utc::now() - dt))
                .unwrap_or(true);

            if !should_poll {
                tokio::time::sleep(Duration::from_millis(1000)).await;
                continue;
            }

            if let Some(post_id) = info.vk_last_post {
                /*
                TODO:
                1. Получить 5 постов со стены
                2. Если в них есть пост с post_id, то взять посты до post_id.
                3. Если нет, сохранить и запросить ещё 5, перейти к п. 2.
                4. Полученный список постов преобразовать в формат Telegram.
                5. Пройтись по каждому посту и отправить его.
                FIXME: Какой должен быть формат поста?
                */
            } else {
                /*
                TODO:
                1. Получить последний не закреплённый пост со стены.
                2. Обновить info
                */
            }

            database.update_channel(id, &info).await;
        }
    }
}

mod poller;

use crate::{
    config,
    db::Db,
    domain::{ChannelEntryId, ChannelInfo},
};
use std::sync::Arc;

use self::poller::VkPoller;

pub struct VkPollManager {
    config: Arc<config::Config>,
    db: Arc<Db>,
}

impl VkPollManager {
    /// Читает список каналов из базы данных и запускает процесс опроса.
    pub async fn new(config: config::Config, db: Db) -> Self {
        let this = Self {
            config: Arc::new(config),
            db: Arc::new(db),
        };

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
        tokio::spawn(VkPoller::new(self.config.clone(), self.db.clone(), database_id, info).run());
    }
}

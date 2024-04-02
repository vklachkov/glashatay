mod converter;
mod poller;

use crate::{
    config::Config,
    db::Db,
    domain::{ChannelEntryId, ChannelInfo},
};
use poller::VkPoller;
use std::sync::Arc;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[derive(Clone)]
pub struct VkPollManager {
    config: Arc<Config>,
    db: Db,
    bot: teloxide::Bot,
    tracker: TaskTracker,
    cancellation_token: CancellationToken,
}

impl VkPollManager {
    /// Читает список каналов из базы данных и запускает процесс опроса.
    pub fn new(
        config: Arc<Config>,
        db: Db,
        bot: teloxide::Bot,
        tracker: TaskTracker,
        token: CancellationToken,
    ) -> Self {
        Self {
            config,
            db,
            bot,
            tracker,
            cancellation_token: token,
        }
    }

    pub async fn run(self) {
        for (id, info) in self.db.get_channels().await {
            self.spawn_poller(id, info);
        }
    }

    /// Сохраняет канал и запускает для него процесс опроса.
    pub async fn create(&self, info: ChannelInfo) {
        let id = self.db.new_channel(&info).await;
        self.spawn_poller(id, info);
    }

    fn spawn_poller(&self, database_id: ChannelEntryId, info: ChannelInfo) {
        self.tracker.spawn(
            VkPoller::new(
                self.config.clone(),
                self.db.clone(),
                database_id,
                info,
                self.bot.clone(),
                self.cancellation_token.clone(),
            )
            .run(),
        );
    }
}

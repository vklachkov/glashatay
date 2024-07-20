mod converter;
mod poller;

use crate::{
    config::Config,
    db::Db,
    domain::{ChannelEntryId, ChannelInfo},
};
use poller::VkPoller;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[derive(Clone)]
pub struct VkPollManager {
    config: Arc<Config>,
    db: Db,
    bot: teloxide::Bot,
    tracker: TaskTracker,
    stop_tokens: Arc<Mutex<HashMap<ChannelEntryId, CancellationToken>>>,
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
            stop_tokens: Default::default(),
            cancellation_token: token,
        }
    }

    pub async fn run(self) {
        for (id, info) in self.db.get_channels().await {
            self.spawn_poller(id, info).await;
        }
    }

    /// Возвращает список пар идентификаторов стен Vk и Telegram каналов.
    pub async fn get_channels(&self) -> HashMap<ChannelEntryId, ChannelInfo> {
        self.db.get_channels().await.into_iter().collect()
    }

    /// Сохраняет канал и запускает для него процесс опроса.
    pub async fn create(&self, info: ChannelInfo) {
        let id = self.db.new_channel(&info).await;
        self.spawn_poller(id, info).await;
    }

    async fn spawn_poller(&self, id: ChannelEntryId, info: ChannelInfo) {
        let stop_token = CancellationToken::new();

        self.tracker.spawn(
            VkPoller::new(
                self.config.clone(),
                self.db.clone(),
                id,
                info,
                self.bot.clone(),
                self.cancellation_token.clone(),
                stop_token.clone(),
            )
            .run(),
        );

        self.stop_tokens.lock().await.insert(id, stop_token);
    }

    pub async fn delete(&self, id: ChannelEntryId) -> bool {
        let Some(stop_token) = self.stop_tokens.lock().await.remove(&id) else {
            return false;
        };

        stop_token.cancel();

        self.db.remove_channel(id).await;

        true
    }
}

use crate::{
    config, db,
    domain::{ChannelEntryId, ChannelInfo},
    vk_api,
};
use anyhow::Context;
use std::{sync::Arc, time::Duration};
use url::Url;

pub struct VkPoller {
    config: Arc<config::Config>,
    db: Arc<db::Db>,
    id: ChannelEntryId,
    info: ChannelInfo,
}

impl VkPoller {
    pub fn new(
        config: Arc<config::Config>,
        db: Arc<db::Db>,
        id: ChannelEntryId,
        info: ChannelInfo,
    ) -> Self {
        Self {
            config,
            db,
            id,
            info,
        }
    }

    pub async fn run(mut self) {
        loop {
            let should_poll = self
                .info
                .last_poll_datetime
                .map(|dt| self.info.poll_interval < (chrono::Utc::now() - dt))
                .unwrap_or(true);

            if !should_poll {
                tokio::time::sleep(Duration::from_millis(1000)).await;
                continue;
            }

            log::debug!("Time to poll VK wall '{}'...", self.info.vk_public_id);

            if let Some(post_id) = self.info.vk_last_post {
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
                self.first_poll().await;
            }
        }
    }

    async fn first_poll(&mut self) {
        let id = &self.info.vk_public_id;

        match self.get_first_non_pinned_post_id().await {
            Ok(Some(post_id)) => {
                log::debug!("Successfully fetch non pinned post {post_id:?} from VK wall '{id}'");
                self.info.vk_last_post = Some(post_id);
            }
            Ok(None) => {
                log::info!("No posts on VK wall '{id}'");
            }
            Err(err) => {
                return log::warn!("Failed to fetch latest post from VK wall '{id}': {err:#}");
            }
        }

        self.info.last_poll_datetime = Some(chrono::Utc::now());

        self.db.update_channel(self.id, &self.info).await;
    }

    async fn get_first_non_pinned_post_id(&self) -> anyhow::Result<Option<vk_api::PostId>> {
        let mut offset = 0;
        let count = 5;

        loop {
            let posts = self
                .get_posts(&self.info.vk_public_id, offset, count)
                .await
                .context("fetching posts from VK")?;

            if posts.is_empty() {
                return Ok(None);
            }

            if let Some(post) = posts.into_iter().find(|post| !post.is_pinned()) {
                return Ok(Some(post.id));
            } else {
                offset += count;
            }
        }
    }

    async fn get_posts(
        &self,
        vk_id: &str,
        offset: usize,
        count: usize,
    ) -> anyhow::Result<Vec<vk_api::Post>> {
        const VERSION: &str = "5.137";
        const METHOD: &str = "wall.get";

        let config = &self.config.vk;

        let url = Url::parse_with_params(
            &format!("{base}method/{METHOD}", base = &config.server),
            &[
                ("v", VERSION),
                ("lang", &config.language),
                ("domain", vk_id),
                ("offset", &offset.to_string()),
                ("count", &count.to_string()),
            ],
        )
        .expect("url should be valid");

        let client = reqwest::Client::new();

        let response = client
            .get(url)
            .bearer_auth(&config.service_key)
            .send()
            .await
            .context("executing wall.get")?;

        let response = response
            .text()
            .await
            .context("reading response from wall.get")?;

        let response = serde_json::from_str::<vk_api::Response<vk_api::Posts>>(&response)
            .context("parsing response from wall.get")?;

        Ok(response.response.items)
    }
}

use crate::{
    config, db,
    domain::{ChannelEntryId, ChannelInfo, TelegramPost},
    vk_api,
};
use anyhow::Context;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use url::Url;

pub struct VkPoller {
    config: Arc<config::Config>,
    db: db::Db,
    id: ChannelEntryId,
    info: ChannelInfo,
    bot: teloxide::Bot,
    cancellation_token: CancellationToken,
}

impl VkPoller {
    pub fn new(
        config: Arc<config::Config>,
        db: db::Db,
        id: ChannelEntryId,
        info: ChannelInfo,
        bot: teloxide::Bot,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            config,
            db,
            id,
            info,
            bot,
            cancellation_token,
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
                tokio::select! {
                    _ = self.cancellation_token.cancelled() => { break; },
                    _ = sleep(Duration::from_millis(1000)) => { continue; }
                }
            }

            log::debug!("Time to poll VK wall '{}'...", self.info.vk_public_id);

            if let Some(last_poll_datetime) = self.info.last_poll_datetime {
                self.poll_new_posts(last_poll_datetime).await;
            } else {
                self.first_poll().await;
                tokio::time::sleep(Duration::from_millis(5000)).await;
            }

            if self.cancellation_token.is_cancelled() {
                break;
            }
        }
    }

    async fn poll_new_posts(&mut self, last_poll_datetime: chrono::DateTime<chrono::Utc>) {
        let posts = match self.get_new_posts(last_poll_datetime).await {
            Ok(posts) => posts,
            Err(err) => {
                return log::warn!(
                    "Failed to fetch new posts from VK wall '{id}': {err:#}",
                    id = self.info.vk_public_id
                );
            }
        };

        if posts.is_empty() {
            self.info.last_poll_datetime = Some(chrono::Utc::now());
            self.db.update_channel(self.id, &self.info).await;
            return;
        }

        let mut last_poll_datetime = last_poll_datetime;

        for post in posts {
            let post_id = post.id.0;
            let post_datetime = post.date;

            let post = self.convert_vk_to_tg(post).await;

            match crate::bot::send_post(&self.bot, post).await {
                Ok(()) => {
                    log::info!("Successfully send post #{post_id} to the Telegram");
                    last_poll_datetime = post_datetime;
                }
                Err(err) => {
                    log::warn!("Failed to send post #{post_id} to the Telegram: {err:#}");
                    break;
                }
            }
        }

        self.info.last_poll_datetime = Some(last_poll_datetime);
        self.db.update_channel(self.id, &self.info).await;
    }

    async fn get_new_posts(
        &mut self,
        last_poll_datetime: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<Vec<vk_api::Post>> {
        let mut offset = 0;
        let count = 5;

        let mut new_posts = Vec::<vk_api::Post>::new();
        'fetch: loop {
            let posts = self
                .get_posts(offset, count)
                .await
                .context("fetching posts from VK")?;

            log::debug!("Posts: {posts:?}");

            if posts.is_empty() {
                break;
            }

            for post in posts {
                if post.is_pinned() {
                    continue;
                }

                if post.date <= last_poll_datetime {
                    dbg!(post.date);
                    dbg!(last_poll_datetime);
                    break 'fetch;
                }

                new_posts.push(post);
            }

            offset += count;
        }

        Ok(new_posts)
    }

    async fn convert_vk_to_tg(&self, post: vk_api::Post) -> TelegramPost {
        TelegramPost {
            channel_id: self.info.tg_channel,
            text: post.text,
        }
    }

    async fn first_poll(&mut self) {
        let id = &self.info.vk_public_id;

        match self.get_first_non_pinned_post_id().await {
            Ok(Some(post_id)) => {
                let post_id = post_id.0;
                log::debug!("Successfully fetch non pinned post {post_id} from VK wall '{id}'");
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
                .get_posts(offset, count)
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

    async fn get_posts(&self, offset: usize, count: usize) -> anyhow::Result<Vec<vk_api::Post>> {
        const VERSION: &str = "5.137";
        const METHOD: &str = "wall.get";

        let config = &self.config.vk;

        let url = Url::parse_with_params(
            &format!("{base}method/{METHOD}", base = &config.server),
            &[
                ("v", VERSION),
                ("lang", &config.language),
                ("domain", &self.info.vk_public_id),
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
            .with_context(|| format!("parsing response '{response}' from wall.get"))?;

        Ok(response.response.items)
    }
}

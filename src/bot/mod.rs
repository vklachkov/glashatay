mod dialogue;

use crate::{domain::TelegramPost, vk_poller};
use anyhow::Context;
use tokio_util::sync::CancellationToken;

pub async fn run_dialogue(
    bot: teloxide::Bot,
    poller: vk_poller::VkPollManager,
    token: CancellationToken,
) {
    let shutdown_token = dialogue::start(bot, poller).await;

    token.cancelled().await;
    shutdown_token.shutdown().unwrap().await;
}

pub async fn send_post(bot: &teloxide::Bot, post: TelegramPost) -> anyhow::Result<()> {
    use teloxide::{
        requests::Requester,
        types::{ChatId, ParseMode},
    };

    let mut message = bot.send_message(ChatId(post.channel_id.0), post.text);
    message.message_thread_id = None;
    message.parse_mode = Some(ParseMode::MarkdownV2);
    message.disable_web_page_preview = Some(true);

    message
        .await
        .with_context(|| format!("sending text to channel {}", post.channel_id.0))?;

    Ok(())
}

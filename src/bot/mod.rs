mod dialogue;

use crate::{domain::TelegramPost, vk_poller};
use anyhow::Context;
use teloxide::payloads::SendPhotoSetters;
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
        types::{ChatId, InputFile, ParseMode},
    };

    let chat_id = ChatId(post.channel_id.0);

    let mut message = bot.send_message(chat_id, post.text);
    message.parse_mode = Some(ParseMode::MarkdownV2);
    message.disable_web_page_preview = Some(true);
    message
        .await
        .with_context(|| format!("sending text to channel {}", post.channel_id.0))?;

    for photo in post.photos {
        bot.send_photo(chat_id, InputFile::memory(photo.bytes))
            .caption(photo.description)
            .await
            .with_context(|| format!("sending photo to channel {}", post.channel_id.0))?;
    }

    Ok(())
}

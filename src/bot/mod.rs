mod dialogue;

use crate::{
    domain::{TelegramPost, TelegramPostPhoto},
    vk_poller,
};
use anyhow::Context;
use teloxide::{
    requests::Requester,
    types::{ChatId, InputFile, InputMedia, InputMediaPhoto, MessageId, ParseMode},
    Bot,
};
use tokio_util::sync::CancellationToken;

pub async fn run_dialogue(bot: Bot, poller: vk_poller::VkPollManager, token: CancellationToken) {
    let shutdown_token = dialogue::start(bot, poller).await;

    token.cancelled().await;
    shutdown_token.shutdown().unwrap().await;
}

// TODO: Обработка частично отправленных публикаций.
pub async fn send_post(bot: &Bot, post: TelegramPost) -> anyhow::Result<()> {
    let chat_id = ChatId(post.channel_id.0);

    let first_text_message_id = send_text(bot, chat_id, post.text).await?;
    let first_photo_message_id = send_photos(bot, chat_id, post.photos).await?;

    if post.is_pinned {
        if let Some(message_id) = first_text_message_id.or(first_photo_message_id) {
            bot.pin_chat_message(chat_id, message_id)
                .await
                .with_context(|| format!("pinning message {message_id} in channel {chat_id}"))?;
        }
    }

    Ok(())
}

async fn send_text(bot: &Bot, chat_id: ChatId, text: String) -> anyhow::Result<Option<MessageId>> {
    if text.is_empty() {
        return Ok(None);
    };

    let mut message = bot.send_message(chat_id, text);
    message.parse_mode = Some(ParseMode::MarkdownV2);
    message.disable_web_page_preview = Some(true);

    let message = message
        .await
        .with_context(|| format!("sending text to channel {chat_id}"))?;

    Ok(Some(message.id))
}

async fn send_photos(
    bot: &Bot,
    chat_id: ChatId,
    photos: Vec<TelegramPostPhoto>,
) -> anyhow::Result<Option<MessageId>> {
    let mut first_message_id = None;

    // TODO: Разбить на чанки без аллокаций.
    let photo_collections: Vec<Vec<TelegramPostPhoto>> =
        photos.chunks(10).map(|chunk| chunk.to_vec()).collect();

    for collection in photo_collections {
        let media = collection.into_iter().map(|photo| {
            InputMedia::Photo(
                InputMediaPhoto::new(InputFile::memory(photo.bytes)).caption(photo.description),
            )
        });

        let messages = bot
            .send_media_group(chat_id, media)
            .await
            .with_context(|| format!("sending photo to channel {chat_id}"))?;

        if first_message_id.is_none() {
            if let Some(message) = messages.first() {
                first_message_id = Some(message.id);
            }
        }
    }

    Ok(first_message_id)
}

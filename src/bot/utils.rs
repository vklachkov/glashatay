use super::dialogue::BotDialogue;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

#[inline(always)]
pub async fn send_msg(bot: &Bot, chat_id: ChatId, text: &str) -> anyhow::Result<()> {
    bot.send_message(chat_id, text)
        .disable_web_page_preview(true)
        .await?;

    Ok(())
}

#[inline(always)]
pub async fn edit_msg(bot: &Bot, msg: &Message, text: &str) -> anyhow::Result<()> {
    bot.edit_message_text(msg.chat.id, msg.id, text)
        .disable_web_page_preview(true)
        .await?;

    Ok(())
}

#[inline(always)]
pub async fn send_interative(
    bot: &Bot,
    dialogue: &BotDialogue,
    text: &str,
    buttons: &[(&str, (usize, &str))],
) -> anyhow::Result<Message> {
    let message = bot
        .send_message(dialogue.chat_id(), text)
        .reply_markup(buttons_to_inline_keyboard(buttons))
        .disable_web_page_preview(true)
        .await?;

    Ok(message)
}

#[inline(always)]
fn buttons_to_inline_keyboard(buttons: &[(&str, (usize, &str))]) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new((0..buttons.len()).map(|idx| {
        buttons
            .iter()
            .filter(move |(_, (row, _))| *row == idx)
            .map(|(id, (_, text))| InlineKeyboardButton::callback(*text, *id))
    }))
}

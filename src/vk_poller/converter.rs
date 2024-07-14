use anyhow::{anyhow, Context};
use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;

use crate::{
    domain::{TelegramChannelId, TelegramPost, TelegramPostPhoto},
    vk_api,
};

pub async fn vk_to_tg(
    channel_id: TelegramChannelId,
    post: vk_api::Post,
) -> anyhow::Result<TelegramPost> {
    let mut photos = Vec::new();

    for attachment in post.attachments {
        match attachment {
            vk_api::Attachment::Photo(photo) => {
                let description = photo.description;

                let photo_url = photo
                    .sizes
                    .into_iter()
                    .find(|size| size.r#type == vk_api::PhotoType::W)
                    .map(|size| size.url)
                    .ok_or_else(|| anyhow!("failed to find W type photo size"))?;

                let bytes = reqwest::get(photo_url)
                    .await
                    .context("requesting photo from VK")?
                    .bytes()
                    .await
                    .context("downloading photo from VK")?
                    .to_vec();

                photos.push(TelegramPostPhoto { bytes, description })
            }
            _ => {
                unimplemented!()
            }
        }
    }

    Ok(TelegramPost {
        channel_id,
        text: vk_format_to_markdown(&post.text),
        photos,
        is_pinned: post.is_pinned == Some(1),
    })
}

fn vk_format_to_markdown(text: &str) -> String {
    let text = convert_links(text);
    escape_characters(&text)
}

fn convert_links(text: &str) -> Cow<str> {
    static LINKS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[(.+)\|(.+)\]").unwrap());

    LINKS_REGEX.replace_all(text, "[$2](https://vk.com/$1)")
}

fn escape_characters(text: &str) -> String {
    const SPECIAL_SYMBOLS: [char; 16] = [
        '+', '-', '=', '#', '!', '?', '_', '.', '*', '[', ']', '(', ')', '{', '}', '`',
    ];

    let mut escaped = String::with_capacity(text.len());

    for chr in text.chars() {
        if SPECIAL_SYMBOLS.contains(&chr) {
            escaped.push('\\');
        }

        escaped.push(chr);
    }

    escaped
}

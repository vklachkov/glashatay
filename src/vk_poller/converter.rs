use anyhow::{anyhow, Context};
use once_cell::sync::Lazy;
use regex::Regex;

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
    })
}

fn vk_format_to_markdown(text: &str) -> String {
    static LINKS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[(.+)\|(.+)\]").unwrap());

    LINKS_REGEX
        .replace_all(text, "[$2](https://vk.com/$1)")
        .to_string()
}

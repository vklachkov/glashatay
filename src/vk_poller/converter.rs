use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    domain::{TelegramChannelId, TelegramPost},
    vk_api,
};

pub async fn vk_to_tg(channel_id: TelegramChannelId, post: vk_api::Post) -> TelegramPost {
    TelegramPost {
        channel_id,
        text: vk_format_to_markdown(&post.text),
    }
}

fn vk_format_to_markdown(text: &str) -> String {
    static LINKS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[(.+)\|(.+)\]").unwrap());

    LINKS_REGEX
        .replace_all(text, "[$2](https://vk.com/$1)")
        .to_string()
}

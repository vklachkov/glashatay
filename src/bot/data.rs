use crate::domain::{ChannelEntryId, ChannelInfo, TelegramChannelId, VkId};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use teloxide::types::ChatId;

pub const START_MESSAGE: &str = "Старт";

pub const HELP_MESSAGE: &str = "Хелп";

pub const REQUEST_CHANNEL_MESSAGE: &str =
    "Перешли, пожалуйста, сообщение из паблика, в который будут отправляться посты из ВКонтакте";

pub const INVALID_CHANNEL_MESSAGE: &str = "Сообщение должно быть переслано из канала";

pub const CHANNEL_RECEIVED_MESSAGE: &dyn Fn(&ChatId, &str) -> String =
    &|id, title| format!("Посты будут публиковаться в паблик '{title}' ({id})");

pub const REQUEST_VK_URL_MESSAGE: &str =
    "Напишите, пожалуйста, ссылку на стену сообщества, группы или человека во ВКонтакте";

pub const CHANNEL_ADDED_MESSAGE: &dyn Fn(&VkId, &TelegramChannelId) -> String =
    &|vk_id, tg_id| format!("Отлично! Посты будут репоститься из {vk_id} в {tg_id}");

pub const NO_CHANNELS_MESSAGE: &str = "Нет каналов";

pub const REQUEST_CHANNEL_NUMBER_MESSAGE: &dyn Fn(&HashMap<ChannelEntryId, ChannelInfo>) -> String =
    &|channels| {
        format!(
            "Отправьте номер записи, которую хотите удалить:\n\n{}",
            format_channels_to_string(channels)
        )
    };

pub const INVALID_CHANNEL_NUMBER_MESSAGE: &str = "Напишите, пожалуйста, номер записи";

pub const APPROVE_CHANNEL_DELETION_MESSAGE: &dyn Fn(usize) -> String =
    &|number| format!("Вы уверены, что хотите удалить запись {}?", number + 1);

#[rustfmt::skip]
pub static APPROVE_CHANNEL_DELETION_BUTTONS: Lazy<[(&str, (usize, &str)); 2]> = Lazy::new(|| {
    [
        ("true", (0, "🗑️ Удалить канал")),
        ("false", (0, "Отмена")),
    ]
});

pub const STOPPING_CHANNEL_JOB_MESSAGE: &dyn Fn(&VkId, &TelegramChannelId) -> String =
    &|vk_id, tg_id| format!("Остановка пересылки постов из {vk_id} в канал {tg_id}...");

pub const CHANNEL_DELETED_MESSAGE: &dyn Fn(&VkId, &TelegramChannelId) -> String =
    &|vk_id, tg_id| format!("Пересылка постов из {vk_id} в канал {tg_id} прекращена!");

pub const CHANNEL_DELETION_CANCELLED_MESSAGE: &str = "Удаление отменено!";

pub const LIST_CHANNELS_MESSAGE: &dyn Fn(&HashMap<ChannelEntryId, ChannelInfo>) -> String =
    &|channels| {
        format!(
            "Ваш список каналов:\n\n{}",
            format_channels_to_string(channels)
        )
    };

pub const CANCEL_MESSAGE: &str = "Команда отменена";

pub const UNKNOWN_ACTION_MESSAGE: &str = "Не понимаю(";

fn format_channels_to_string(channels: &HashMap<ChannelEntryId, ChannelInfo>) -> String {
    channels
        .iter()
        .enumerate()
        .map(|(n, (_id, info))| {
            let n = n + 1;
            let vk_id = &info.vk_public_id;
            let tg_id = &info.tg_channel;

            format!("{n}. Из {vk_id} в {tg_id}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

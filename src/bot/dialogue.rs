use crate::{
    domain::{ChannelEntryId, ChannelInfo, TelegramChannelId, VkId},
    vk_poller,
};
use std::collections::HashMap;
use teloxide::{
    dispatching::{
        dialogue::{self, InMemStorage},
        ShutdownToken, UpdateHandler,
    },
    macros::BotCommands,
    prelude::*,
    types::{
        Chat, ChatKind, ChatPublic, ForwardedFrom, InlineKeyboardButton, InlineKeyboardMarkup,
    },
};
use url::Url;

type BotDialogue = Dialogue<BotState, InMemStorage<BotState>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum BotCommand {
    Start,
    Help,
    Add,
    Delete,
    List,
    Cancel,
}

#[derive(Clone, Default)]
pub enum BotState {
    #[default]
    Empty,
    AddingChanneд(AddingChannelBotState),
    DeletingChannel(DeletingChannelBotState),
}

#[derive(Clone)]
pub enum AddingChannelBotState {
    ReceiveChannelId,
    ReceiveVkUrl { channel_id: ChatId },
}

#[derive(Clone)]
pub enum DeletingChannelBotState {
    ReceiveChannelNumber {
        channels: HashMap<ChannelEntryId, ChannelInfo>,
    },
    ApproveDelete {
        message: Message,
        id: ChannelEntryId,
        info: ChannelInfo,
    },
}

pub async fn start(bot: Bot, poller: vk_poller::VkPollManager) -> ShutdownToken {
    let bot_state = InMemStorage::<BotState>::new();

    let mut dispatcher = Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![bot_state, poller])
        .build();

    let token = dispatcher.shutdown_token();

    tokio::spawn(async move {
        dispatcher.dispatch().await;
    });

    token
}

#[rustfmt::skip]
fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<BotCommand, _>()
        .branch(
            case![BotState::Empty]
                .branch(case![BotCommand::Start].endpoint(help))
                .branch(case![BotCommand::Help].endpoint(help))
                .branch(case![BotCommand::Add].endpoint(add_channel))
                .branch(case![BotCommand::Delete].endpoint(delete_channel))
                .branch(case![BotCommand::List].endpoint(list_channels))
        )
        .branch(case![BotCommand::Cancel].endpoint(cancel_action)
    );

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(
            case![BotState::AddingChanneд(state)]
                .branch(case![AddingChannelBotState::ReceiveChannelId].endpoint(receive_channel_id))
                .branch(case![AddingChannelBotState::ReceiveVkUrl { channel_id }].endpoint(receive_vk_url))
        )
        .branch(
            case![BotState::DeletingChannel(state)]
                .branch(case![DeletingChannelBotState::ReceiveChannelNumber {channels }].endpoint(receive_entry_for_delete))
        )
        .branch(dptree::endpoint(other));
        
    let callback_query_handler = Update::filter_callback_query()
        .branch(
            case![BotState::DeletingChannel(state)]
                .branch(case![DeletingChannelBotState::ApproveDelete { message, id, info }].endpoint(approve_delete))
        );

    dialogue::enter::<Update, InMemStorage<BotState>, BotState, _>().branch(message_handler)
    .branch(callback_query_handler)
}

async fn help(bot: Bot, dialogue: BotDialogue) -> HandlerResult {
    bot.send_message(dialogue.chat_id(), "Start/Help TODO")
        .await?;

    Ok(())
}

async fn add_channel(bot: Bot, dialogue: BotDialogue) -> HandlerResult {
    let text = "Привет! Перешли, пожалуйста, сообщение из паблика, в который будут отправляться посты из ВКонтакте";
    bot.send_message(dialogue.chat_id(), text).await?;

    dialogue
        .update(BotState::AddingChanneд(
            AddingChannelBotState::ReceiveChannelId,
        ))
        .await?;

    Ok(())
}

async fn receive_channel_id(bot: Bot, dialogue: BotDialogue, msg: Message) -> HandlerResult {
    let non_channel_forward = async {
        bot.send_message(
            dialogue.chat_id(),
            "Сообщение должно быть переслано из канала",
        )
        .await?;

        Ok(())
    };

    let Some(ForwardedFrom::Chat(Chat {
        id: channel_id,
        kind:
            ChatKind::Public(ChatPublic {
                title: Some(channel_title),
                ..
            }),
        ..
    })) = msg.forward_from()
    else {
        return non_channel_forward.await;
    };

    if channel_id.0 >= 0 {
        return non_channel_forward.await;
    }

    bot.send_message(
        dialogue.chat_id(),
        format!("Посты будут публиковаться в паблик '{channel_title}' ({channel_id})"),
    )
    .await?;

    bot.send_message(
        dialogue.chat_id(),
        "Напишите, пожалуйста, ссылку на стену сообщества, группы или человека во ВКонтакте",
    )
    .await?;

    dialogue
        .update(BotState::AddingChanneд(
            AddingChannelBotState::ReceiveVkUrl {
                channel_id: *channel_id,
            },
        ))
        .await?;

    Ok(())
}

async fn receive_vk_url(
    bot: Bot,
    dialogue: BotDialogue,
    posts_channel_id: ChatId,
    msg: Message,
    poller: vk_poller::VkPollManager,
) -> HandlerResult {
    let Some(url) = msg.text().and_then(|text| Url::parse(text).ok()) else {
        bot.send_message(
            dialogue.chat_id(),
            "Напишите, пожалуйста, ссылку на стену сообщества, группы или человека во ВКонтакте",
        )
        .await?;

        return Ok(());
    };

    // Первый символ всегда /
    let vk_id = VkId(url.path().chars().skip(1).collect::<String>());

    let tg_id = TelegramChannelId(posts_channel_id.0);

    bot.send_message(
        dialogue.chat_id(),
        format!("Отлично! Посты будут репоститься из {vk_id} в {tg_id}"),
    )
    .await?;

    poller
        .create(ChannelInfo {
            tg_channel: tg_id,
            vk_public_id: vk_id,
            poll_interval: chrono::Duration::seconds(2),
            last_poll_datetime: None,
            last_post_datetime: None,
        })
        .await;

    dialogue.update(BotState::Empty).await?;

    Ok(())
}

async fn delete_channel(
    bot: Bot,
    dialogue: BotDialogue,
    poller: vk_poller::VkPollManager,
) -> HandlerResult {
    let channels = poller.get_channels().await;

    if channels.is_empty() {
        bot.send_message(dialogue.chat_id(), "Нет каналов").await?;
        return Ok(());
    }

    bot.send_message(
        dialogue.chat_id(),
        format!(
            "Привет! Отправьте номер записи, которую хотите удалить:\n\n{}",
            format_channels_to_string(&channels)
        ),
    )
    .disable_web_page_preview(true)
    .await?;

    dialogue
        .update(BotState::DeletingChannel(
            DeletingChannelBotState::ReceiveChannelNumber { channels },
        ))
        .await?;

    Ok(())
}

async fn receive_entry_for_delete(
    bot: Bot,
    dialogue: BotDialogue,
    msg: Message,
    channels: HashMap<ChannelEntryId, ChannelInfo>,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(dialogue.chat_id(), "Напишите, пожалуйста, номер записи")
            .await?;

        return Ok(());
    };

    let Some(number) = text
        .parse()
        .ok()
        .and_then(|number: usize| number.checked_sub(1))
    else {
        bot.send_message(dialogue.chat_id(), "Напишите, пожалуйста, номер записи")
            .await?;

        return Ok(());
    };

    let Some((id, info)) = channels.into_iter().nth(number) else {
        bot.send_message(dialogue.chat_id(), "Напишите, пожалуйста, номер записи")
            .await?;

        return Ok(());
    };

    let message = bot
        .send_message(
            dialogue.chat_id(),
            format!("Вы уверены, что хотите удалить запись {text}?",),
        )
        .reply_markup(InlineKeyboardMarkup::new([[
            InlineKeyboardButton::callback("Да", true.to_string()),
            InlineKeyboardButton::callback("Нет", false.to_string()),
        ]]))
        .await?;

    dialogue
        .update(BotState::DeletingChannel(
            DeletingChannelBotState::ApproveDelete { message, id, info },
        ))
        .await?;

    Ok(())
}

async fn approve_delete(
    bot: Bot,
    dialogue: BotDialogue,
    poller: vk_poller::VkPollManager,
    q: CallbackQuery,
    (message, channel_id, channel_info): (Message, ChannelEntryId, ChannelInfo),
) -> HandlerResult {
    let Some(approved) = q.data else {
        return Ok(());
    };

    let Ok(approved) = approved.parse() else {
        panic!("Invalid value '{approved}'");
    };

    if approved {
        let vk_id = &channel_info.vk_public_id;
        let tg_id = &channel_info.tg_channel;

        bot.edit_message_text(
            dialogue.chat_id(),
            message.id,
            format!("Остановка пересылки постов из {vk_id} в канал {tg_id}..."),
        )
        .await?;

        let delete_success = poller.delete(channel_id).await;
        assert!(delete_success);

        bot.edit_message_text(
            dialogue.chat_id(),
            message.id,
            format!("Пересылка постов из {vk_id} в канал {tg_id} прекращена!"),
        )
        .await?;
    } else {
        bot.edit_message_text(dialogue.chat_id(), message.id, "Удаление отменено!")
            .await?;
    }

    dialogue.update(BotState::Empty).await?;

    Ok(())
}

async fn list_channels(
    bot: Bot,
    dialogue: BotDialogue,
    poller: vk_poller::VkPollManager,
) -> HandlerResult {
    let channels = poller.get_channels().await;

    if channels.is_empty() {
        bot.send_message(dialogue.chat_id(), "Нет каналов").await?;
        return Ok(());
    }

    bot.send_message(
        dialogue.chat_id(),
        format!(
            "Ваш список каналов:\n\n{}",
            format_channels_to_string(&channels)
        ),
    )
    .disable_web_page_preview(true)
    .await?;

    Ok(())
}

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

async fn cancel_action(bot: Bot, dialogue: BotDialogue) -> HandlerResult {
    bot.send_message(dialogue.chat_id(), "Отмена действия")
        .await?;

    dialogue.update(BotState::Empty).await?;

    Ok(())
}

async fn other(bot: Bot, dialogue: BotDialogue) -> HandlerResult {
    bot.send_message(dialogue.chat_id(), "Не понимаю(").await?;

    Ok(())
}

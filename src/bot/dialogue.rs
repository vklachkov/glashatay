use super::{data::*, utils::*};
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
    types::{Chat, ChatKind, ChatPublic, ForwardedFrom},
};
use url::Url;

pub(crate) type BotDialogue = Dialogue<BotState, InMemStorage<BotState>>;
pub(crate) type HandlerResult = Result<(), anyhow::Error>;
pub(crate) type HandlerError = anyhow::Error;

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
fn schema() -> UpdateHandler<HandlerError> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<BotCommand, _>()
        .branch(
            case![BotState::Empty]
                .branch(case![BotCommand::Start].endpoint(hello))
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

/// Команда `/start`.
async fn hello(bot: Bot, dialogue: BotDialogue) -> HandlerResult {
    send_msg(&bot, dialogue.chat_id(), START_MESSAGE).await
}

/// Команда `/help`.
async fn help(bot: Bot, dialogue: BotDialogue) -> HandlerResult {
    send_msg(&bot, dialogue.chat_id(), HELP_MESSAGE).await
}

/// Команда `/add`.
async fn add_channel(bot: Bot, dialogue: BotDialogue) -> HandlerResult {
    send_msg(&bot, dialogue.chat_id(), REQUEST_CHANNEL_MESSAGE).await?;

    dialogue
        .update(BotState::AddingChanneд(
            AddingChannelBotState::ReceiveChannelId,
        ))
        .await
        .map_err(Into::into)
}

async fn receive_channel_id(bot: Bot, dialogue: BotDialogue, msg: Message) -> HandlerResult {
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
        return send_msg(&bot, dialogue.chat_id(), INVALID_CHANNEL_MESSAGE).await;
    };

    if channel_id.0 >= 0 {
        return send_msg(&bot, dialogue.chat_id(), INVALID_CHANNEL_MESSAGE).await;
    }

    send_msg(
        &bot,
        dialogue.chat_id(),
        &CHANNEL_RECEIVED_MESSAGE(channel_id, channel_title),
    )
    .await?;

    send_msg(&bot, dialogue.chat_id(), REQUEST_VK_URL_MESSAGE).await?;

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
        return send_msg(&bot, dialogue.chat_id(), REQUEST_VK_URL_MESSAGE).await;
    };

    let vk_id = VkId::from(url);
    let tg_id = TelegramChannelId(posts_channel_id.0);

    send_msg(
        &bot,
        dialogue.chat_id(),
        &CHANNEL_ADDED_MESSAGE(&vk_id, &tg_id),
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

/// Команда `/delete`.
async fn delete_channel(
    bot: Bot,
    dialogue: BotDialogue,
    poller: vk_poller::VkPollManager,
) -> HandlerResult {
    let channels = poller.get_channels().await;

    if channels.is_empty() {
        return send_msg(&bot, dialogue.chat_id(), NO_CHANNELS_MESSAGE).await;
    }

    send_msg(
        &bot,
        dialogue.chat_id(),
        &REQUEST_CHANNEL_NUMBER_MESSAGE(&channels),
    )
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
        return send_msg(&bot, dialogue.chat_id(), INVALID_CHANNEL_NUMBER_MESSAGE).await;
    };

    let Some(number) = text
        .parse()
        .ok()
        .and_then(|number: usize| number.checked_sub(1))
    else {
        return send_msg(&bot, dialogue.chat_id(), INVALID_CHANNEL_NUMBER_MESSAGE).await;
    };

    let Some((id, info)) = channels.into_iter().nth(number) else {
        return send_msg(&bot, dialogue.chat_id(), INVALID_CHANNEL_NUMBER_MESSAGE).await;
    };

    let message = send_interative(
        &bot,
        &dialogue,
        &APPROVE_CHANNEL_DELETION_MESSAGE(number),
        &*APPROVE_CHANNEL_DELETION_BUTTONS,
    )
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

        edit_msg(&bot, &message, &STOPPING_CHANNEL_JOB_MESSAGE(vk_id, tg_id)).await?;

        let delete_success = poller.delete(channel_id).await;
        assert!(delete_success);

        edit_msg(&bot, &message, &CHANNEL_DELETED_MESSAGE(vk_id, tg_id)).await?;
    } else {
        edit_msg(&bot, &message, CHANNEL_DELETION_CANCELLED_MESSAGE).await?;
    }

    dialogue.update(BotState::Empty).await?;

    Ok(())
}

/// Команда `/list`.
async fn list_channels(
    bot: Bot,
    dialogue: BotDialogue,
    poller: vk_poller::VkPollManager,
) -> HandlerResult {
    let channels = poller.get_channels().await;

    if channels.is_empty() {
        send_msg(&bot, dialogue.chat_id(), NO_CHANNELS_MESSAGE).await
    } else {
        send_msg(&bot, dialogue.chat_id(), &LIST_CHANNELS_MESSAGE(&channels)).await
    }
}

/// Команда `/cancel`.
async fn cancel_action(bot: Bot, dialogue: BotDialogue) -> HandlerResult {
    dialogue.update(BotState::Empty).await?;
    send_msg(&bot, dialogue.chat_id(), CANCEL_MESSAGE).await
}

/// Fallback.
async fn other(bot: Bot, dialogue: BotDialogue) -> HandlerResult {
    send_msg(&bot, dialogue.chat_id(), UNKNOWN_ACTION_MESSAGE).await
}

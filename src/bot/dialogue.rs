use crate::{
    domain::{ChannelInfo, TelegramChannelId},
    vk_poller,
};
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

type MyDialogue = Dialogue<BotState, InMemStorage<BotState>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum BotCommand {
    Start,
}

#[derive(Clone, Default)]
pub enum BotState {
    #[default]
    Start,
    ReceiveChannelId,
    ReceiveVkUrl {
        channel_id: ChatId,
    },
    Active {
        channel_id: ChatId,
        vk_id: String,
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
            case![BotState::Start]
                .branch(case![BotCommand::Start].endpoint(bot_start)));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![BotState::Start].endpoint(lets_start))
        .branch(case![BotState::ReceiveChannelId].endpoint(receive_channel_id))
        .branch(case![BotState::ReceiveVkUrl { channel_id }].endpoint(receive_vk_url))
        .branch(dptree::endpoint(other));

    dialogue::enter::<Update, InMemStorage<BotState>, BotState, _>().branch(message_handler)
}

async fn lets_start(bot: Bot, msg: Message) -> HandlerResult {
    let text = "Чтобы начать, введите /start";
    bot.send_message(msg.chat.id, text).await?;

    Ok(())
}

async fn bot_start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let text = "Привет! Перешли, пожалуйста, сообщение из паблика, в который будут отправляться посты из ВКонтакте";
    bot.send_message(msg.chat.id, text).await?;

    dialogue.update(BotState::ReceiveChannelId).await?;

    Ok(())
}

async fn receive_channel_id(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let non_channel_forward = async {
        bot.send_message(msg.chat.id, "Сообщение должно быть переслано из канала")
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
        msg.chat.id,
        format!("Посты будут публиковаться в паблик '{channel_title}' ({channel_id})"),
    )
    .await?;

    bot.send_message(
        msg.chat.id,
        "Напишите, пожалуйста, ссылку на стену сообщества, группы или человека во ВКонтакте",
    )
    .await?;

    dialogue
        .update(BotState::ReceiveVkUrl {
            channel_id: *channel_id,
        })
        .await?;

    Ok(())
}

async fn receive_vk_url(
    bot: Bot,
    dialogue: MyDialogue,
    posts_channel_id: ChatId,
    msg: Message,
    poller: vk_poller::VkPollManager,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(
            msg.chat.id,
            "Напишите, пожалуйста, ссылку на стену сообщества, группы или человека во ВКонтакте",
        )
        .await?;

        return Ok(());
    };

    let Ok(url) = Url::parse(text) else {
        bot.send_message(
            msg.chat.id,
            "Напишите, пожалуйста, ссылку на стену сообщества, группы или человека во ВКонтакте",
        )
        .await?;

        return Ok(());
    };

    // Первый символ всегда /
    let id = url.path().chars().skip(1).collect::<String>();

    bot.send_message(
        msg.chat.id,
        format!("Отлично! Посты будут репоститься из vk.com/{id} в {posts_channel_id}"),
    )
    .await?;

    poller
        .create(ChannelInfo {
            tg_channel: TelegramChannelId(posts_channel_id.0),
            vk_public_id: id.clone(),
            poll_interval: chrono::Duration::seconds(2),
            last_poll_datetime: None,
            vk_last_post: None,
        })
        .await;

    dialogue
        .update(BotState::Active {
            channel_id: posts_channel_id,
            vk_id: id,
        })
        .await?;

    Ok(())
}

async fn other(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Всё настроено").await?;
    Ok(())
}

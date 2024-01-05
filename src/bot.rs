use anyhow::Context;
use std::{env, sync::Arc};
use teloxide::{
    dispatching::{
        dialogue::{self, InMemStorage},
        UpdateHandler,
    },
    macros::BotCommands,
    prelude::*,
    types::{Chat, ChatKind, ChatPublic, ForwardedFrom},
};
use url::Url;

use crate::GlobalState;

type MyDialogue = Dialogue<BotState, InMemStorage<BotState>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum BotCommand {
    Start,
    TestPost,
}

#[derive(Clone, Default)]
pub enum BotState {
    #[default]
    Start,
    ReceiveChannelId,
    ReceiveVkUrl { channel_id: ChatId },
    Active { channel_id: ChatId, vk_id: String },
}

pub async fn run(global_state: Arc<GlobalState>) {
    const TOKEN: &str = "6786939573:AAEF6ciF_gBzKZzhsVV35qsYUgzfzX5IOA0";

    env::set_var("TELOXIDE_TOKEN", TOKEN);
    let bot = Bot::from_env();

    let bot_state = InMemStorage::<BotState>::new();
    let global_state = Arc::new(GlobalState {});

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![bot_state, global_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<BotCommand, _>()
        .branch(case![BotState::Start].branch(case![BotCommand::Start].endpoint(bot_start)))
        .branch(
            case![BotState::Active { channel_id, vk_id }]
                .branch(case![BotCommand::TestPost].endpoint(bot_test_post)),
        );

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![BotState::Start].endpoint(bot_lets_start))
        .branch(case![BotState::ReceiveChannelId].endpoint(bot_receive_channel_id))
        .branch(case![BotState::ReceiveVkUrl { channel_id }].endpoint(bot_receive_vk_url))
        .branch(dptree::endpoint(bot_other));

    dialogue::enter::<Update, InMemStorage<BotState>, BotState, _>().branch(message_handler)
}

async fn bot_lets_start(bot: Bot, msg: Message) -> HandlerResult {
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

async fn bot_receive_channel_id(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
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

async fn bot_receive_vk_url(
    bot: Bot,
    dialogue: MyDialogue,
    posts_channel_id: ChatId,
    msg: Message,
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

    dialogue
        .update(BotState::Active {
            channel_id: posts_channel_id,
            vk_id: id,
        })
        .await?;

    Ok(())
}

async fn bot_other(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Всё настроено").await?;
    Ok(())
}

async fn bot_test_post(bot: Bot, (channel_id, vk_id): (ChatId, String)) -> HandlerResult {
    let post = get_post(&vk_id).await.context("requesting to vk api")?;
    let post = vk2md2(post);

    bot.send_message(channel_id, post)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;

    Ok(())
}

async fn get_post(vk_id: &str) -> anyhow::Result<String> {
    const SERVER: &str = "api.vk.com";
    const VERSION: &str = "5.137";
    const LANGUAGE: &str = "ru";
    const METHOD: &str = "wall.get";
    const SERVICE_KEY: &str =
        "d9312976d9312976d931297609da27e21fdd931d9312976bca7810da8f13e38180454b8";

    let url: Url = Url::parse_with_params(
        &format!("https://{SERVER}/method/{METHOD}"),
        &[
            ("v", VERSION),
            ("lang", LANGUAGE),
            ("domain", vk_id),
            ("offset", "0"),
            ("count", "5"),
        ],
    )
    .expect("url should be valid");

    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .bearer_auth(SERVICE_KEY)
        .send()
        .await
        .context("executing wall.get")?;

    let response = response
        .json::<serde_json::Value>()
        .await
        .context("parsing response from wall.get")?;

    let text = response["response"]["items"][0]["text"]
        .as_str()
        .unwrap_or("[Нет текста]")
        .to_owned();

    Ok(text)
}

fn vk2md2(post: String) -> String {
    post
}

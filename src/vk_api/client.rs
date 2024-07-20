use crate::domain::VkId;
use anyhow::Context;
use chrono::Utc;
use serde::Serialize;
use std::path::PathBuf;
use tokio::fs;
use url::Url;

const SERVER: &str = "https://api.vk.com";
const VERSION: &str = "5.137";

/// Клиент для работы с API ВКонтакте.
pub struct Client {
    client: reqwest::Client,
    token: String,
    language: String,
    debug: Option<ClientDebug>,
}

/// Параметры отладки клиента.
pub struct ClientDebug {
    /// Флаг сохранения ответов в `responses_dir_path`.
    pub save_responses: bool,

    /// Путь до директории, куда будут сохраняться ответы от ВК,
    /// если установлен флаг `save_responses`.
    ///
    /// Формат имени файла: `vk-response-method-timestamp.json`.
    pub responses_dir_path: PathBuf,
}

#[derive(Serialize)]
struct MethodParams<'a, P> {
    #[serde(rename = "v")]
    pub api_version: &'a str,

    #[serde(rename = "lang")]
    pub language: &'a str,

    #[serde(flatten)]
    pub method_params: P,
}

impl Client {
    pub fn new(token: &str, language: &str, debug: Option<ClientDebug>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .build()
                .expect("builder should be valid"),
            language: language.to_owned(),
            token: token.to_owned(),
            debug,
        }
    }

    /// Возвращает список публикаций со стены пользователя или сообщества.
    ///
    /// # Параметры
    ///
    /// * `id` - Короткий адрес пользователя или сообщества.
    /// * `offset` - Смещение, необходимое для выборки определённого подмножества записей.
    /// * `count` - Количество записей, которое необходимо получить. Максимальное значение: 100.
    pub async fn get_posts_from_wall(
        &self,
        id: &VkId,
        offset: usize,
        count: usize,
    ) -> anyhow::Result<Vec<super::Post>> {
        #[derive(Serialize)]
        struct Params<'a> {
            domain: &'a str,
            offset: usize,
            count: usize,
        }

        self.get::<_, super::Posts>(
            "wall.get",
            Params {
                domain: &id.0,
                offset,
                count,
            },
        )
        .await
        .map(|posts| posts.items)
    }

    async fn get<P, R>(&self, method: &str, params: P) -> anyhow::Result<R>
    where
        P: serde::Serialize,
        R: for<'a> serde::Deserialize<'a>,
    {
        let params = serde_urlencoded::to_string(MethodParams {
            api_version: VERSION,
            language: &self.language,
            method_params: params,
        })
        .expect("params should be serializable");

        let url = &format!("{SERVER}/method/{method}?{params}");
        let url = Url::parse(url).expect("url should be valid");

        let response = self
            .client
            .get(url)
            .bearer_auth(&self.token)
            .send()
            .await
            .with_context(|| format!("executing method '{method}'"))?;

        let response = response
            .text()
            .await
            .with_context(|| format!("reading response from method '{method}'"))?;

        self.dump_response(method, &response).await;

        let response = serde_json::from_str::<super::Response<R>>(&response)
            .with_context(|| format!("parsing response '{response}' from method '{method}'"))?;

        // TODO: Обработка ошибок от ВК.

        Ok(response.response)
    }

    async fn dump_response(&self, method: &str, response: &str) {
        let Some(debug) = &self.debug else {
            return;
        };

        if !debug.save_responses {
            return;
        }

        let dump_path = debug.responses_dir_path.join(format!(
            "vk-response-{method}-{timestamp}.json",
            timestamp = Utc::now().timestamp_millis()
        ));

        match fs::write(&dump_path, response).await {
            Ok(()) => log::debug!(
                "Successfully save vk response into file '{path}'",
                path = dump_path.display(),
            ),
            Err(err) => log::error!(
                "Failed to save vk response into file '{path}': {err}",
                path = dump_path.display(),
                err = err,
            ),
        }
    }
}

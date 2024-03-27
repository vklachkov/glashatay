pub use super::attachment::Attachment;

use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Posts {
    pub count: u64,
    pub items: Vec<Post>,
}

/// Структура пабликации на стене, взятая из [https://dev.vk.com/ru/reference/objects/post].
///
/// Она не содержит все поля. Только необходимые для работы сервиса.
#[derive(Clone, Debug, Deserialize)]
pub struct Post {
    /// Идентификатор записи.
    pub id: i64,

    /// Идентификатор автора записи (от чьего имени опубликована запись).
    pub from_id: i64,

    /// Время публикации записи.
    #[serde(with = "ts_seconds")]
    pub date: DateTime<Utc>,

    /// Текст записи.
    pub text: String,

    /// Идентификатор владельца записи, в ответ на которую была оставлена текущая.
    pub reply_owner_id: Option<i64>,

    /// Идентификатор записи, в ответ на которую была оставлена текущая.
    pub reply_post_id: Option<i64>,

    /// Источник материала.
    pub copyright: Option<Copyright>,

    /// Тип записи.
    pub post_type: Type,

    /// Массив объектов, соответствующих медиаресурсам, прикреплённым к записи: фотографиям, документам, видеофайлам и другим.
    pub attachments: Vec<Attachment>,

    /// Информация о местоположении.
    pub geo: Option<Geolocation>,

    /// Идентификатор автора, если запись была опубликована от имени сообщества и подписана пользователем.
    pub signer_id: Option<i64>,

    /// Информация о том, что запись закреплена.
    pub is_pinned: Option<u8>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Copyright {
    pub id: i64,
    pub link: Option<String>,
    pub name: Option<String>,
    pub r#type: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Post,
    Copy,
    Reply,
    Postpone,
    Suggest,

    #[serde(untagged)]
    Other(String),
}

#[derive(Clone, Debug, Deserialize)]
pub struct Geolocation {
    pub r#type: String,
    pub coordinates: String,
    pub place: Place,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Place {
    /// Название места.
    pub title: String,

    /// Географическая широта, заданная в градусах (от -90 до 90).
    pub latitude: f64,

    /// Географическая широта, заданная в градусах (от -90 до 90).
    pub longitude: f64,

    /// Идентификатор страны.
    pub country: i64,

    /// Идентификатор города.
    pub city: i64,

    /// Адрес места.
    pub address: String,
}

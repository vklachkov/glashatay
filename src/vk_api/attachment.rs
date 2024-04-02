use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::{Deserialize, Deserializer};
use url::Url;

//
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase", remote = "Self")]
pub enum Attachment {
    Photo(Photo),
    PhotosList(PhotosList),
    Album(Album),
    Video(Video),
    Event(Event),
}

// Хак из ишью https://github.com/serde-rs/serde/issues/1343.
// TODO: Сохранять информацию о неподдерживаемом вложении.
impl<'de> Deserialize<'de> for Attachment {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Wrapper {
            #[serde(rename = "type")]
            _ignore: String,
            #[serde(flatten, with = "Attachment")]
            inner: Attachment,
        }

        Wrapper::deserialize(deserializer).map(|w| w.inner)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Photo {
    /// Идентификатор фотографии.
    pub id: i64,

    /// Идентификатор альбома, в котором находится фотография.
    pub album_id: i64,

    /// Идентификатор владельца фотографии.
    pub owner_id: i64,

    /// Идентификатор пользователя, загрузившего фото (если фотография размещена в сообществе).
    /// Для фотографий, размещенных от имени сообщества, user_id = 100.
    pub user_id: Option<i64>,

    /// Текст описания фотографии.
    #[serde(rename = "text")]
    pub description: String,

    /// Дата добавления.
    #[serde(with = "ts_seconds")]
    pub date: DateTime<Utc>,

    /// Массив со ссылками на копии изображения в разных размерах.
    pub sizes: Vec<PhotoSize>,

    /// Ширина оригинала фотографии в пикселях.
    pub width: Option<i64>,

    /// Высота оригинала фотографии в пикселях.
    pub height: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PhotoSize {
    /// Ссылка на копию изображения.
    pub url: Url,

    /// Ширина копии в пикселях.
    pub width: i64,

    /// Высота копии в пикселях.
    pub height: i64,

    /// Обозначение размера и пропорций копии.
    pub r#type: PhotoType,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PhotoType {
    /// Пропорциональная копия изображения с максимальной стороной 75px.
    S,

    /// Пропорциональная копия изображения с максимальной стороной 130px.
    M,

    /// Пропорциональная копия изображения с максимальной стороной 604px.
    X,

    /// Если соотношение "ширина/высота" исходного изображения меньше или равно 3:2,
    /// то пропорциональная копия с максимальной стороной 130px.
    ///
    /// Если соотношение "ширина/высота" больше 3:2,
    /// то копия обрезанного слева изображения с максимальной стороной 130px и соотношением сторон 3:2.
    O,

    /// Если соотношение "ширина/высота" исходного изображения меньше или равно 3:2,
    /// то пропорциональная копия с максимальной стороной 200px.
    ///
    /// Если соотношение "ширина/высота" больше 3:2,
    /// то копия обрезанного слева и справа изображения с максимальной стороной 200px и соотношением сторон 3:2.
    P,

    /// Если соотношение "ширина/высота" исходного изображения меньше или равно 3:2,
    /// то пропорциональная копия с максимальной стороной 320px.
    ///
    /// Если соотношение "ширина/высота" больше 3:2,
    /// то копия обрезанного слева и справа изображения с максимальной стороной 320px и соотношением сторон 3:2.
    Q,

    /// Если соотношение "ширина/высота" исходного изображения меньше или равно 3:2,
    /// то пропорциональная копия с максимальной стороной 510px.
    ///
    /// Если соотношение "ширина/высота" больше 3:2,
    /// то копия обрезанного слева и справа изображения с максимальной стороной 510px и соотношением сторон 3:2
    R,

    /// Пропорциональная копия изображения с максимальной стороной 807px;
    Y,

    /// Пропорциональная копия изображения с максимальным размером 1080x1024;
    Z,

    /// Пропорциональная копия изображения с максимальным размером 2560x2048px.
    W,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PhotosList {
    // TODO
}

#[derive(Clone, Debug, Deserialize)]
pub struct Album {
    // TODO
}

#[derive(Clone, Debug, Deserialize)]
pub struct Video {
    // TODO
}

#[derive(Clone, Debug, Deserialize)]
pub struct Event {
    // TODO
}

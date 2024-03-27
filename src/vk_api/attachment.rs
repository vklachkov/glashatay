use serde::{Deserialize, Deserializer};

//
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase", remote = "Self")]
pub enum Attachment {
    Photo(Photo),
    PhotosList(PhotosList),
    Album(Album),
    Video(Video),
    Event(Event),
    Unsupported,
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

        Ok(Wrapper::deserialize(deserializer).map_or(Attachment::Unsupported, |w| w.inner))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Photo {
    // TODO
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

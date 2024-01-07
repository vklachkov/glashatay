use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Response<T> {
    // TODO: Add errors.
    pub response: T,
}

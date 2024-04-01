// @generated automatically by Diesel CLI.

diesel::table! {
    channels (id) {
        id -> Integer,
        tg_channel_id -> BigInt,
        vk_public_id -> Text,
        poll_interval_secs -> Integer,
        last_poll_timestamp -> Nullable<BigInt>,
        last_post_id -> Nullable<BigInt>,
        last_post_timestamp -> Nullable<BigInt>,
    }
}

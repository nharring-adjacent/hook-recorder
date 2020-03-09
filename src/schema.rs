table! {
    tags (tag_id) {
        tag_id -> Int4,
        url_suffix -> Varchar,
        created_at -> Timestamp,
        active -> Bool,
    }
}

table! {
    webhooks (id) {
        id -> Int4,
        headers -> Text,
        body -> Text,
        upload_time -> Timestamp,
        tag_id -> Nullable<Int4>,
    }
}

joinable!(webhooks -> tags (tag_id));

allow_tables_to_appear_in_same_query!(
    tags,
    webhooks,
);

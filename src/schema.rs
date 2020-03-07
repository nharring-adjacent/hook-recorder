table! {
    webhooks (id) {
        id -> Int4,
        headers -> Text,
        body -> Text,
        upload_time -> Timestamp,
        tag -> Nullable<Varchar>,
    }
}

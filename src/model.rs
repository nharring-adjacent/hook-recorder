use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Deserialize, Serialize, Clone, Debug)]
pub struct Webhook {
    pub id: i32,
    pub headers: String,
    pub body: String,
    pub upload_time: NaiveDateTime,
    pub tag_id: Option<i32>,
}

use super::schema::webhooks;
#[derive(Insertable)]
#[table_name = "webhooks"]
pub struct NewWebhook<'a> {
    pub headers: &'a str,
    pub body: &'a str,
    pub tag_id: i32,
}

#[derive(Queryable, Deserialize, Serialize, Clone, Debug)]
pub struct Tag {
    pub tag_id: i32,
    pub url_suffix: String,
    pub created_at: NaiveDateTime,
    pub active: bool,
}

use super::schema::tags;
#[derive(Insertable)]
#[table_name = "tags"]
pub struct NewTag {
    pub url_suffix: String,
    pub active: bool,
}

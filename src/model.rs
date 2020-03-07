use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Deserialize, Serialize, Clone, Debug)]
pub struct Webhook {
    pub id: i32,
    pub headers: String,
    pub body: String,
    pub upload_time: NaiveDateTime,
    pub tag: Option<String>,
}

use super::schema::webhooks;
#[derive(Insertable)]
#[table_name = "webhooks"]
pub struct NewWebhook<'a> {
    pub headers: &'a str,
    pub body: &'a str,
    pub tag: Option<&'a str>,
}

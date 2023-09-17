use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use madtofan_microservice_common::{
    notification::MessageResponse, repository::connection_pool::ServiceConnectionPool,
};
use sqlx::{query, query_as, types::time::OffsetDateTime, FromRow};

#[derive(FromRow)]
pub struct MessageEntity {
    pub id: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub channel: String,
    pub subject: String,
    pub message: String,
}

impl MessageEntity {
    pub fn into_message_response(self) -> MessageResponse {
        MessageResponse {
            id: self.id,
            subject: self.subject,
            message: self.message,
            channel: self.channel,
            date: self.created_at.unix_timestamp(),
        }
    }
}

#[async_trait]
pub trait MessageRepositoryTrait {
    async fn get_messages(
        &self,
        channels: Vec<String>,
        offset: i64,
        limit: i64,
    ) -> anyhow::Result<Vec<MessageEntity>>;
    async fn get_messages_count(&self, channels: Vec<String>) -> anyhow::Result<i64>;
    async fn add_message(
        &self,
        channel: &str,
        subject: &str,
        message: &str,
    ) -> anyhow::Result<MessageEntity>;
    async fn clean_messages(&self, date: i64) -> anyhow::Result<Vec<MessageEntity>>;
}

pub type DynMessageRepositoryTrait = Arc<dyn MessageRepositoryTrait + Send + Sync>;

#[derive(Clone)]
pub struct MessageRepository {
    pool: ServiceConnectionPool,
}

impl MessageRepository {
    pub fn new(pool: ServiceConnectionPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MessageRepositoryTrait for MessageRepository {
    async fn get_messages(
        &self,
        channels: Vec<String>,
        offset: i64,
        limit: i64,
    ) -> anyhow::Result<Vec<MessageEntity>> {
        query_as!(
            MessageEntity,
            r#"
                select *
                from notification_message
                where channel = any($1::text[])
                order by created_at desc
                limit $2::int
                offset $3::int
            "#,
            &channels,
            limit as i32,
            offset as i32,
        )
        .fetch_all(&self.pool)
        .await
        .context("an unexpected error occured while searching for group")
    }

    async fn get_messages_count(&self, channels: Vec<String>) -> anyhow::Result<i64> {
        let count_result = query!(
            "select count(*) from notification_message where channel = any($1::text[])",
            &channels
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count_result.count.unwrap())
    }

    async fn add_message(
        &self,
        channel: &str,
        subject: &str,
        message: &str,
    ) -> anyhow::Result<MessageEntity> {
        query_as!(
            MessageEntity,
            r#"
                insert into notification_message (
                        channel,
                        subject,
                        message
                    )
                values (
                        $1::varchar,
                        $2::varchar,
                        $3::varchar
                    )
                returning *
            "#,
            channel,
            subject,
            message,
        )
        .fetch_one(&self.pool)
        .await
        .context("an unexpected error occured while creating notification message")
    }

    async fn clean_messages(&self, date: i64) -> anyhow::Result<Vec<MessageEntity>> {
        let timestamp = OffsetDateTime::from_unix_timestamp(date);
        if let Err(e) = timestamp {
            return Err(e.into());
        };

        query_as!(
            MessageEntity,
            r#"
                delete from notification_message 
                where created_at < $1::timestamptz
                returning *
            "#,
            timestamp.unwrap(),
        )
        .fetch_all(&self.pool)
        .await
        .context("an unexpected error occured while removing the subscription group")
    }
}

use std::sync::Arc;

use async_trait::async_trait;
use madtofan_microservice_common::errors::ServiceResult;

use crate::repository::message::{DynMessageRepositoryTrait, MessageEntity};

#[async_trait]
pub trait MessageServiceTrait {
    async fn get_messages(
        &self,
        channel: Vec<String>,
        offset: i64,
        limit: i64,
    ) -> ServiceResult<Vec<MessageEntity>>;
    async fn get_messages_count(&self, channel: Vec<String>) -> ServiceResult<i64>;
    async fn add_message(
        &self,
        channel: String,
        subject: String,
        message: String,
    ) -> ServiceResult<MessageEntity>;
    async fn clear_messages(&self, date: i64) -> ServiceResult<Vec<MessageEntity>>;
}

pub type DynMessageServiceTrait = Arc<dyn MessageServiceTrait + Sync + Send>;

pub struct MessageService {
    repository: DynMessageRepositoryTrait,
}

impl MessageService {
    pub fn new(repository: DynMessageRepositoryTrait) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl MessageServiceTrait for MessageService {
    async fn get_messages(
        &self,
        channel: Vec<String>,
        offset: i64,
        limit: i64,
    ) -> ServiceResult<Vec<MessageEntity>> {
        let result = self.repository.get_messages(channel, offset, limit).await?;

        Ok(result)
    }

    async fn get_messages_count(&self, channel: Vec<String>) -> ServiceResult<i64> {
        let result = self.repository.get_messages_count(channel).await?;

        Ok(result)
    }

    async fn add_message(
        &self,
        channel: String,
        subject: String,
        message: String,
    ) -> ServiceResult<MessageEntity> {
        let result = self
            .repository
            .add_message(&channel, &subject, &message)
            .await?;

        Ok(result)
    }
    async fn clear_messages(&self, date: i64) -> ServiceResult<Vec<MessageEntity>> {
        let result = self.repository.clean_messages(date).await?;

        Ok(result)
    }
}

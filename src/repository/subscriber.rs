use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use madtofan_microservice_common::{
    notification::subscribers_response::Subscriber,
    repository::connection_pool::ServiceConnectionPool,
};
use sqlx::{query_as, types::time::OffsetDateTime, FromRow};

use super::group::GroupEntity;

#[derive(FromRow)]
pub struct SubscriberEntity {
    pub id: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub user_id: i64,
    pub group_id: i64,
}

impl SubscriberEntity {
    pub fn into_subscriber_response(self) -> Subscriber {
        Subscriber {
            user_id: self.user_id,
        }
    }
}

#[async_trait]
pub trait SubscriberRepositoryTrait {
    async fn add_subscriber(
        &self,
        user_id: i64,
        group: &GroupEntity,
    ) -> anyhow::Result<SubscriberEntity>;
    async fn remove_subscriber(
        &self,
        user_id: i64,
        group_name: &str,
    ) -> anyhow::Result<Option<SubscriberEntity>>;
    async fn list_subs_by_group(
        &self,
        group: &GroupEntity,
    ) -> anyhow::Result<Vec<SubscriberEntity>>;
}

pub type DynSubscriberRepositoryTrait = Arc<dyn SubscriberRepositoryTrait + Send + Sync>;

#[derive(Clone)]
pub struct SubscriberRepository {
    pool: ServiceConnectionPool,
}

impl SubscriberRepository {
    pub fn new(pool: ServiceConnectionPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubscriberRepositoryTrait for SubscriberRepository {
    async fn add_subscriber(
        &self,
        user_id: i64,
        group: &GroupEntity,
    ) -> anyhow::Result<SubscriberEntity> {
        query_as!(
            SubscriberEntity,
            r#"
                insert into notification_subscriber (
                        user_id,
                        group_id
                    )
                values (
                        $1::bigint,
                        $2::bigint
                    )
                returning *
            "#,
            user_id,
            group.id,
        )
        .fetch_one(&self.pool)
        .await
        .context("an unexpected error occured while creating the subscriber")
    }

    async fn remove_subscriber(
        &self,
        user_id: i64,
        group_name: &str,
    ) -> anyhow::Result<Option<SubscriberEntity>> {
        query_as!(
            SubscriberEntity,
            r#"
                delete from notification_subscriber 
                where 
                    user_id = $1::bigint 
                    and group_id = (select id from notification_group where name = $2::varchar)
                returning *
            "#,
            user_id,
            group_name,
        )
        .fetch_optional(&self.pool)
        .await
        .context("an unexpected error occured while removing the subscriber")
    }

    async fn list_subs_by_group(
        &self,
        group: &GroupEntity,
    ) -> anyhow::Result<Vec<SubscriberEntity>> {
        query_as!(
            SubscriberEntity,
            r#"
                select
                    id,
                    user_id,
                    group_id,
                    created_at,
                    updated_at
                from notification_subscriber
                where group_id = $1::bigint
            "#,
            group.id
        )
        .fetch_all(&self.pool)
        .await
        .context("an unexpected error occured while search for subscribers by group")
    }
}

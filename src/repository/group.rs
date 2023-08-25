use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use madtofan_microservice_common::repository::connection_pool::ServiceConnectionPool;
use sqlx::{query_as, types::time::OffsetDateTime, FromRow};

#[derive(FromRow)]
pub struct GroupEntity {
    pub id: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub name: String,
}

#[async_trait]
pub trait GroupRepositoryTrait {
    async fn get_group(&self, name: &str) -> anyhow::Result<Option<GroupEntity>>;
    async fn add_group(&self, name: &str) -> anyhow::Result<GroupEntity>;
    async fn remove_group(&self, name: &str) -> anyhow::Result<Option<GroupEntity>>;
}

pub type DynGroupRepositoryTrait = Arc<dyn GroupRepositoryTrait + Send + Sync>;

#[derive(Clone)]
pub struct GroupRepository {
    pool: ServiceConnectionPool,
}

impl GroupRepository {
    pub fn new(pool: ServiceConnectionPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GroupRepositoryTrait for GroupRepository {
    async fn get_group(&self, name: &str) -> anyhow::Result<Option<GroupEntity>> {
        query_as!(
            GroupEntity,
            r#"
                select
                    id,
                    name,
                    created_at,
                    updated_at
                from notification_group
                where name = $1::varchar
            "#,
            name,
        )
        .fetch_optional(&self.pool)
        .await
        .context("an unexpected error occured while searching for group")
    }

    async fn add_group(&self, name: &str) -> anyhow::Result<GroupEntity> {
        query_as!(
            GroupEntity,
            r#"
                insert into notification_group (
                        name
                    )
                values (
                        $1::varchar
                    )
                returning *
            "#,
            name,
        )
        .fetch_one(&self.pool)
        .await
        .context("an unexpected error occured while creating the subscription group")
    }

    async fn remove_group(&self, name: &str) -> anyhow::Result<Option<GroupEntity>> {
        query_as!(
            GroupEntity,
            r#"
                delete from notification_group 
                where name = $1::varchar
                returning *
            "#,
            name,
        )
        .fetch_optional(&self.pool)
        .await
        .context("an unexpected error occured while removing the subscription group")
    }
}

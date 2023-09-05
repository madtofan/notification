use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use madtofan_microservice_common::{
    notification::groups_response::Group, repository::connection_pool::ServiceConnectionPool,
};
use sqlx::{query_as, types::time::OffsetDateTime, FromRow};

#[derive(FromRow)]
pub struct GroupEntity {
    pub id: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub name: String,
}

impl GroupEntity {
    pub fn into_group_response(self) -> Group {
        Group { name: self.name }
    }
}

#[async_trait]
pub trait GroupRepositoryTrait {
    async fn get_group(&self, name: &str) -> anyhow::Result<Option<GroupEntity>>;
    async fn add_group(&self, name: &str) -> anyhow::Result<GroupEntity>;
    async fn remove_group(&self, name: &str) -> anyhow::Result<Option<GroupEntity>>;
    async fn list_groups_by_sub(&self, user_id: i64) -> anyhow::Result<Vec<GroupEntity>>;
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

    async fn list_groups_by_sub(&self, user_id: i64) -> anyhow::Result<Vec<GroupEntity>> {
        query_as!(
            GroupEntity,
            r#"
                select
                    ng.id as id,
                    ng.name as name,
                    ng.created_at as created_at,
                    ng.updated_at as updated_at
                from notification_group as ng
                join notification_subscriber as ns
                on ng.id = ns.group_id
                where ns.user_id = $1::bigint
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .context("an unexpected error occured while search for subscribers by group")
    }
}

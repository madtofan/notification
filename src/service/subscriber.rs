use std::sync::Arc;

use async_trait::async_trait;
use madtofan_microservice_common::errors::{ServiceError, ServiceResult};
use tracing::log::{error, info};

use crate::repository::{
    group::DynGroupRepositoryTrait,
    subscriber::{DynSubscriberRepositoryTrait, SubscriberEntity},
};

#[async_trait]
pub trait SubscriberServiceTrait {
    async fn list_subs_by_group(&self, group_name: String) -> ServiceResult<Vec<SubscriberEntity>>;
    async fn add_subscriber(&self, user_id: i64, group_name: String) -> ServiceResult<()>;
    async fn remove_subscriber(&self, user_id: i64) -> ServiceResult<()>;
}

pub type DynSubscriberServiceTrait = Arc<dyn SubscriberServiceTrait + Sync + Send>;

pub struct SubscriberService {
    subscriber_repository: DynSubscriberRepositoryTrait,
    group_repository: DynGroupRepositoryTrait,
}

impl SubscriberService {
    pub fn new(
        subscriber_repository: DynSubscriberRepositoryTrait,
        group_repository: DynGroupRepositoryTrait,
    ) -> Self {
        Self {
            subscriber_repository,
            group_repository,
        }
    }
}

#[async_trait]
impl SubscriberServiceTrait for SubscriberService {
    async fn list_subs_by_group(&self, group_name: String) -> ServiceResult<Vec<SubscriberEntity>> {
        let existing_group = self.group_repository.get_group(&group_name).await?;

        match existing_group {
            Some(group) => {
                info!("listing subscriber from group {:?}", &group_name);
                let subscribers = self
                    .subscriber_repository
                    .list_subs_by_group(&group)
                    .await?;

                info!("successfully obtained list of subscriber from group");
                Ok(subscribers)
            }
            None => {
                error!("group {:?} does not exists", &group_name);
                Err(ServiceError::ObjectConflict(String::from(
                    "group name does not exist",
                )))
            }
        }
    }

    async fn add_subscriber(&self, user_id: i64, group_name: String) -> ServiceResult<()> {
        let existing_group = self.group_repository.get_group(&group_name).await?;

        match existing_group {
            Some(group) => {
                info!("add subscriber into group {:?}", &group_name);
                self.subscriber_repository
                    .add_subscriber(user_id, &group)
                    .await?;

                info!("successfully added subscriber into group");
                Ok(())
            }
            None => {
                error!("group {:?} does not exists", &group_name);
                Err(ServiceError::ObjectConflict(String::from(
                    "group name does not exist",
                )))
            }
        }
    }

    async fn remove_subscriber(&self, user_id: i64) -> ServiceResult<()> {
        self.subscriber_repository
            .remove_subscriber(user_id)
            .await?;
        Ok(())
    }
}

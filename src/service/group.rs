use std::sync::Arc;

use async_trait::async_trait;
use madtofan_microservice_common::errors::{ServiceError, ServiceResult};
use tracing::{error, info};

use crate::repository::group::{DynGroupRepositoryTrait, GroupEntity};

#[async_trait]
pub trait GroupServiceTrait {
    async fn add_group(
        &self,
        name: String,
        admin_email: String,
        token: String,
    ) -> ServiceResult<()>;
    async fn remove_group(
        &self,
        name: String,
        admin_email: String,
    ) -> ServiceResult<Option<GroupEntity>>;
    async fn list_groups_by_sub(&self, user_id: i64) -> ServiceResult<Vec<GroupEntity>>;
    async fn verify_token(&self, name: String, token: String) -> ServiceResult<bool>;
}

pub type DynGroupServiceTrait = Arc<dyn GroupServiceTrait + Sync + Send>;

pub struct GroupService {
    repository: DynGroupRepositoryTrait,
}

impl GroupService {
    pub fn new(repository: DynGroupRepositoryTrait) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl GroupServiceTrait for GroupService {
    async fn add_group(
        &self,
        name: String,
        admin_email: String,
        token: String,
    ) -> ServiceResult<()> {
        let existing_group = self.repository.get_group(&name).await?;

        if existing_group.is_some() {
            error!("group {:?} already exists", &name);
            return Err(ServiceError::ObjectConflict(String::from(
                "group name is taken",
            )));
        }

        info!("creating group {:?}", &name);
        self.repository
            .add_group(&name, &admin_email, &token)
            .await?;

        info!("group successfully created");

        Ok(())
    }

    async fn remove_group(
        &self,
        name: String,
        admin_email: String,
    ) -> ServiceResult<Option<GroupEntity>> {
        let existing_group = self.repository.get_group(&name).await?;

        if existing_group.is_none() {
            error!("group {:?} does not exist", &name);
            return Err(ServiceError::ObjectConflict(String::from(
                "group does not exist",
            )));
        }

        info!("deleting group {:?}", &name);
        let removed_group = self.repository.remove_group(&name, &admin_email).await?;

        if removed_group.is_none() {
            error!(
                "incorrect admin email ({:?}) used for group {:?}",
                &admin_email, &name
            );
            return Err(ServiceError::ObjectConflict(String::from(
                "group does not exist",
            )));
        }
        info!("group successfully removed");

        Ok(removed_group)
    }

    async fn list_groups_by_sub(&self, user_id: i64) -> ServiceResult<Vec<GroupEntity>> {
        info!("listing group from subscriber {:?}", user_id);
        let groups = self.repository.list_groups_by_sub(user_id).await?;

        info!("successfully obtained list of groups from subscriber");
        Ok(groups)
    }

    async fn verify_token(&self, name: String, token: String) -> ServiceResult<bool> {
        let group_option = self.repository.get_group(&name).await?;
        match group_option {
            Some(group) => Ok(group.token.eq(&token)),
            None => Err(ServiceError::NotFound(String::from("group not found"))),
        }
    }
}

use madtofan_microservice_common::errors::ServiceResult;
use mockall::lazy_static;
use tracing::info;

use crate::repository::group::DynGroupRepositoryTrait;

lazy_static! {
    static ref GROUP_SERVER_1: &'static str = "server_one";
    static ref GROUP_SERVER_2: &'static str = "server_two";
}

pub struct SeedService {
    group_repository: DynGroupRepositoryTrait,
}

impl SeedService {
    pub fn new(group_repository: DynGroupRepositoryTrait) -> Self {
        Self { group_repository }
    }

    pub async fn seed(&self) -> ServiceResult<()> {
        let group_server_one_name = String::from(*GROUP_SERVER_1);
        let group_server_two_name = String::from(*GROUP_SERVER_2);

        let existing_server_one = self
            .group_repository
            .get_group(&group_server_one_name)
            .await?;
        if existing_server_one.is_some() {
            info!("data has already been seeded, bypassing test data setup");
            return Ok(());
        }

        self.group_repository
            .add_group(&group_server_one_name)
            .await?;
        self.group_repository
            .add_group(&group_server_two_name)
            .await?;
        Ok(())
    }
}

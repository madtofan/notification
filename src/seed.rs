use madtofan_microservice_common::errors::ServiceResult;
use mockall::lazy_static;
use tracing::info;

use crate::repository::group::DynGroupRepositoryTrait;

lazy_static! {
    static ref GROUP_SERVER_1: &'static str = "server_one";
    static ref GROUP_ADMIN_1: &'static str = "ahmadclab@gmail.com";
    static ref TOKEN_SERVER_1: &'static str = "server_one_token";
    static ref GROUP_SERVER_2: &'static str = "server_two";
    static ref GROUP_ADMIN_2: &'static str = "manzainul@gmail.com";
    static ref TOKEN_SERVER_2: &'static str = "server_two_token";
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
        let _group_admin_one = String::from(*GROUP_ADMIN_1);
        let _token_one = String::from(*TOKEN_SERVER_1);
        let _group_server_two_name = String::from(*GROUP_SERVER_2);
        let _group_admin_two = String::from(*GROUP_ADMIN_2);
        let _token_two = String::from(*TOKEN_SERVER_2);

        let existing_server_one = self
            .group_repository
            .get_group(&group_server_one_name)
            .await?;
        if existing_server_one.is_some() {
            info!("data has already been seeded, bypassing test data setup");
            return Ok(());
        }

        // self.group_repository
        //     .add_group(&group_server_one_name, &group_admin_one, &token_one)
        //     .await?;
        // self.group_repository
        //     .add_group(&group_server_two_name, &group_admin_two, &token_two)
        //     .await?;
        Ok(())
    }
}

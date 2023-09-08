pub mod group;
pub mod subscriber;

#[cfg(test)]
pub mod test {
    use std::sync::Arc;

    use sqlx::PgPool;

    use crate::repository::group::{DynGroupRepositoryTrait, GroupRepository};

    use super::subscriber::{DynSubscriberRepositoryTrait, SubscriberRepository};

    struct AllTraits {
        subscriber_repository: DynSubscriberRepositoryTrait,
        group_repository: DynGroupRepositoryTrait,
    }

    fn initialize_handler(pool: PgPool) -> AllTraits {
        let subscriber_repository =
            Arc::new(SubscriberRepository::new(pool.clone())) as DynSubscriberRepositoryTrait;
        let group_repository =
            Arc::new(GroupRepository::new(pool.clone())) as DynGroupRepositoryTrait;

        AllTraits {
            subscriber_repository,
            group_repository,
        }
    }

    #[sqlx::test]
    async fn add_group_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group_to_add = "group_to_add";
        traits
            .group_repository
            .add_group(group_to_add, "admin_email", "token")
            .await?;

        let obtained_group = traits.group_repository.get_group(group_to_add).await?;
        assert_eq!(obtained_group.unwrap().name, group_to_add);

        Ok(())
    }

    #[sqlx::test]
    async fn remove_group_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group_to_remove = "group_to_remove";
        let group_to_remove_admin_email = "admin_email";
        traits
            .group_repository
            .add_group(group_to_remove, group_to_remove_admin_email, "token")
            .await?;
        traits
            .group_repository
            .remove_group(group_to_remove, group_to_remove_admin_email)
            .await?;

        let obtained_group = traits.group_repository.get_group(group_to_remove).await?;
        assert!(obtained_group.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn list_groups_by_sub(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group_1_name = "group_1_name";
        let group1 = traits
            .group_repository
            .add_group(group_1_name, "admin_email", "token")
            .await?;

        let group_2_name = "group_2_name";
        let group2 = traits
            .group_repository
            .add_group(group_2_name, "admin_email", "token")
            .await?;

        let sub_1_id = 0;
        traits
            .subscriber_repository
            .add_subscriber(sub_1_id, &group1)
            .await?;

        let sub_2_id = 1;
        traits
            .subscriber_repository
            .add_subscriber(sub_2_id, &group2)
            .await?;

        let group_list = traits.group_repository.list_groups_by_sub(sub_1_id).await?;

        assert_eq!(group_list.len(), 1);
        assert_eq!(group_list.first().unwrap().name, group_1_name);

        Ok(())
    }

    #[sqlx::test]
    async fn list_subs_by_group_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group_1_name = "group_1_name";
        let group1 = traits
            .group_repository
            .add_group(group_1_name, "admin_email", "token")
            .await?;

        let group_2_name = "group_2_name";
        let group2 = traits
            .group_repository
            .add_group(group_2_name, "admin_email", "token")
            .await?;

        let sub_1_id = 0;
        let sub1 = traits
            .subscriber_repository
            .add_subscriber(sub_1_id, &group1)
            .await?;

        let sub_2_id = 1;
        traits
            .subscriber_repository
            .add_subscriber(sub_2_id, &group2)
            .await?;

        let subscribers_list = traits
            .subscriber_repository
            .list_subs_by_group(&group1)
            .await?;

        assert_eq!(subscribers_list.len(), 1);
        assert_eq!(subscribers_list.first().unwrap().user_id, sub1.user_id);

        Ok(())
    }

    #[sqlx::test]
    async fn remove_subscriber_from_group_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group_name = "group_name";
        let group = traits
            .group_repository
            .add_group(group_name, "admin_email", "token")
            .await?;

        let sub_1_id = 0;
        traits
            .subscriber_repository
            .add_subscriber(sub_1_id, &group)
            .await?;

        let sub_2_address = 1;
        let sub2 = traits
            .subscriber_repository
            .add_subscriber(sub_2_address, &group)
            .await?;

        traits
            .subscriber_repository
            .remove_subscriber(sub_1_id, group_name)
            .await?;

        let subscribers_list = traits
            .subscriber_repository
            .list_subs_by_group(&group)
            .await?;

        assert_eq!(subscribers_list.len(), 1);
        assert_eq!(subscribers_list.first().unwrap().user_id, sub2.user_id);

        Ok(())
    }
}

pub mod group;
pub mod subscriber;

#[cfg(test)]
pub mod test {
    use std::sync::Arc;

    use sqlx::PgPool;

    use crate::{
        repository::{
            group::{DynGroupRepositoryTrait, GroupRepository},
            subscriber::{DynSubscriberRepositoryTrait, SubscriberRepository},
        },
        service::{
            group::{DynGroupServiceTrait, GroupService},
            subscriber::{DynSubscriberServiceTrait, SubscriberService},
        },
    };

    struct AllTraits {
        subscriber_repository: DynSubscriberRepositoryTrait,
        group_repository: DynGroupRepositoryTrait,
        subscriber_service: DynSubscriberServiceTrait,
        group_service: DynGroupServiceTrait,
    }

    fn initialize_handler(pool: PgPool) -> AllTraits {
        let subscriber_repository =
            Arc::new(SubscriberRepository::new(pool.clone())) as DynSubscriberRepositoryTrait;
        let group_repository =
            Arc::new(GroupRepository::new(pool.clone())) as DynGroupRepositoryTrait;
        let subscriber_service = Arc::new(SubscriberService::new(
            subscriber_repository.clone(),
            group_repository.clone(),
        )) as DynSubscriberServiceTrait;
        let group_service =
            Arc::new(GroupService::new(group_repository.clone())) as DynGroupServiceTrait;

        AllTraits {
            subscriber_repository,
            subscriber_service,
            group_repository,
            group_service,
        }
    }

    #[sqlx::test]
    async fn add_group_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group_name = "group_name";

        traits
            .group_service
            .add_group(group_name.to_string())
            .await?;

        let group = traits.group_repository.get_group(group_name).await?;

        assert_eq!(group.unwrap().name, group_name);

        Ok(())
    }

    #[sqlx::test]
    async fn remove_group_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group_to_remove_name = "group_to_remove";
        traits
            .group_repository
            .add_group(group_to_remove_name)
            .await?;

        let removed_group = traits
            .group_service
            .remove_group(group_to_remove_name.to_string())
            .await?;

        assert!(removed_group.is_some());
        assert_eq!(removed_group.unwrap().name, group_to_remove_name);

        let obtained_group = traits
            .group_repository
            .get_group(group_to_remove_name)
            .await?;

        assert!(obtained_group.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn list_subs_by_group_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group1_name = "group1_name";
        let group1 = traits.group_repository.add_group(group1_name).await?;
        let group2 = traits.group_repository.add_group("group2_name").await?;

        let sub1_id = 0;
        traits
            .subscriber_repository
            .add_subscriber(sub1_id, &group1)
            .await?;
        let sub2_id = 1;
        traits
            .subscriber_repository
            .add_subscriber(sub2_id, &group2)
            .await?;

        let subs_list = traits
            .subscriber_service
            .list_subs_by_group(group1_name.to_string())
            .await?;

        assert_eq!(subs_list.len(), 1);
        assert_eq!(subs_list.first().unwrap().user_id, sub1_id);

        Ok(())
    }

    #[sqlx::test]
    async fn remove_subcriber_from_group_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let group_name = "group_name";
        let group = traits.group_repository.add_group(group_name).await?;

        let sub1_id = 0;
        traits
            .subscriber_repository
            .add_subscriber(sub1_id, &group)
            .await?;
        let sub2_id = 1;
        traits
            .subscriber_repository
            .add_subscriber(sub2_id, &group)
            .await?;

        traits.subscriber_service.remove_subscriber(sub1_id).await?;
        let subs_list = traits
            .subscriber_repository
            .list_subs_by_group(&group)
            .await?;

        assert_eq!(subs_list.len(), 1);
        assert_eq!(subs_list.first().unwrap().user_id, sub2_id);

        Ok(())
    }
}
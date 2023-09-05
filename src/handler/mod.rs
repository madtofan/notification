pub mod notification;

#[cfg(test)]
pub mod test {
    use std::sync::Arc;

    use madtofan_microservice_common::notification::{
        notification_server::Notification, AddGroupRequest, AddSubscriberRequest, GetGroupsRequest,
        GetSubscribersRequest, RemoveGroupRequest, RemoveSubscriberRequest,
    };
    use sqlx::PgPool;
    use tonic::Request;

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

    use super::notification::RequestHandler;

    struct AllTraits {
        subscriber_repository: DynSubscriberRepositoryTrait,
        group_repository: DynGroupRepositoryTrait,
        handler: RequestHandler,
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
        let handler = RequestHandler::new(subscriber_service.clone(), group_service.clone());

        AllTraits {
            subscriber_repository,
            group_repository,
            handler,
        }
    }

    #[sqlx::test]
    async fn add_subscriber_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let group_name = "group_name";
        let created_group = all_traits.group_repository.add_group(group_name).await?;

        let sub_id = 0;
        let request = Request::new(AddSubscriberRequest {
            user_id: sub_id,
            group: group_name.to_string(),
        });

        all_traits.handler.add_subscriber(request).await?;
        let added_sub = all_traits
            .subscriber_repository
            .list_subs_by_group(&created_group)
            .await?;
        assert_eq!(added_sub.first().unwrap().user_id, sub_id);

        Ok(())
    }

    #[sqlx::test]
    async fn remove_subscriber_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let group_name = "group_name";
        let group = all_traits.group_repository.add_group(group_name).await?;

        let sub1_id = 0;
        all_traits
            .subscriber_repository
            .add_subscriber(sub1_id, &group)
            .await?;
        let sub2_id = 1;
        all_traits
            .subscriber_repository
            .add_subscriber(sub2_id, &group)
            .await?;

        let request = Request::new(RemoveSubscriberRequest { user_id: sub1_id });

        all_traits.handler.remove_subscriber(request).await?;
        let subs_list = all_traits
            .subscriber_repository
            .list_subs_by_group(&group)
            .await?;

        assert_eq!(subs_list.len(), 1);
        assert_eq!(subs_list.first().unwrap().user_id, sub2_id);

        Ok(())
    }

    #[sqlx::test]
    async fn add_group_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let group_name = "group_name";

        let request = Request::new(AddGroupRequest {
            name: group_name.to_string(),
        });

        all_traits.handler.add_group(request).await?;

        let group = all_traits.group_repository.get_group(group_name).await?;

        assert_eq!(group.unwrap().name, group_name);

        Ok(())
    }

    #[sqlx::test]
    async fn remove_group_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let group_to_remove_name = "group_to_remove";
        all_traits
            .group_repository
            .add_group(group_to_remove_name)
            .await?;

        let request = Request::new(RemoveGroupRequest {
            name: group_to_remove_name.to_string(),
        });

        all_traits.handler.remove_group(request).await?;

        let obtained_group = all_traits
            .group_repository
            .get_group(group_to_remove_name)
            .await?;

        assert!(obtained_group.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn get_subscribers_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let group1_name = "group1_name";
        let group1 = all_traits.group_repository.add_group(group1_name).await?;
        let group2 = all_traits.group_repository.add_group("group2_name").await?;

        let sub1_id = 0;
        all_traits
            .subscriber_repository
            .add_subscriber(sub1_id, &group1)
            .await?;
        let sub2_id = 1;
        all_traits
            .subscriber_repository
            .add_subscriber(sub2_id, &group2)
            .await?;

        let request = Request::new(GetSubscribersRequest {
            group: group1_name.to_string(),
        });

        let subs_list = all_traits
            .handler
            .get_subscribers(request)
            .await?
            .into_inner()
            .subscribers;

        assert_eq!(subs_list.len(), 1);
        assert_eq!(subs_list.first().unwrap().user_id, sub1_id);

        Ok(())
    }

    #[sqlx::test]
    async fn get_groups_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let group1_name = "group1_name";
        let group1 = all_traits.group_repository.add_group(group1_name).await?;
        let group2 = all_traits.group_repository.add_group("group2_name").await?;

        let sub1_id = 0;
        all_traits
            .subscriber_repository
            .add_subscriber(sub1_id, &group1)
            .await?;
        let sub2_id = 1;
        all_traits
            .subscriber_repository
            .add_subscriber(sub2_id, &group2)
            .await?;

        let request = Request::new(GetGroupsRequest { user_id: sub1_id });

        let groups_list = all_traits
            .handler
            .get_groups(request)
            .await?
            .into_inner()
            .groups;

        assert_eq!(groups_list.len(), 1);
        assert_eq!(groups_list.first().unwrap().name, group1_name);

        Ok(())
    }
}

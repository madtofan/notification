pub mod notification;

#[cfg(test)]
pub mod test {
    use std::{sync::Arc, thread, time};

    use madtofan_microservice_common::notification::{
        notification_server::Notification, AddGroupRequest, AddMessageRequest,
        AddSubscriberRequest, ClearMessagesRequest, GetGroupsRequest, GetMessagesRequest,
        GetSubscribersRequest, RemoveGroupRequest, RemoveSubscriberRequest, VerifyTokenRequest,
    };
    use sqlx::PgPool;
    use tonic::Request;

    use crate::{
        repository::{
            group::{DynGroupRepositoryTrait, GroupRepository},
            message::{DynMessageRepositoryTrait, MessageRepository},
            subscriber::{DynSubscriberRepositoryTrait, SubscriberRepository},
        },
        service::{
            group::{DynGroupServiceTrait, GroupService},
            message::{DynMessageServiceTrait, MessageService},
            subscriber::{DynSubscriberServiceTrait, SubscriberService},
        },
    };

    use super::notification::RequestHandler;

    struct AllTraits {
        subscriber_repository: DynSubscriberRepositoryTrait,
        group_repository: DynGroupRepositoryTrait,
        message_repository: DynMessageRepositoryTrait,
        handler: RequestHandler,
    }

    fn initialize_handler(pool: PgPool) -> AllTraits {
        let subscriber_repository =
            Arc::new(SubscriberRepository::new(pool.clone())) as DynSubscriberRepositoryTrait;
        let group_repository =
            Arc::new(GroupRepository::new(pool.clone())) as DynGroupRepositoryTrait;
        let message_repository =
            Arc::new(MessageRepository::new(pool.clone())) as DynMessageRepositoryTrait;
        let subscriber_service = Arc::new(SubscriberService::new(
            subscriber_repository.clone(),
            group_repository.clone(),
        )) as DynSubscriberServiceTrait;
        let group_service =
            Arc::new(GroupService::new(group_repository.clone())) as DynGroupServiceTrait;
        let message_service =
            Arc::new(MessageService::new(message_repository.clone())) as DynMessageServiceTrait;
        let handler = RequestHandler::new(
            subscriber_service.clone(),
            group_service.clone(),
            message_service.clone(),
        );

        AllTraits {
            subscriber_repository,
            group_repository,
            message_repository,
            handler,
        }
    }

    #[sqlx::test]
    async fn add_subscriber_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let group_name = "group_name";
        let created_group = all_traits
            .group_repository
            .add_group(group_name, "admin_email", "token")
            .await?;

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
        let group = all_traits
            .group_repository
            .add_group(group_name, "admin_email", "token")
            .await?;

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

        let request = Request::new(RemoveSubscriberRequest {
            user_id: sub1_id,
            group: group_name.to_string(),
        });

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
            admin_email: "admin_email".to_string(),
            token: "token".to_string(),
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
        let group_to_remove_admin_email = "admin_email";
        all_traits
            .group_repository
            .add_group(group_to_remove_name, group_to_remove_admin_email, "token")
            .await?;

        let request = Request::new(RemoveGroupRequest {
            name: group_to_remove_name.to_string(),
            admin_email: group_to_remove_admin_email.to_string(),
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
        let group1 = all_traits
            .group_repository
            .add_group(group1_name, "admin_email", "token")
            .await?;
        let group2 = all_traits
            .group_repository
            .add_group("group2_name", "admin_email", "token")
            .await?;

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
        let group1 = all_traits
            .group_repository
            .add_group(group1_name, "admin_email", "token")
            .await?;
        let group2 = all_traits
            .group_repository
            .add_group("group2_name", "admin_email", "token")
            .await?;

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

    #[sqlx::test]
    async fn verify_token_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let group_name = "group_name";
        let token = "token";
        all_traits
            .group_repository
            .add_group(group_name, "admin_email", token)
            .await?;

        let request = Request::new(VerifyTokenRequest {
            name: group_name.to_string(),
            token: token.to_string(),
        });

        let verify_valid_token = all_traits.handler.verify_token(request).await?;
        assert!(verify_valid_token.into_inner().valid);

        let request = Request::new(VerifyTokenRequest {
            name: group_name.to_string(),
            token: "invalid_token".to_string(),
        });

        let verify_invalid_token = all_traits.handler.verify_token(request).await?;
        assert!(!verify_invalid_token.into_inner().valid);

        Ok(())
    }

    #[sqlx::test]
    async fn add_message_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let message = "test_message";
        let add_message_request = Request::new(AddMessageRequest {
            channel: "channel1".to_string(),
            subject: "subject".to_string(),
            message: message.to_string(),
        });
        let request = all_traits.handler.add_message(add_message_request).await?;

        assert_eq!(request.into_inner().message, message);

        Ok(())
    }

    #[sqlx::test]
    async fn get_messages_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let channel = "channel1";
        let channels = vec![channel.to_string()];
        let message = "test_message";

        all_traits
            .message_repository
            .add_message(channel, "subject", "message")
            .await?;
        all_traits
            .message_repository
            .add_message(channel, "subject", message)
            .await?;

        let get_message_request = Request::new(GetMessagesRequest {
            channels,
            offset: 0,
            limit: 10,
        });
        let request = all_traits.handler.get_messages(get_message_request).await?;

        let request_values = request.into_inner();
        assert_eq!(request_values.messages.len(), 2);
        assert_eq!(request_values.count, 2);
        assert_eq!(request_values.messages.first().unwrap().message, message);

        Ok(())
    }

    #[sqlx::test]
    async fn clear_messages_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let channel = "channel1";
        let channels = vec![channel.to_string()];
        let message = "test_message";
        let first_message = all_traits
            .message_repository
            .add_message(channel, "subject", "message")
            .await?;

        let first_message_time = first_message.created_at;
        let two_seconds = time::Duration::from_millis(2000);
        thread::sleep(two_seconds);

        all_traits
            .message_repository
            .add_message(channel, "subject", message)
            .await?;

        let clear_message_request = Request::new(ClearMessagesRequest {
            date: first_message_time.unix_timestamp() + 1,
        });
        all_traits
            .handler
            .clear_messages(clear_message_request)
            .await?;

        let left_messages = all_traits
            .message_repository
            .get_messages(channels, 0, 50)
            .await?;

        assert_eq!(left_messages.len(), 1);
        assert_eq!(left_messages.first().unwrap().message, message);

        Ok(())
    }
}

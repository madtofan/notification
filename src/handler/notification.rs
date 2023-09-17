use tonic::{Request, Response, Status};

use madtofan_microservice_common::notification::{
    groups_response::Group, notification_server::Notification, subscribers_response::Subscriber,
    AddGroupRequest, AddMessageRequest, AddSubscriberRequest, ClearMessagesRequest,
    GetGroupsRequest, GetMessagesRequest, GetSubscribersRequest, GroupsResponse, MessageResponse,
    MessagesResponse, NotificationResponse, RemoveGroupRequest, RemoveSubscriberRequest,
    SubscribersResponse, VerifyTokenRequest, VerifyTokenResponse,
};

use crate::service::{
    group::DynGroupServiceTrait, message::DynMessageServiceTrait,
    subscriber::DynSubscriberServiceTrait,
};

pub struct RequestHandler {
    subscriber_service: DynSubscriberServiceTrait,
    group_service: DynGroupServiceTrait,
    message_service: DynMessageServiceTrait,
}

impl RequestHandler {
    pub fn new(
        subscriber_service: DynSubscriberServiceTrait,
        group_service: DynGroupServiceTrait,
        message_service: DynMessageServiceTrait,
    ) -> Self {
        Self {
            subscriber_service,
            group_service,
            message_service,
        }
    }
}

#[tonic::async_trait]
impl Notification for RequestHandler {
    async fn add_subscriber(
        &self,
        request: Request<AddSubscriberRequest>,
    ) -> Result<Response<NotificationResponse>, Status> {
        let req = request.into_inner();

        self.subscriber_service
            .add_subscriber(req.user_id, req.group)
            .await?;

        Ok(Response::new(NotificationResponse {
            message: String::from("Successfully add subscriber!"),
        }))
    }

    async fn remove_subscriber(
        &self,
        request: Request<RemoveSubscriberRequest>,
    ) -> Result<Response<NotificationResponse>, Status> {
        let req = request.into_inner();

        self.subscriber_service
            .remove_subscriber(req.user_id, req.group)
            .await?;

        Ok(Response::new(NotificationResponse {
            message: String::from("Successfully removed subscriber!"),
        }))
    }

    async fn add_group(
        &self,
        request: Request<AddGroupRequest>,
    ) -> Result<Response<NotificationResponse>, Status> {
        let req = request.into_inner();

        self.group_service
            .add_group(req.name, req.admin_email, req.token)
            .await?;

        Ok(Response::new(NotificationResponse {
            message: String::from("Successfully add group!"),
        }))
    }

    async fn remove_group(
        &self,
        request: Request<RemoveGroupRequest>,
    ) -> Result<Response<NotificationResponse>, Status> {
        let req = request.into_inner();

        self.group_service
            .remove_group(req.name, req.admin_email)
            .await?;

        Ok(Response::new(NotificationResponse {
            message: String::from("Successfully removed group!"),
        }))
    }

    async fn get_subscribers(
        &self,
        request: Request<GetSubscribersRequest>,
    ) -> Result<Response<SubscribersResponse>, Status> {
        let req = request.into_inner();

        let subscriber_entity = self
            .subscriber_service
            .list_subs_by_group(req.group)
            .await?;

        let subscribers = subscriber_entity
            .into_iter()
            .map(|sub| sub.into_subscriber_response())
            .collect::<Vec<Subscriber>>();

        Ok(Response::new(SubscribersResponse { subscribers }))
    }

    async fn get_groups(
        &self,
        request: Request<GetGroupsRequest>,
    ) -> Result<Response<GroupsResponse>, Status> {
        let req = request.into_inner();

        let subscriber_entity = self.group_service.list_groups_by_sub(req.user_id).await?;

        let groups = subscriber_entity
            .into_iter()
            .map(|sub| sub.into_group_response())
            .collect::<Vec<Group>>();

        Ok(Response::new(GroupsResponse { groups }))
    }

    async fn verify_token(
        &self,
        request: Request<VerifyTokenRequest>,
    ) -> Result<Response<VerifyTokenResponse>, Status> {
        let req = request.into_inner();
        let valid = self.group_service.verify_token(req.name, req.token).await?;

        Ok(Response::new(VerifyTokenResponse { valid }))
    }

    async fn get_messages(
        &self,
        request: Request<GetMessagesRequest>,
    ) -> Result<Response<MessagesResponse>, Status> {
        let req = request.into_inner();

        let messages = self
            .message_service
            .get_messages(req.channels.clone(), req.offset, req.limit)
            .await?
            .into_iter()
            .map(|msg| msg.into_message_response())
            .collect::<Vec<MessageResponse>>();

        let count = self
            .message_service
            .get_messages_count(req.channels)
            .await?;

        Ok(Response::new(MessagesResponse { messages, count }))
    }

    async fn add_message(
        &self,
        request: Request<AddMessageRequest>,
    ) -> Result<Response<MessageResponse>, Status> {
        let req = request.into_inner();

        let message = self
            .message_service
            .add_message(req.channel, req.subject, req.message)
            .await?;

        Ok(Response::new(message.into_message_response()))
    }

    async fn clear_messages(
        &self,
        request: Request<ClearMessagesRequest>,
    ) -> Result<Response<NotificationResponse>, Status> {
        let req = request.into_inner();

        let deleted_messages = self.message_service.clear_messages(req.date).await?;

        let message = format!("Successfully deleted {} messages", deleted_messages.len());
        Ok(Response::new(NotificationResponse { message }))
    }
}

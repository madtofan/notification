use tonic::{Request, Response, Status};

use madtofan_microservice_common::notification::{
    notification_server::Notification, subscribers_response::Subscriber, AddGroupRequest,
    AddSubscriberRequest, GetSubscribersRequest, NotificationResponse, RemoveGroupRequest,
    RemoveSubscriberRequest, SubscribersResponse,
};

use crate::service::{group::DynGroupServiceTrait, subscriber::DynSubscriberServiceTrait};

pub struct RequestHandler {
    subscriber_service: DynSubscriberServiceTrait,
    group_service: DynGroupServiceTrait,
}

impl RequestHandler {
    pub fn new(
        subscriber_service: DynSubscriberServiceTrait,
        group_service: DynGroupServiceTrait,
    ) -> Self {
        Self {
            subscriber_service,
            group_service,
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
            .remove_subscriber(req.user_id)
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

        self.group_service.add_group(req.name).await?;

        Ok(Response::new(NotificationResponse {
            message: String::from("Successfully add group!"),
        }))
    }

    async fn remove_group(
        &self,
        request: Request<RemoveGroupRequest>,
    ) -> Result<Response<NotificationResponse>, Status> {
        let req = request.into_inner();

        self.group_service.remove_group(req.name).await?;

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
}

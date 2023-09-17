use clap::Parser;
use dotenv::dotenv;
use madtofan_microservice_common::notification::notification_server::NotificationServer;
use madtofan_microservice_common::repository::connection_pool::ServiceConnectionManager;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::{error, info};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::config::AppConfig;
use crate::handler::notification::RequestHandler;
use crate::repository::group::{DynGroupRepositoryTrait, GroupRepository};
use crate::repository::message::{DynMessageRepositoryTrait, MessageRepository};
use crate::repository::subscriber::{DynSubscriberRepositoryTrait, SubscriberRepository};
use crate::seed::SeedService;
use crate::service::group::{DynGroupServiceTrait, GroupService};
use crate::service::message::{DynMessageServiceTrait, MessageService};
use crate::service::subscriber::{DynSubscriberServiceTrait, SubscriberService};

mod config;
mod handler;
mod repository;
mod seed;
mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let config = Arc::new(AppConfig::parse());

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Environment loaded and configuration parsed, initializing Postgres connection...");
    let pg_pool = ServiceConnectionManager::new_pool(&config.database_url)
        .await
        .expect("could not initialize the database connection pool");

    if config.run_migrations {
        info!("migrations enabled, running...");
        sqlx::migrate!()
            .run(&pg_pool)
            .await
            .unwrap_or_else(|err| error!("There was an error during migration: {:?}", err));
    }
    info!("Database configured! initializing repositories...");

    let app_host = &config.service_url;
    let app_port = &config.service_port;
    let app_url = format!("{}:{}", app_host, app_port).parse().unwrap();
    let subscriber_repository =
        Arc::new(SubscriberRepository::new(pg_pool.clone())) as DynSubscriberRepositoryTrait;
    let group_repository =
        Arc::new(GroupRepository::new(pg_pool.clone())) as DynGroupRepositoryTrait;
    let message_repository = Arc::new(MessageRepository::new(pg_pool)) as DynMessageRepositoryTrait;
    info!("Repositories initialized, Initializing Services");
    let subscriber_service = Arc::new(SubscriberService::new(
        subscriber_repository,
        group_repository.clone(),
    )) as DynSubscriberServiceTrait;
    let group_service =
        Arc::new(GroupService::new(group_repository.clone())) as DynGroupServiceTrait;
    let message_service =
        Arc::new(MessageService::new(message_repository)) as DynMessageServiceTrait;
    info!("Services initialized, Initializing Handler");
    let request_handler = RequestHandler::new(subscriber_service, group_service, message_service);

    if config.seed {
        info!("seeding enabled, creating test data...");
        SeedService::new(group_repository)
            .seed()
            .await
            .expect("unexpected error occurred while seeding application data");
    }

    info!("Service ready for request at {:#?}!", app_url);
    Server::builder()
        .add_service(NotificationServer::new(request_handler))
        .serve(app_url)
        .await?;
    Ok(())
}

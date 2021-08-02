use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::email_client::EmailClient;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

static TRACING: once_cell::sync::Lazy<()> = once_cell::sync::Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber("zero2prod-test".into(), "debug".into(), std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub async fn spawn_app() -> TestApp {
    once_cell::sync::Lazy::force(&TRACING);
    let mut configuration = get_configuration().expect("Failed to load configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let address = format!("http://{}", listener.local_addr().unwrap().to_string());
    let db_pool = configure_db(&configuration.database).await;

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
    );
    let server = run(listener, db_pool.clone(), email_client).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp { address, db_pool }
}

async fn configure_db(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to db");

    connection
        .execute(&*format!(r#"CREATE DATABASE "{}""#, config.database_name))
        .await
        .expect("Failed to create db");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database");

    connection_pool
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

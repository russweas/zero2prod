use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use std::convert::TryFrom;
use uuid::Uuid;
#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(from: FormData) -> Result<NewSubscriber, Self::Error> {
        let name = SubscriberName::parse(from.name)?;
        let email = SubscriberEmail::parse(from.email)?;
        Ok(NewSubscriber { email, name })
    }
}

#[tracing::instrument(
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let new_subscriber = match NewSubscriber::try_from(form.into_inner()) {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    match insert_subscriber(&pool, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_e) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(skip(new_sub, pool))]
pub async fn insert_subscriber(pool: &PgPool, new_sub: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_sub.email.as_ref(),
        new_sub.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

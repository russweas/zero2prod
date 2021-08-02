use super::{SubscriberEmail, SubscriberName};
#[derive(Debug, Clone)]
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

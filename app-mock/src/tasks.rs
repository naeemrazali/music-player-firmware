pub mod player_task;
pub mod ui_task;

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, Publisher, Subscriber},
};

use app_core::event::{Event, EventHandler};

const QUEUE_LENGTH: usize = 64;
const NUM_PUBLISHERS: usize = 2;
const NUM_SUBSCRIBERS: usize = 2;

type ChannelPublisher = Publisher<
    'static,
    CriticalSectionRawMutex,
    Event,
    QUEUE_LENGTH,
    NUM_SUBSCRIBERS,
    NUM_PUBLISHERS,
>;

type ChannelSubscriber = Subscriber<
    'static,
    CriticalSectionRawMutex,
    Event,
    QUEUE_LENGTH,
    NUM_SUBSCRIBERS,
    NUM_PUBLISHERS,
>;

pub type EventChannel =
    PubSubChannel<CriticalSectionRawMutex, Event, QUEUE_LENGTH, NUM_SUBSCRIBERS, NUM_PUBLISHERS>;

pub struct Task<T: EventHandler> {
    publisher: ChannelPublisher,
    subscriber: ChannelSubscriber,
    actor: T,
}

impl<T: EventHandler> Task<T> {
    fn new(actor: T, event_channel: &'static EventChannel) -> Self {
        Self {
            actor,
            publisher: event_channel.publisher().unwrap(),
            subscriber: event_channel.subscriber().unwrap(),
        }
    }

    async fn service_event(&mut self, event: &Event) {
        for event in self.actor.handle_event(event) {
            self.publisher.publish(event).await;
        }
    }

    pub fn actor(&self) -> &T {
        &self.actor
    }
}

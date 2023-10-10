use actix::prelude::*;
use std::collections::HashMap;

use crate::turtle_scheme::TurtleEvents;

use super::{FilterItem, Note, Notification, TurtleClosed, TurtleConnected, Warning};

macro_rules! impl_register_notification_listener {
    ($message_type:ty, $listener_type:path, $($notif_type:ty),+) => {
        impl<F: FnMut($($notif_type),+) + 'static> Handler<$message_type> for NotificationRouter {
            type Result = usize;

            fn handle(&mut self, msg: $message_type, _: &mut Self::Context) -> Self::Result {
                self.register($listener_type(Box::new(msg.0)))
            }
        }
    };
}

enum ListenerType {
    Any(Option<Vec<FilterItem>>, Box<dyn FnMut(Notification)>),
    TurtleConnected(Box<dyn FnMut(String)>),
    TurtleClosed(Box<dyn FnMut(String)>),
    TurtleEvent(Box<dyn FnMut(String, TurtleEvents)>),
}

impl ListenerType {
    pub fn call(&mut self, notification: Notification) {
        match (self, notification) {
            (ListenerType::Any(None, l), n) => l(n),
            (ListenerType::Any(Some(f), l), n) if f.iter().any(|f| *f == n.get_filter()) => l(n),
            (ListenerType::TurtleConnected(l), Notification::Note(Note::TurtleConnected(n))) => {
                l(n)
            }
            (ListenerType::TurtleClosed(l), Notification::Warning(Warning::TurtleClosed(n))) => {
                l(n)
            }
            (ListenerType::TurtleEvent(l), Notification::Note(Note::TurtleEvent(name, e))) => {
                l(name, e)
            }
            _ => {}
        }
    }
}

pub struct NotificationRouter {
    listeners: HashMap<usize, ListenerType>,
    next_id: usize,
}

impl NotificationRouter {
    pub fn new() -> Self {
        NotificationRouter {
            listeners: HashMap::new(),
            next_id: 0,
        }
    }

    fn register(&mut self, listener: ListenerType) -> usize {
        let id = self.next_id;

        self.listeners.insert(id, listener);

        id
    }
}

impl Actor for NotificationRouter {
    type Context = Context<NotificationRouter>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Notify(pub Notification);

impl Handler<Notify> for NotificationRouter {
    type Result = ();

    fn handle(&mut self, msg: Notify, ctx: &mut Self::Context) -> Self::Result {
        self.listeners
            .values_mut()
            .for_each(|l| l.call(msg.0.clone()));
    }
}

pub struct RegisterNotificationListener<F> {
    pub listener: F,
    pub filter: Option<Vec<FilterItem>>,
}

impl<F> Message for RegisterNotificationListener<F> {
    type Result = usize;
}

impl<F: FnMut(Notification) + 'static> Handler<RegisterNotificationListener<F>>
    for NotificationRouter
{
    type Result = usize;

    fn handle(
        &mut self,
        msg: RegisterNotificationListener<F>,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let RegisterNotificationListener { listener, filter } = msg;

        self.register(ListenerType::Any(filter, Box::new(listener)))
    }
}

pub struct RegisterConnectedListener<F>(pub F);

impl<F> Message for RegisterConnectedListener<F> {
    type Result = usize;
}

impl_register_notification_listener!(
    RegisterConnectedListener<F>,
    ListenerType::TurtleConnected,
    String
);

pub struct RegisterClosedListener<F>(pub F);

impl<F> Message for RegisterClosedListener<F> {
    type Result = usize;
}

impl_register_notification_listener!(
    RegisterClosedListener<F>,
    ListenerType::TurtleClosed,
    String
);

pub struct RegisterTurtleEventListener<F>(pub F);

impl<F> Message for RegisterTurtleEventListener<F> {
    type Result = usize;
}

impl_register_notification_listener!(
    RegisterTurtleEventListener<F>,
    ListenerType::TurtleEvent,
    String,
    TurtleEvents
);

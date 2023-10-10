use actix::Message;

pub enum TurtleNotificationType {
    Warning,
}

pub enum TurtleWarningType {
    ConnectionClosed,
}

impl Into<String> for TurtleWarningType {
    fn into(self) -> String {
        match self {
            TurtleWarningType::ConnectionClosed => ConnectionClosed::NAME.to_string(),
        }
    }
}

impl TryFrom<String> for TurtleWarningType {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            ConnectionClosed::NAME => Ok(TurtleWarningType::ConnectionClosed),
            _ => Err(()),
        }
    }
}

pub struct TurtleNotificationFilter {
    pub notification_type: Option<TurtleNotificationType>,
    pub notification_name: Option<String>,
}

pub trait TurtleNotificationData: TurtleNotification {
    const NAME: &'static str;

    const TYPE: TurtleNotificationType;
}
pub trait TurtleNotification: Message<Result = ()> {
    type Inner;
    fn into_inner(self) -> Self::Inner;
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct ConnectionClosed(String);

impl ConnectionClosed {
    pub fn new(name: String) -> Self {
        ConnectionClosed(name)
    }

    pub fn name(&self) -> &str {
        &self.0
    }

    pub fn into_name(self) -> String {
        self.0
    }
}

impl TurtleNotification for ConnectionClosed {
    type Inner = ();
    fn into_inner(self) {}
}
impl TurtleNotificationData for ConnectionClosed {
    const NAME: &'static str = "CONNECTION_CLOSED";

    const TYPE: TurtleNotificationType = TurtleNotificationType::Warning;
}

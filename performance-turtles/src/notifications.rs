mod filter;
mod router;

pub use filter::FilterItem;

// pub const TURTLE_CONNECTED: u32 = 1;
// pub const TURTLE_CLOSED: u64 = 1 << 32;

#[derive(Debug, Clone)]
pub enum Notification {
    Note(Note),
    Warning(Warning),
}

impl Notification {
    pub fn get_filter(&self) -> FilterItem {
        match self {
            Notification::Note(note) => note.get_filter(),
            Notification::Warning(warning) => warning.get_filter(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Note {
    TurtleConnected(TurtleConnected),
}

impl Note {
    pub fn get_filter(&self) -> FilterItem {
        match self {
            Note::TurtleConnected(_) => FilterItem::TurtleConnected,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Warning {
    TurtleClosed(TurtleClosed),
}

impl Warning {
    pub fn get_filter(&self) -> FilterItem {
        match self {
            Warning::TurtleClosed(_) => FilterItem::TurtleClosed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TurtleConnected(pub String);

#[derive(Debug, Clone)]
pub struct TurtleClosed(pub String);

use ratatui::{
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::{List, Widget},
};
use turtle_types::{client_views::TurtleReport, turtle_scheme::TurtleStatus};

#[derive(Debug, Clone, Copy)]
pub struct TurtleListItem<'a> {
    name: &'a str,
    status: TurtleStatus,
}

impl<'a> From<&'a TurtleReport> for TurtleListItem<'a> {
    fn from(value: &'a TurtleReport) -> Self {
        TurtleListItem {
            name: &value.name,
            status: value.status,
        }
    }
}

impl<'a> From<TurtleListItem<'a>> for Line<'a> {
    fn from(val: TurtleListItem<'a>) -> Self {
        Line::from(val.to_string())
    }
}

impl std::fmt::Display for TurtleListItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (status, color) = if self.status == TurtleStatus::Connected {
            ("Connected", Color::Green)
        } else {
            ("Disconnected", Color::Red)
        };

        write!(f, "{}\t{}", self.name, status.fg(color))
    }
}

pub struct TurtleList<T>(T);

impl<'a, T> TurtleList<T>
where
    T: IntoIterator,
    T::Item: Into<TurtleListItem<'a>>,
{
    pub fn new(turtles: T) -> Self {
        TurtleList(turtles)
    }
}

impl<'a, T> Widget for TurtleList<T>
where
    T: IntoIterator,
    T::Item: Into<TurtleListItem<'a>>,
{
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let list = List::new(Text::from_iter(self.0.into_iter().map(Into::into)));
        list.render(area, buf);
    }
}

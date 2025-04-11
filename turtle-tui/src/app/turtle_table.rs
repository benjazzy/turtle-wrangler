use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    widgets::{Cell, Row, Table, TableState},
    Frame,
};
use turtle_types::{
    client_views::TurtleReport,
    turtle_scheme::{TurtleStatus, TurtleType},
};

#[derive(Debug, Default)]
pub struct TurtleTable {
    table_state: TableState,
    turtles: Vec<TurtleReport>,
}

impl TurtleTable {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn update_turtles(&mut self, turtles: Vec<TurtleReport>) {
        self.turtles = turtles;
    }

    pub fn select_next(&mut self) {
        self.table_state.select_next();
    }

    pub fn select_previous(&mut self) {
        self.table_state.select_previous();
    }

    pub fn selected_turtle(&self) -> &TurtleReport {
        &self.turtles[self.table_state.selected().unwrap_or_default()]
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let widths = [
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
        ];
        let headers = ["Name", "Status", "Type", "Fuel"];

        let table = Table::new(self.turtles.iter().map(report_to_row), widths)
            .column_spacing(1)
            .header(Row::new(headers))
            .row_highlight_style(Style::new().reversed())
            .highlight_symbol(">>");

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }
}

fn report_to_row(turtle: &TurtleReport) -> Row {
    let name = Cell::new(turtle.name.as_ref());
    let status = match turtle.status {
        TurtleStatus::Connected => Cell::new("Connected").style(Color::Green),
        TurtleStatus::Disconnected => Cell::new("Disconnected").style(Color::Red),
    };
    let turtle_type = match turtle.turtle_type {
        TurtleType::Normal => Cell::new("Normal").style(Color::Gray),
        TurtleType::Advanced => Cell::new("Advanced").style(Color::Yellow),
    };
    let fuel = Cell::new(turtle.fuel.to_string());

    let cells = [name, status, turtle_type, fuel];

    Row::new(cells)
}

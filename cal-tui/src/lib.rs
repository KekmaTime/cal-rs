use anyhow::Result;
use cal_core::Calendar;
use cal_events::EventManager;
use chrono::{Date, Datelike, Local};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
};
use std::{
    io,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub enum ViewMode {
    Main,
    EventManager(usize), // selected option
    AddEventWindow(EventInput),
    DeleteEventWindow,
    ModifyEventWindow,
}

pub struct App {
    calendar: Calendar,
    event_manager: EventManager,
    view_mode: ViewMode,
}

impl App {
    pub fn new() -> Self {
        Self {
            calendar: Calendar::new(),
            event_manager: EventManager::new(),
            view_mode: ViewMode::Main,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') if matches!(app.view_mode, ViewMode::Main) => break Ok(()),

                    e if matches!(app.view_mode, ViewMode::Main) => {
                        handle_main_keyevents(e, &mut app)
                    }

                    e if matches!(app.view_mode, ViewMode::EventManager(_)) => {
                        handle_eventmanager_keyevent(e, &mut app)
                    }

                    e if matches!(app.view_mode, ViewMode::AddEventWindow(_)) => {
                        handle_add_event_keyevents(e, &mut app)
                    }

                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn create_mini_calendar(app: &App) -> Table {
    let weekdays = ["S", "M", "T", "W", "T", "F", "S"];
    let header_cells = weekdays
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Gray)));
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1);

    let grid = app.calendar.get_month_grid();
    let rows: Vec<Row> = grid
        .iter()
        .map(|week| {
            let cells = week.iter().map(|day| match day {
                Some(d) => {
                    let now = Local::now();
                    let is_current_day = d == &now.day()
                        && app.calendar.current_date.month() == now.month()
                        && app.calendar.current_date.year() == now.year();

                    let style = if is_current_day {
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    Cell::from(format!("{:2}", d)).style(style)
                }
                None => Cell::from("  "),
            });
            Row::new(cells).height(1)
        })
        .collect();

    let widths = [
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
    ];

    Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "{}  {}",
            app.calendar.current_date.format("%B"),
            app.calendar.current_date.format("%Y")
        )))
        .column_spacing(1)
}

fn ui(f: &mut Frame, app: &mut App) {
    let area = f.area();

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(area);

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(20),
            Constraint::Length(10),
        ])
        .split(main_chunks[1]);

    let header_text = format!("← {}   Today   → {} ", "Previous", "Next");
    let header = Paragraph::new(header_text)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, content_chunks[0]);

    let weekdays = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
    let header_cells = weekdays
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Gray)));
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(2);

    let grid = app.calendar.get_month_grid();
    let rows: Vec<Row> = grid
        .iter()
        .map(|week| {
            let cells = week.iter().map(|day| match day {
                Some(d) => {
                    let now = Local::now();
                    let is_current_day = d == &now.day()
                        && app.calendar.current_date.month() == now.month()
                        && app.calendar.current_date.year() == now.year();

                    let is_selected = d == &app.calendar.selected_date.day()
                        && app.calendar.current_date.month() == app.calendar.selected_date.month()
                        && app.calendar.current_date.year() == app.calendar.selected_date.year();

                    let style = match (is_current_day, is_selected) {
                        (true, true) => Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD | Modifier::REVERSED),
                        (true, false) => Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                        (false, true) => Style::default().add_modifier(Modifier::REVERSED),
                        (false, false) => Style::default(),
                    };

                    Cell::from(format!(" {} ", d)).style(style)
                }
                None => Cell::from("   "),
            });
            Row::new(cells).height(3)
        })
        .collect();

    let widths = [
        Constraint::Percentage(14),
        Constraint::Percentage(14),
        Constraint::Percentage(14),
        Constraint::Percentage(14),
        Constraint::Percentage(14),
        Constraint::Percentage(14),
        Constraint::Percentage(14),
    ];

    let calendar_table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "{}  {}",
            app.calendar.current_date.format("%B"),
            app.calendar.current_date.format("%Y")
        )))
        .column_spacing(1);

    f.render_widget(calendar_table, content_chunks[1]);

    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(12), Constraint::Min(0)])
        .split(main_chunks[0]);

    let mini_calendar = create_mini_calendar(app);
    f.render_widget(mini_calendar, sidebar_chunks[0]);

    let events = app
        .event_manager
        .list_events_for_day(app.calendar.current_date);
    let events_text = if events.is_empty() {
        "No events scheduled".to_string()
    } else {
        events
            .iter()
            .map(|e| format!("• {} ({})", e.title, e.start_time.format("%H:%M")))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let events_widget = Paragraph::new(events_text)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Events for {}",
            app.calendar.selected_date.format("%B %d, %Y")
        )))
        .style(Style::default())
        .alignment(Alignment::Left);

    f.render_widget(events_widget, content_chunks[2]);

    #[allow(clippy::single_match)]
    match &mut app.view_mode {
        ViewMode::EventManager(index) => {
            let area = center(
                f.area(),
                Constraint::Percentage(30),
                Constraint::Percentage(20),
            );

            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Event Manager ");

            let options = ["Add an event", "Delete an event", "Modify an event"];

            let items: Vec<ListItem> = options
                .iter()
                .enumerate()
                .map(|(i, opt)| {
                    let mut item = ListItem::new(*opt);

                    if i == *index {
                        item = item.bg(Color::Gray).fg(Color::Black);
                    }

                    item
                })
                .collect();

            let list = List::new(items)
                .block(block)
                .highlight_symbol(">")
                .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

            f.render_widget(Clear, area);
            f.render_widget(list, area);
        }

        ViewMode::AddEventWindow(input) => {
            let area = center(
                f.area(),
                Constraint::Percentage(40),
                Constraint::Percentage(40),
            );

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(6), Constraint::Length(1)])
                .split(area);

            let widths = [Constraint::Percentage(40), Constraint::Length(60)];

            // (Label, Value, Field Index)
            let fields = [
                ("Title", &input.title, 0),
                ("Description", &input.description, 1),
                ("Start Time (HH:MM)", &input.start_time, 2),
                ("End Time (HH:MM)", &input.end_time, 3),
            ];

            let rows: Vec<Row> = fields
                .iter()
                .map(|(label, value, field_idx)| {
                    let label = Cell::from(format!("{label}: "));
                    let input_field_style = if input.focused_field == *field_idx {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    let input_field = Cell::from((**value).clone()).style(input_field_style);

                    Row::new([label, input_field])
                })
                .collect();

            let block = Block::default().borders(Borders::ALL).title(" Add Event ");
            let table = Table::new(rows, widths).block(block).column_spacing(1);

            let info_text = Paragraph::new(Line::from(
                " Press Esc to go back, Enter to confirm "
                    .black()
                    .on_white(),
            ));

            f.render_widget(Clear, area);
            f.render_widget(table, layout[0]);
            f.render_widget(info_text, layout[1]);
        }

        _ => {}
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(layout::Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical])
        .flex(layout::Flex::Center)
        .areas(area);
    area
}

#[derive(Debug, Clone, Default)]
pub struct EventInput {
    pub title: String,
    pub description: String,
    pub start_time: String,
    pub end_time: String,
    pub focused_field: usize, // Tracks which field is currently focused
}

fn handle_eventmanager_keyevent(key: event::KeyCode, app: &mut App) {
    let ViewMode::EventManager(i) = &mut app.view_mode else {
        unreachable!()
    };

    match key {
        KeyCode::Esc => app.view_mode = ViewMode::Main,
        KeyCode::Up if *i != 0 => *i -= 1,
        KeyCode::Down => *i = (*i + 1) % 3,

        KeyCode::Enter => {
            app.view_mode = [
                ViewMode::AddEventWindow(EventInput::default()),
                ViewMode::DeleteEventWindow,
                ViewMode::ModifyEventWindow,
            ][*i]
                .clone();
        }

        _ => {}
    }
}

fn handle_main_keyevents(key: event::KeyCode, app: &mut App) {
    match key {
        KeyCode::Char('e') => app.view_mode = ViewMode::EventManager(0),
        KeyCode::Left => {
            if !app.calendar.move_selection("left") {
                app.calendar.prev_month();
                let grid = app.calendar.get_month_grid();
                for week in grid.iter().rev() {
                    if let Some(Some(last_day)) = week.iter().rev().find(|d| d.is_some()) {
                        app.calendar.selected_date =
                            app.calendar.current_date.with_day(*last_day).unwrap();
                        break;
                    }
                }
            }
        }

        KeyCode::Right => {
            if !app.calendar.move_selection("right") {
                app.calendar.next_month();
                if let Some(Some(first_day)) = app
                    .calendar
                    .get_month_grid()
                    .iter()
                    .flat_map(|week| week.iter())
                    .find(|d| d.is_some())
                {
                    app.calendar.selected_date =
                        app.calendar.current_date.with_day(*first_day).unwrap();
                }
            }
        }

        KeyCode::Up => {
            app.calendar.move_selection("up");
        }

        KeyCode::Down => {
            app.calendar.move_selection("down");
        }

        _ => {}
    }
}

fn handle_add_event_keyevents(key: event::KeyCode, app: &mut App) {
    let ViewMode::AddEventWindow(input) = &mut app.view_mode else {
        unreachable!()
    };

    if let ViewMode::AddEventWindow(input) = &mut app.view_mode {
        match key {
            KeyCode::Down => {
                input.focused_field = (input.focused_field + 1) % 4;
            }

            KeyCode::Up => {
                input.focused_field = if input.focused_field == 0 {
                    3
                } else {
                    input.focused_field - 1
                };
            }

            // Pressing enter will crash the app due to panic
            KeyCode::Enter => {
                println!("Event added!");
                if let Ok(event) = cal_events::Event::new(
                    input.title.clone(),
                    if input.description.is_empty() {
                        None
                    } else {
                        Some(input.description.clone())
                    },
                    // it panic shere
                    chrono::DateTime::parse_from_str("2021-10-10 10:00", "%Y-%m-%d %H:%M")
                        .unwrap()
                        .with_timezone(&Local),
                    chrono::DateTime::parse_from_str("2021-10-10 11:00", "%Y-%m-%d %H:%M")
                        .unwrap()
                        .with_timezone(&Local),
                    // input.start_time.clone(),
                    // input.end_time.clone(),
                ) {
                    let _ = app.event_manager.add_event(event).unwrap();
                }

                // app.view_mode = ViewMode::Main;
            }

            KeyCode::Esc => app.view_mode = ViewMode::EventManager(0),

            KeyCode::Backspace => {
                let field = match input.focused_field {
                    0 => &mut input.title,
                    1 => &mut input.description,
                    2 => &mut input.start_time,
                    3 => &mut input.end_time,
                    _ => unreachable!(),
                };

                field.pop();
            }

            KeyCode::Char(ch) => {
                let field = match input.focused_field {
                    0 => &mut input.title,
                    1 => &mut input.description,
                    2 => &mut input.start_time,
                    3 => &mut input.end_time,
                    _ => unreachable!(),
                };

                field.push(ch);
            }

            _ => {}
        }
    }
}

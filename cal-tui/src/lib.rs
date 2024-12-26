use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Table, Row, Cell},
    layout::{Layout, Direction, Constraint},
    style::{Style, Modifier, Color},
};
use std::{io, time::{Duration, Instant}};
use cal_core::Calendar;
use cal_events::EventManager;
use chrono::{Datelike, Local};

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Month,
    Week,
    Day,
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
            view_mode: ViewMode::Month,
        }
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
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('m') => app.view_mode = ViewMode::Month,
                    KeyCode::Char('w') => app.view_mode = ViewMode::Week,
                    KeyCode::Char('d') => app.view_mode = ViewMode::Day,
                    KeyCode::Left => {
                        if !app.calendar.move_selection("left") {
                            app.calendar.prev_month();
                            let grid = app.calendar.get_month_grid();
                            for week in grid.iter().rev() {
                                if let Some(Some(last_day)) = week.iter().rev().find(|d| d.is_some()) {
                                    app.calendar.selected_date = app.calendar.current_date.with_day(*last_day).unwrap();
                                    break;
                                }
                            }
                        }
                    },
                    KeyCode::Right => {
                        if !app.calendar.move_selection("right") {
                            app.calendar.next_month();
                            if let Some(Some(first_day)) = app.calendar.get_month_grid()
                                .iter()
                                .flat_map(|week| week.iter())
                                .find(|d| d.is_some()) {
                                app.calendar.selected_date = app.calendar.current_date.with_day(*first_day).unwrap();
                            }
                        }
                    },
                    KeyCode::Up => { app.calendar.move_selection("up"); },
                    KeyCode::Down => { app.calendar.move_selection("down"); },
                    KeyCode::Char('e') => {
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
    let header_cells = weekdays.iter()
        .map(|h| Cell::from(*h)
            .style(Style::default().fg(Color::Gray)));
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1);

    let grid = app.calendar.get_month_grid();
    let rows: Vec<Row> = grid.iter().map(|week| {
        let cells = week.iter().map(|day| {
            match day {
                Some(d) => {
                    let now = Local::now();
                    let is_current_day = d == &now.day() && 
                        app.calendar.current_date.month() == now.month() &&
                        app.calendar.current_date.year() == now.year();
                    
                    let style = if is_current_day {
                        Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    
                    Cell::from(format!("{:2}", d)).style(style)
                },
                None => Cell::from("  "),
            }
        });
        Row::new(cells).height(1)
    }).collect();

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
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("{}  {}", 
                app.calendar.current_date.format("%B"),
                app.calendar.current_date.format("%Y"))))
        .column_spacing(1)
}

fn create_clock() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

fn ui(f: &mut Frame, app: &App) {
    let area = f.area();
    
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),  
            Constraint::Percentage(80),
        ])
        .split(area);

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   
            Constraint::Min(20),     
            Constraint::Length(10),  
        ])
        .split(main_chunks[1]);

    let header_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(content_chunks[0]);

    let nav_text = format!(
        "← {}   Today   → {} ", 
        "Previous", 
        "Next"
    );
    let nav_header = Paragraph::new(nav_text)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    
    let clock_text = create_clock();
    let clock = Paragraph::new(clock_text)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(nav_header, header_layout[0]);
    f.render_widget(clock, header_layout[1]);

    let weekdays = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
    let header_cells = weekdays.iter()
        .map(|h| Cell::from(*h)
            .style(Style::default().fg(Color::Gray)));
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(2);

    let grid = app.calendar.get_month_grid();
    let rows: Vec<Row> = grid.iter().map(|week| {
        let cells = week.iter().map(|day| {
            match day {
                Some(d) => {
                    let now = Local::now();
                    let is_current_day = d == &now.day() && 
                        app.calendar.current_date.month() == now.month() &&
                        app.calendar.current_date.year() == now.year();
                    let is_selected = d == &app.calendar.selected_date.day() &&
                        app.calendar.current_date.month() == app.calendar.selected_date.month() &&
                        app.calendar.current_date.year() == app.calendar.selected_date.year();
                    
                    let style = match (is_current_day, is_selected) {
                        (true, true) => Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD | Modifier::REVERSED),
                        (true, false) => Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                        (false, true) => Style::default()
                            .add_modifier(Modifier::REVERSED),
                        (false, false) => Style::default(),
                    };
                    
                    Cell::from(format!(" {} ", d)).style(style)
                },
                None => Cell::from("   "),
            }
        });
        Row::new(cells).height(3) 
    }).collect();

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
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("{}  {}", 
                app.calendar.current_date.format("%B"),
                app.calendar.current_date.format("%Y"))))
        .column_spacing(1);

    f.render_widget(calendar_table, content_chunks[1]);

    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12),  
            Constraint::Min(0),      
        ])
        .split(main_chunks[0]);

    let mini_calendar = create_mini_calendar(app);
    f.render_widget(mini_calendar, sidebar_chunks[0]);

    let events = app.event_manager.list_events_for_day(app.calendar.current_date);
    let events_text = if events.is_empty() {
        "No events scheduled".to_string()
    } else {
        events.iter()
            .map(|e| format!("• {} ({})", e.title, e.start_time.format("%H:%M")))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let events_widget = Paragraph::new(events_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("Events for {}", 
                app.calendar.selected_date.format("%B %d, %Y"))))
        .style(Style::default())
        .alignment(Alignment::Left);

    f.render_widget(events_widget, content_chunks[2]);
}
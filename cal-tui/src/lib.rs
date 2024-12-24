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

pub struct App {
    calendar: Calendar,
    event_manager: EventManager,
}

impl App {
    pub fn new() -> Self {
        Self {
            calendar: Calendar::new(),
            event_manager: EventManager::new(),
        }
    }
}

pub fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

    // Cleanup
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
                    KeyCode::Left => app.calendar.prev_month(),
                    KeyCode::Right => app.calendar.next_month(),
                    KeyCode::Char('e') => {
                        // TODO: Add event mode
                    }
                    KeyCode::Char('d') => {
                        // TODO: Delete event mode
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

fn ui(f: &mut Frame, app: &App) {
    let area = f.area();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Title
            Constraint::Min(20),     // Calendar
            Constraint::Length(10),  // Events list
        ])
        .split(area);

    // Title
    let title = Paragraph::new(format!("Cal-rs - {}", app.calendar.current_date.format("%B %Y")))
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Calendar Grid
    let weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let header_cells = weekdays.iter().map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).style(Style::default().add_modifier(Modifier::BOLD));

    let grid = app.calendar.get_month_grid();
    let rows: Vec<Row> = grid.iter().map(|week| {
        let cells = week.iter().map(|day| {
            match day {
                Some(d) => Cell::from(d.to_string()),
                None => Cell::from(" "),
            }
        });
        Row::new(cells)
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
        .block(Block::default().borders(Borders::ALL).title("Calendar"));

    f.render_widget(calendar_table, chunks[1]);

    // Events section
    let events = app.event_manager.list_events_for_day(app.calendar.current_date);
    let events_text = if events.is_empty() {
        "No events for this day".to_string()
    } else {
        events.iter()
            .map(|e| format!("• {} ({})", e.title, e.start_time.format("%H:%M")))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let events_widget = Paragraph::new(events_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Events"))
        .style(Style::default())
        .alignment(Alignment::Left);

    f.render_widget(events_widget, chunks[2]);
}
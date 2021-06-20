use tui::widgets::ListState;
pub struct StatefulList<T> {
  pub state: ListState,
  pub items: Vec<T>,
}

impl<T> StatefulList<T> {
  pub fn with_items(items: Vec<T>) -> StatefulList<T> {
      StatefulList {
          state: ListState::default(),
          items,
      }
  }

  pub fn next(&mut self) {
      let i = match self.state.selected() {
          Some(i) => {
              if i >= self.items.len() - 1 {
                  0
              } else {
                  i + 1
              }
          }
          None => 0,
      };
      self.state.select(Some(i));
  }

  pub fn previous(&mut self) {
      let i = match self.state.selected() {
          Some(i) => {
              if i == 0 {
                  self.items.len() - 1
              } else {
                  i - 1
              }
          }
          None => 0,
      };
      self.state.select(Some(i));
  }
}

const TASKS: [&str; 24] = [
  "Item1", "Item2", "Item3", "Item4", "Item5", "Item6", "Item7", "Item8", "Item9", "Item10",
  "Item11", "Item12", "Item13", "Item14", "Item15", "Item16", "Item17", "Item18", "Item19",
  "Item20", "Item21", "Item22", "Item23", "Item24",
];
pub struct App<'a> {
  pub title: &'a str,
  pub should_quit: bool,
  pub tasks: StatefulList<&'a str>,
}

impl<'a> App<'a> {
  pub fn new(title: &'a str) -> App<'a> {
      App {
          title,
          should_quit: false,
          tasks: StatefulList::with_items(TASKS.to_vec()),
      }
  }

  pub fn on_up(&mut self) {
      self.tasks.previous();
  }

  pub fn on_down(&mut self) {
      self.tasks.next();
  }

  pub fn on_key(&mut self, c: char) {
      match c {
          'q' => {
              self.should_quit = true;
          }
          _ => {}
      }
  }

  pub fn on_tick(&mut self) {
      // Update progress
  }
}

use tui::{
  backend::Backend,
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Span, Spans},
  widgets::{
      Block, Borders, List, ListItem, Paragraph, Wrap,
  },
  Frame,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
  let chunks = Layout::default()
  .constraints(
      [
          Constraint::Min(8),
          Constraint::Length(4),
      ]
      .as_ref(),
  )
  .split(f.size());
  draw_term_output(f, app, chunks[0]);
  draw_term_input(f, chunks[1]);
}

fn draw_term_output<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
  B: Backend,
{
  let constraints = vec![Constraint::Percentage(100)];
  let chunks = Layout::default()
      .constraints(constraints)
      .direction(Direction::Horizontal)
      .split(area);

  let chunks = Layout::default()
      .constraints([Constraint::Percentage(10), Constraint::Percentage(50)].as_ref())
      .direction(Direction::Horizontal)
      .split(chunks[0]);
  // Draw tasks
  let tasks: Vec<ListItem> = app
      .tasks
      .items
      .iter()
      .map(|i| ListItem::new(vec![Spans::from(Span::raw(*i))]))
      .collect();
  let tasks = List::new(tasks)
      .block(Block::default().borders(Borders::ALL).title("History"))
      .highlight_style(Style::default().add_modifier(Modifier::BOLD))
      .highlight_symbol("> ");
  f.render_stateful_widget(tasks, chunks[0], &mut app.tasks.state);

  let cmd_out = "This is a paragraph with several linekjdsh fa;k fjd slkafjlk 
  s;dj falkfjdsl jflksdjfk lsjflsk;daf
  jk lsfjslakdfjk lsafjdsklfjdslk fjdslkfjs
  dl;akfjskdl ajfsdklfjdsklfjslk dfjslakjfsld
  kfjdklsfjlskdjflksdfjskldfjsldkfjlskd;fjdl;sk
  jfdslk;fjlskdjflskjfdlsak;fjlsads. You can chan
  ge style your text the way you want";
  let text = vec![
      Spans::from(cmd_out),
  ];
  let block = Block::default().borders(Borders::ALL).title(Span::styled(
      "Output",
      Style::default()
          .fg(Color::Magenta)
          .add_modifier(Modifier::BOLD),
  ));
  let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
  f.render_widget(paragraph, chunks[1]);
}

fn draw_term_input<B>(f: &mut Frame<B>, area: Rect)
where
  B: Backend,
{
  let text = vec![
      Spans::from("This is a paragraph with several lines. You can change style your text the way you want"),
  ];
  let block = Block::default().borders(Borders::ALL).title(Span::styled(
      "Command",
      Style::default()
          .fg(Color::Magenta)
          .add_modifier(Modifier::BOLD),
  ));
  let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
  f.render_widget(paragraph, area);
}


use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io::stdout,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, Terminal};

enum Event<I> {
    Input(I),
    Tick,
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    // Setup input handling
    let (tx, rx) = mpsc::channel();

    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });

    let mut app = App::new("Termline");

    terminal.clear()?;

    loop {
        terminal.draw(|f| draw(f, &mut app))?;
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char(c) => app.on_key(c),
                KeyCode::Up => app.on_up(),
                KeyCode::Down => app.on_down(),
                _ => {}
            },
            Event::Tick => {
                app.on_tick();
            }
        }
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
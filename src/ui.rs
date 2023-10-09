mod tab_abnormality;
mod tab_reader;
mod tab_topic;
mod tab_writer;

use crate::state::State;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols::DOT,
    widgets::{Block, Borders, Tabs},
    Frame, Terminal,
};
use std::{
    io,
    ops::ControlFlow,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tracing::error;

use self::{
    tab_abnormality::TabAbnormality, tab_reader::TabReader, tab_topic::TabTopic,
    tab_writer::TabWriter,
};

const TAB_TITLES: &[&str] = &["Writers", "Reader", "Topics", "Abnormalities"];

pub(crate) struct Tui {
    tab_writer: TabWriter,
    tab_reader: TabReader,
    tab_topic: TabTopic,
    tab_abnormality: TabAbnormality,
    tick_dur: Duration,
    tab_index: usize,
    state: Arc<Mutex<State>>,
}

impl Tui {
    pub fn new(tick_dur: Duration, state: Arc<Mutex<State>>) -> Self {
        Self {
            tick_dur,
            state,
            tab_index: 0,
            tab_writer: TabWriter::new(),
            tab_topic: TabTopic::new(),
            tab_abnormality: TabAbnormality::new(),
            tab_reader: TabReader::new(),
        }
    }

    pub fn run(mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        self.run_loop(&mut terminal)?;

        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn run_loop<B>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()>
    where
        B: Backend,
    {
        let mut last_tick = Instant::now();

        loop {
            // Wait for key event
            {
                let timeout = self
                    .tick_dur
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                // Process keyboard events
                let ctrl_flow = self.process_events(timeout)?;
                if let ControlFlow::Break(_) = ctrl_flow {
                    break;
                }
            }

            let elapsed_time = last_tick.elapsed();
            if elapsed_time >= self.tick_dur {
                // Draw UI
                terminal.draw(|frame| self.draw_ui(frame, elapsed_time))?;

                // Clean up state
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    fn process_events(&mut self, timeout: Duration) -> io::Result<ControlFlow<()>> {
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                use KeyCode as C;

                let n_tabs = TAB_TITLES.len();

                match key.code {
                    C::Char('q') => return Ok(ControlFlow::Break(())),
                    C::Up => {
                        self.key_up();
                    }
                    C::Down => {
                        self.key_down();
                    }
                    C::Left => {
                        // *self.table_state.offset_mut() =
                        //     self.table_state.offset().saturating_sub(1);
                    }
                    C::Right => {
                        // *self.table_state.offset_mut() =
                        //     self.table_state.offset().saturating_add(1);
                    }
                    C::PageUp => {
                        self.key_page_up();
                    }
                    C::PageDown => {
                        self.key_page_down();
                    }
                    C::Home => {
                        self.key_home();
                    }
                    C::End => {
                        self.key_end();
                    }
                    C::Tab => {
                        // Jump to next tab
                        self.tab_index = (self.tab_index + 1) % n_tabs;
                    }
                    C::BackTab => {
                        // Go to previous tab
                        self.tab_index = (self.tab_index + (n_tabs - 1)) % n_tabs;
                    }
                    _ => {}
                }
            }
        }

        Ok(ControlFlow::Continue(()))
    }

    fn draw_ui<B>(&mut self, frame: &mut Frame<B>, _elapsed_time: Duration)
    where
        B: Backend,
    {
        // Unlock the state
        let Ok(state) = self.state.lock() else {
            // TODO: show error
            error!("State lock is poisoned");
            return;
        };

        // Split the screen vertically into two chunks.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(frame.size());

        // Build the container for tabs
        let tabs_block = Block::default().title("Tabs").borders(Borders::ALL);
        let tabs = Tabs::new(TAB_TITLES.to_vec())
            .block(tabs_block)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow))
            .divider(DOT)
            .select(self.tab_index);
        frame.render_widget(tabs, chunks[0]);

        // Render the tab content according to the current tab index.
        match self.tab_index {
            0 => self.tab_writer.render(&state, frame, chunks[1]),
            1 => self.tab_reader.render(&state, frame, chunks[1]),
            2 => self.tab_topic.render(&state, frame, chunks[1]),
            3 => self.tab_abnormality.render(&state, frame, chunks[1]),
            _ => unreachable!(),
        }
    }

    fn key_up(&mut self) {
        match self.tab_index {
            0 => self.tab_writer.previous_item(),
            1 => self.tab_reader.previous_item(),
            2 => self.tab_topic.previous_item(),
            3 => self.tab_abnormality.previous_item(),
            _ => unreachable!(),
        }
    }

    fn key_down(&mut self) {
        match self.tab_index {
            0 => self.tab_writer.next_item(),
            1 => self.tab_reader.next_item(),
            2 => self.tab_topic.next_item(),
            3 => self.tab_abnormality.next_item(),
            _ => unreachable!(),
        }
    }

    fn key_page_up(&mut self) {
        match self.tab_index {
            0 => self.tab_writer.previous_page(),
            1 => self.tab_reader.previous_page(),
            2 => self.tab_topic.previous_page(),
            3 => self.tab_abnormality.previous_page(),
            _ => unreachable!(),
        }
    }

    fn key_page_down(&mut self) {
        match self.tab_index {
            0 => self.tab_writer.next_page(),
            1 => self.tab_reader.next_page(),
            2 => self.tab_topic.next_page(),
            3 => self.tab_abnormality.next_page(),
            _ => unreachable!(),
        }
    }

    fn key_home(&mut self) {
        match self.tab_index {
            0 => self.tab_writer.first_item(),
            1 => self.tab_reader.first_item(),
            2 => self.tab_topic.first_item(),
            3 => self.tab_abnormality.first_item(),
            _ => unreachable!(),
        }
    }

    fn key_end(&mut self) {
        match self.tab_index {
            0 => self.tab_writer.last_item(),
            1 => self.tab_reader.last_item(),
            2 => self.tab_topic.last_item(),
            3 => self.tab_abnormality.last_item(),
            _ => unreachable!(),
        }
    }
}

use crate::engine::EvaluationObserver;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::io::{stdout, Stdout};

pub struct TuiState {
    pub task_id: String,
    pub provider: String,
    pub model: String,
    pub current_turn: usize,
    pub max_turns: usize,
    pub status: String,
    pub reasoning_logs: Vec<String>,
    pub sandbox_logs: Vec<String>,
}

pub struct TuiDashboard {
    terminal: Option<Terminal<CrosstermBackend<Stdout>>>,
    pub state: TuiState,
}

impl TuiDashboard {
    pub fn new(task_id: String, provider: String, model: String, max_turns: usize) -> Result<Self> {
        let state = TuiState {
            task_id,
            provider,
            model,
            current_turn: 0,
            max_turns,
            status: "RUNNING".to_string(),
            reasoning_logs: Vec::new(),
            sandbox_logs: Vec::new(),
        };

        Ok(Self {
            terminal: None,
            state,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        self.terminal = Some(terminal);
        self.draw()?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if self.terminal.is_some() {
            disable_raw_mode()?;
            execute!(stdout(), LeaveAlternateScreen)?;
            self.terminal = None;
        }
        Ok(())
    }

    pub fn draw(&mut self) -> Result<()> {
        if let Some(ref mut terminal) = self.terminal {
            let state = &self.state;
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(10),
                        Constraint::Length(3),
                    ])
                    .split(f.area());

                let status_style = match state.status.as_str() {
                    "PASSED" => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    "FAILED" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    _ => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                };

                let header_text = Line::from(vec![
                    Span::styled("⚡ Spacetime Harness  |  ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("Task: {}  |  ", state.task_id), Style::default().fg(Color::White)),
                    Span::styled(format!("Provider: {} ({})  |  ", state.provider, state.model), Style::default().fg(Color::Magenta)),
                    Span::styled(format!("Turn: {}/{}  |  ", state.current_turn, state.max_turns), Style::default().fg(Color::Yellow)),
                    Span::styled(&state.status, status_style),
                ]);

                let header = Paragraph::new(header_text)
                    .block(Block::default().borders(Borders::ALL).title(" Benchmark Status "));
                f.render_widget(header, chunks[0]);

                let body_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[1]);

                let reasoning_content = state.reasoning_logs.join("\n\n---\n\n");
                let thoughts_panel = Paragraph::new(reasoning_content)
                    .block(Block::default().borders(Borders::ALL).title(" Agent Reasoning & Thoughts "))
                    .wrap(Wrap { trim: true });
                f.render_widget(thoughts_panel, body_chunks[0]);

                let logs_content = state.sandbox_logs.join("\n");
                let logs_panel = Paragraph::new(logs_content)
                    .block(Block::default().borders(Borders::ALL).title(" Sandbox Container Logs "))
                    .wrap(Wrap { trim: true });
                f.render_widget(logs_panel, body_chunks[1]);

                let footer_text = Line::from(vec![
                    Span::styled("Press 'q' or Ctrl+C to abort evaluation session", Style::default().fg(Color::DarkGray)),
                ]);
                let footer = Paragraph::new(footer_text)
                    .block(Block::default().borders(Borders::ALL).title(" Controls "));
                f.render_widget(footer, chunks[2]);
            })?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn handle_events(&mut self) -> Result<bool> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q')
                    || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

impl Drop for TuiDashboard {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

impl EvaluationObserver for TuiDashboard {
    fn on_turn_start(&mut self, turn: usize) {
        self.state.current_turn = turn;
        let _ = self.draw();
    }

    fn on_reasoning(&mut self, turn: usize, reasoning: &str) {
        self.state.reasoning_logs.push(format!("[Turn {}]\n{}", turn, reasoning));
        let _ = self.draw();
    }

    fn on_command(
        &mut self,
        turn: usize,
        command: &str,
        stdout: &str,
        stderr: &str,
        exit_code: i64,
    ) {
        self.state.sandbox_logs.push(format!(
            "[Turn {}] $ {}\nExit Code: {}\nSTDOUT: {}\nSTDERR: {}",
            turn, command, exit_code, stdout, stderr
        ));
        let _ = self.draw();
    }
}

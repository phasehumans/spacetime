use crate::engine::EvaluationObserver;
use crate::task::BenchmarkTask;
use anyhow::{anyhow, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::ui::{Attributes, Color, RenderConfig, StyleSheet, Styled};
use inquire::Select;
use std::fmt;

pub fn print_banner() {
    let logo = r#"
  _____ _____   _   ___ _____ _____ _____ __  __ _____ 
 /  ___|  _  \ / \ /  _|  ___|_   _|_   _|  \/  |  ___|
 \ `--.| |_| // _ \| | | |__   | |   | | | .  . | |__  
  `--. \  ___/ ___ \ |___| |___  | |  | || |  | | |___ 
 \____/\_|  \/     \/\__|_____|\_/  \___/\_|  |_|_____|
"#;
    println!("{}", logo.white().bold());
    println!(
        "{}\n",
        "A benchmark for evaluating AI agents on interactive terminal tasks."
            .dimmed()
    );
}

struct TaskOption<'a>(&'a BenchmarkTask);

impl<'a> fmt::Display for TaskOption<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.id)
    }
}

pub fn select_task_interactively<'a>(tasks: &'a [BenchmarkTask]) -> Result<&'a BenchmarkTask> {
    print_banner();

    let render_config = RenderConfig {
        prompt_prefix: Styled::new("> ").with_fg(Color::DarkGrey),
        prompt: StyleSheet::new().with_fg(Color::White).with_attr(Attributes::BOLD),
        highlighted_option_prefix: Styled::new("❯ ").with_fg(Color::White).with_attr(Attributes::BOLD),
        option: StyleSheet::new().with_fg(Color::DarkGrey),
        selected_option: Some(StyleSheet::new().with_fg(Color::White).with_attr(Attributes::BOLD)),
        ..Default::default()
    };

    let options: Vec<TaskOption<'a>> = tasks.iter().map(TaskOption).collect();

    let ans = Select::new("Select a task to run:", options)
        .with_page_size(10)
        .with_render_config(render_config)
        .prompt()
        .map_err(|e| anyhow!("Selection prompt error: {}", e))?;

    Ok(ans.0)
}

pub struct TerminalObserver {
    spinner: Option<ProgressBar>,
}

impl TerminalObserver {
    pub fn new() -> Self {
        Self { spinner: None }
    }
}

impl Default for TerminalObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl EvaluationObserver for TerminalObserver {
    fn on_turn_start(&mut self, turn: usize) {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner} [{elapsed_precise}] {msg}")
                .unwrap(),
        );
        pb.set_message(format!("Turn {}: Agent thinking...", turn));
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        self.spinner = Some(pb);
    }

    fn on_reasoning(&mut self, turn: usize, reasoning: &str) {
        if let Some(pb) = self.spinner.take() {
            pb.finish_and_clear();
        }
        println!(
            "\n{}",
            format!("• Turn {} Agent Thought:", turn).bold().white()
        );
        for line in reasoning.lines() {
            println!("  {}", line.dimmed());
        }
    }

    fn on_command(
        &mut self,
        _turn: usize,
        command: &str,
        stdout: &str,
        stderr: &str,
        exit_code: i64,
    ) {
        println!("{}", format!("  $ {}", command).bold().white());
        println!("{}", format!("    [exit: {}]", exit_code).dimmed());

        if !stdout.is_empty() {
            for line in stdout.lines() {
                println!("    {}", line.dimmed());
            }
        }
        if !stderr.is_empty() {
            for line in stderr.lines() {
                println!("    {}", line.dimmed());
            }
        }
    }
}

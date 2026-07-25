use crate::engine::EvaluationObserver;
use crate::task::BenchmarkTask;
use anyhow::{anyhow, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Select;
use std::fmt;

pub fn print_banner() {
    let logo = r#"
  ____  ____   _    ____ _____ _____ ___ __  __ _____ 
 / ___||  _ \ / \  / ___| ____|_   _|_ _|  \/  | ____|
 \___ \| |_) / _ \| |   |  _|   | |  | || |\/| |  _|  
  ___) |  __/ ___ \ |___| |___  | |  | || |  | | |___ 
 |____/|_| /_/   \_\____|_____| |_| |___|_|  |_|_____|
"#;
    println!("{}", logo.bright_cyan().bold());
    println!(
        "{}\n",
        "A benchmark for evaluating AI agents on interactive terminal tasks."
            .dimmed()
            .italic()
    );
}

struct TaskOption<'a>(&'a BenchmarkTask);

impl<'a> fmt::Display for TaskOption<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.0.id.bold(), self.0.name)
    }
}

pub fn select_task_interactively<'a>(tasks: &'a [BenchmarkTask]) -> Result<&'a BenchmarkTask> {
    print_banner();

    let options: Vec<TaskOption<'a>> = tasks.iter().map(TaskOption).collect();

    let ans = Select::new("Select a benchmark task to run:", options)
        .with_page_size(10)
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
                .template("{spinner:.cyan} [{elapsed_precise}] {msg}")
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
            format!("• Turn {} Agent Thought:", turn)
                .bold()
                .bright_blue()
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
        let exit_str = if exit_code == 0 {
            format!("exit: {}", exit_code).green()
        } else {
            format!("exit: {}", exit_code).red()
        };

        println!(
            "{}",
            format!("  $ {}", command).bold().yellow()
        );
        println!("    [{}]", exit_str);

        if !stdout.is_empty() {
            for line in stdout.lines() {
                println!("    {}", line.dimmed());
            }
        }
        if !stderr.is_empty() {
            for line in stderr.lines() {
                println!("    {}", line.red());
            }
        }
    }
}

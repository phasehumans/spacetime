pub mod runner;
pub mod tasks;
pub mod theme;
pub mod wizard;

use std::path::Path;
use anyhow::Result;

use crate::docker::ensure_sandbox_image;
use crate::task::load_all_tasks;
use crate::tui::runner::execute_benchmark_suite_tui;
use crate::tui::theme::{muted, print_banner, show_cursor};
use crate::tui::wizard::run_wizard_navigation;

pub async fn run_spacetime_wizard(
    tasks_dir: &Path,
    image: String,
    timeout: Option<u64>,
    force_rebuild: bool,
) -> Result<()> {
    print_banner();

    let all_tasks = load_all_tasks(tasks_dir)?;

    let (agent_profile, selected_tasks) = match run_wizard_navigation(&all_tasks)? {
        Some((profile, tasks)) => (profile, tasks),
        None => {
            show_cursor();
            println!("{}", muted("operation canceled."));
            return Ok(());
        }
    };

    ensure_sandbox_image(&image, force_rebuild).await?;

    let res = execute_benchmark_suite_tui(
        selected_tasks,
        agent_profile,
        image,
        timeout,
        None,
    )
    .await;

    show_cursor();
    res?;

    Ok(())
}

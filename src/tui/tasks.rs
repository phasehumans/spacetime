use anyhow::{anyhow, Result};
use inquire::{MultiSelect, Select};

use crate::types::BenchmarkTask;
use crate::tui::theme::{
    clear_lines, get_spacetime_render_config, multiselect_help_message, muted, orange,
    select_help_message, trunk, white,
};

pub fn get_task_category_tag(task_id: &str) -> &'static str {
    if task_id.contains("nginx") || task_id.contains("port") {
        "[net]"
    } else if task_id.contains("git") {
        "[git]"
    } else if task_id.contains("perm") || task_id.contains("user") || task_id.contains("ssh") {
        "[sec]"
    } else if task_id.contains("json") || task_id.contains("sqlite") || task_id.contains("base64") {
        "[data]"
    } else if task_id.contains("docker") || task_id.contains("process") {
        "[dev]"
    } else if task_id.contains("file") || task_id.contains("tar") || task_id.contains("symlink") {
        "[fs]"
    } else if task_id.contains("log") || task_id.contains("ip") {
        "[logs]"
    } else {
        "[os]"
    }
}

pub fn prompt_task_selection(all_tasks: &[BenchmarkTask]) -> Result<Option<Vec<BenchmarkTask>>> {
    if all_tasks.is_empty() {
        return Err(anyhow!("No benchmark tasks available"));
    }

    let mode_options = vec![
        format!(" run all tasks ({}/{} tasks)", all_tasks.len(), all_tasks.len()),
        format!(" select tasks individually from tree..."),
    ];

    'task_mode_loop: loop {
        let mode_choice = match Select::new(
            "select tasks to benchmark\n",
            mode_options.clone(),
        )
        .without_filtering()
        .with_help_message(&select_help_message())
        .with_render_config(get_spacetime_render_config())
        .prompt()
        {
            Ok(choice) => choice,
            Err(inquire::InquireError::OperationCanceled) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        if mode_choice.trim().to_lowercase().contains("run all tasks") {
            return Ok(Some(all_tasks.to_vec()));
        }

        // Build multi-select options
        let mut task_display_items = Vec::new();
        for t in all_tasks {
            let tag = get_task_category_tag(&t.id);
            let desc = if !t.description.is_empty() {
                &t.description
            } else {
                &t.name
            };
            task_display_items.push(format!(
                " {:<24} {:<7} {}",
                t.id,
                orange(tag),
                muted(desc)
            ));
        }

        println!("{}", trunk("│"));
        let num_items = task_display_items.len();
        let selected_indices = match MultiSelect::new(
            "select benchmark tasks (space to toggle, 'a' select all)\n",
            task_display_items,
        )
        .without_filtering()
        .with_page_size(25)
        .with_help_message(&multiselect_help_message())
        .with_render_config(get_spacetime_render_config())
        .with_default(&(0..all_tasks.len()).collect::<Vec<usize>>())
        .prompt()
        {
            Ok(indices) => indices,
            Err(inquire::InquireError::OperationCanceled) => {
                // Esc on individual task list -> clear lines and go back to mode choice
                clear_lines(num_items + 4 + 1);
                continue 'task_mode_loop;
            }
            Err(e) => return Err(e.into()),
        };

        if selected_indices.is_empty() {
            println!("{}", trunk("│"));
            println!("{} {}", trunk("│"), muted("no tasks selected, please choose at least one task."));
            println!("{}", trunk("│"));
            continue 'task_mode_loop;
        }

        let mut selected_tasks = Vec::new();
        for item in selected_indices {
            if let Some(t) = all_tasks.iter().find(|t| item.contains(&t.id)) {
                selected_tasks.push(t.clone());
            }
        }

        println!("{}", trunk("│"));
        println!(
            "{} {} {}",
            trunk("│  selected:"),
            white(&format!("{} tasks", selected_tasks.len())),
            muted("| est. timeout: 120s/task")
        );
        println!("{}", trunk("│"));

        return Ok(Some(selected_tasks));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_tagging() {
        assert_eq!(get_task_category_tag("001-nginx-config"), "[net]");
        assert_eq!(get_task_category_tag("004-port-conflict"), "[net]");
        assert_eq!(get_task_category_tag("005-resolve-git-conflict"), "[git]");
        assert_eq!(get_task_category_tag("012-fix-permissions"), "[sec]");
        assert_eq!(get_task_category_tag("003-json-parsing"), "[data]");
        assert_eq!(get_task_category_tag("006-find-largest-file"), "[fs]");
        assert_eq!(get_task_category_tag("015-fix-dockerfile"), "[dev]");
        assert_eq!(get_task_category_tag("007-extract-log-errors"), "[logs]");
    }
}

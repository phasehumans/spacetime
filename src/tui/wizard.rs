use anyhow::Result;
use inquire::{MultiSelect, Select, Text};

use crate::agent::profile::{AgentProfile, HarnessType};
use crate::tui::tasks::get_task_category_tag;
use crate::tui::theme::{
    clear_lines, get_spacetime_render_config, multiselect_help_message, muted, muted_italic,
    orange, print_breadcrumb, select_help_message, show_cursor, trunk, white,
};
use crate::types::BenchmarkTask;

#[derive(Debug, Clone)]
pub enum WizardStep {
    SelectHarness,
    SelectModel {
        harness: HarnessType,
    },
    ConfigureApiKey {
        profile: AgentProfile,
    },
    SelectTasks {
        profile: AgentProfile,
    },
    ConfirmBenchmark {
        profile: AgentProfile,
        tasks: Vec<BenchmarkTask>,
    },
}

pub fn run_wizard_navigation(
    all_tasks: &[BenchmarkTask],
) -> Result<Option<(AgentProfile, Vec<BenchmarkTask>)>> {
    let harnesses = vec![
        HarnessType::ClaudeCode,
        HarnessType::GeminiCli,
        HarnessType::Antigravity,
        HarnessType::Codex,
        HarnessType::Aider,
        HarnessType::Devin,
        HarnessType::December,
        HarnessType::Pi,
        HarnessType::CursorCli,
        HarnessType::SweAgent,
        HarnessType::OpenHands,
        HarnessType::Goose,
        HarnessType::Plandex,
        HarnessType::Cline,
        HarnessType::Smolagents,
        HarnessType::Mentat,
        HarnessType::Custom,
    ];

    let harness_options: Vec<String> = harnesses
        .iter()
        .map(|h| format!(" {:<20} {}", h.to_string(), muted(h.description())))
        .collect();

    let mut current_step = WizardStep::SelectHarness;

    loop {
        match current_step {
            WizardStep::SelectHarness => {
                let harness_choice = match Select::new("select agent harness\n", harness_options.clone())
                    .without_filtering()
                    .with_page_size(25)
                    .with_help_message(&select_help_message())
                    .with_render_config(get_spacetime_render_config())
                    .prompt()
                {
                    Ok(choice) => choice,
                    Err(inquire::InquireError::OperationInterrupted) => {
                        show_cursor();
                        std::process::exit(130);
                    }
                    Err(inquire::InquireError::OperationCanceled) => {
                        clear_lines(2);
                        return Ok(None);
                    }
                    Err(e) => return Err(e.into()),
                };

                let trimmed_choice = harness_choice.trim();
                let selected_index = harnesses
                    .iter()
                    .position(|h| trimmed_choice.contains(&h.to_string()))
                    .unwrap_or(0);
                let selected_harness = harnesses[selected_index].clone();

                clear_lines(2);
                print_breadcrumb("harness", &selected_harness.to_string());
                println!("{}", trunk("│"));

                if selected_harness == HarnessType::Custom {
                    // Step A: Agent display name
                    let name_help = format!("e.g. AutoDev, MyCoder-v1, Qwen-Runner");
                    let agent_name = match Text::new(&format!("agent display name:\n  {}", muted_italic(&name_help)))
                        .with_default("Custom-Agent")
                        .with_render_config(get_spacetime_render_config())
                        .prompt()
                    {
                        Ok(n) => {
                            let trimmed = n.trim();
                            if trimmed.is_empty() {
                                "Custom-Agent".to_string()
                            } else {
                                trimmed.to_string()
                            }
                        }
                        Err(inquire::InquireError::OperationInterrupted) => {
                            show_cursor();
                            std::process::exit(130);
                        }
                        Err(inquire::InquireError::OperationCanceled) => {
                            clear_lines(2 + 2);
                            current_step = WizardStep::SelectHarness;
                            continue;
                        }
                        Err(e) => return Err(e.into()),
                    };
                    clear_lines(2);

                    // Step B: Select execution template or enter custom
                    let template_options = vec![
                        format!(" {:<44} {}", "python3 /workspace/agent.py \"{prompt}\"", muted("[python script]")),
                        format!(" {:<44} {}", "node /workspace/index.js \"{prompt}\"", muted("[node/ts script]")),
                        format!(" {:<44} {}", "bash /workspace/agent.sh \"{prompt}\"", muted("[shell script]")),
                        format!(" custom command string..."),
                    ];

                    let template_choice = match Select::new("select execution template or enter custom\n", template_options)
                        .without_filtering()
                        .with_page_size(25)
                        .with_help_message(&select_help_message())
                        .with_render_config(get_spacetime_render_config())
                        .prompt()
                    {
                        Ok(c) => c,
                        Err(inquire::InquireError::OperationInterrupted) => {
                            show_cursor();
                            std::process::exit(130);
                        }
                        Err(inquire::InquireError::OperationCanceled) => {
                            clear_lines(2 + 2);
                            current_step = WizardStep::SelectHarness;
                            continue;
                        }
                        Err(e) => return Err(e.into()),
                    };

                    let custom_cmd = if template_choice.contains("custom command string") {
                        clear_lines(2);
                        let cmd_help = format!("use {{prompt}} where task instruction should be inserted");
                        let cmd_input = match Text::new(&format!("enter command template:\n  {}", muted_italic(&cmd_help)))
                            .with_default("./my-binary --prompt \"{prompt}\"")
                            .with_render_config(get_spacetime_render_config())
                            .prompt()
                        {
                            Ok(c) => c,
                            Err(inquire::InquireError::OperationInterrupted) => {
                                show_cursor();
                                std::process::exit(130);
                            }
                            Err(inquire::InquireError::OperationCanceled) => {
                                clear_lines(2 + 2);
                                current_step = WizardStep::SelectHarness;
                                continue;
                            }
                            Err(e) => return Err(e.into()),
                        };
                        clear_lines(2);
                        cmd_input
                    } else {
                        clear_lines(2);
                        if template_choice.contains("python3") {
                            "python3 /workspace/agent.py \"{prompt}\"".to_string()
                        } else if template_choice.contains("node") {
                            "node /workspace/index.js \"{prompt}\"".to_string()
                        } else if template_choice.contains("bash") {
                            "bash /workspace/agent.sh \"{prompt}\"".to_string()
                        } else {
                            "python3 /workspace/agent.py \"{prompt}\"".to_string()
                        }
                    };

                    // Step C: Host directory to mount inside sandbox at /workspace
                    let mount_help = format!("local folder containing your agent code or scripts");
                    let mount_input = match Text::new(&format!("host directory to mount inside sandbox at /workspace:\n  {}", muted_italic(&mount_help)))
                        .with_default(".")
                        .with_render_config(get_spacetime_render_config())
                        .prompt()
                    {
                        Ok(m) => m,
                        Err(inquire::InquireError::OperationInterrupted) => {
                            show_cursor();
                            std::process::exit(130);
                        }
                        Err(inquire::InquireError::OperationCanceled) => {
                            clear_lines(2 + 2);
                            current_step = WizardStep::SelectHarness;
                            continue;
                        }
                        Err(e) => return Err(e.into()),
                    };
                    clear_lines(2);

                    let trimmed_mount = mount_input.trim();
                    let resolved_mount = if trimmed_mount.is_empty() || trimmed_mount == "." {
                        std::env::current_dir()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|_| ".".to_string())
                    } else {
                        std::fs::canonicalize(trimmed_mount)
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|_| trimmed_mount.to_string())
                    };

                    print_breadcrumb("agent", &format!("{} ({})", agent_name, custom_cmd));
                    println!(
                        "{} {} {}",
                        trunk("│  mount ›"),
                        white(&format!("{} ➔ /workspace", resolved_mount)),
                        muted("")
                    );
                    println!("{}", trunk("│"));

                    let profile = AgentProfile::custom_with_details(
                        agent_name,
                        custom_cmd,
                        Some(resolved_mount),
                    );
                    current_step = WizardStep::SelectTasks { profile };
                } else {
                    current_step = WizardStep::SelectModel {
                        harness: selected_harness,
                    };
                }
            }

            WizardStep::SelectModel { harness } => {
                let default_models = harness.default_models();
                let mut model_options = Vec::new();

                for m in &default_models {
                    let tag_colored = muted(&m.tag);
                    model_options.push(format!(" {:<32} {}", m.id, tag_colored));
                }
                model_options.push(format!(" custom model id..."));

                let model_choice = match Select::new(
                    &format!("select model for {}\n", harness),
                    model_options.clone(),
                )
                .without_filtering()
                .with_page_size(25)
                .with_help_message(&select_help_message())
                .with_render_config(get_spacetime_render_config())
                .prompt()
                {
                    Ok(choice) => choice,
                    Err(inquire::InquireError::OperationInterrupted) => {
                        show_cursor();
                        std::process::exit(130);
                    }
                    Err(inquire::InquireError::OperationCanceled) => {
                        // Esc on model select -> clear 2 lines + 2 lines harness breadcrumb
                        clear_lines(2 + 2);
                        current_step = WizardStep::SelectHarness;
                        continue;
                    }
                    Err(e) => return Err(e.into()),
                };

                let trimmed_model_choice = model_choice.trim();
                let (selected_model, is_custom): (String, bool) =
                    if trimmed_model_choice.contains("custom model id") {
                        clear_lines(2);
                        let model_help = format!("e.g. claude-3-7-sonnet, ollama/qwen2.5-coder:32b, openrouter/deepseek-r1");
                        let id = match Text::new(&format!("enter custom model identifier:\n  {}", muted_italic(&model_help)))
                            .with_render_config(get_spacetime_render_config())
                            .prompt()
                        {
                            Ok(id) => {
                                let trimmed = id.trim().to_string();
                                if trimmed.is_empty() {
                                    clear_lines(2);
                                    current_step = WizardStep::SelectModel { harness };
                                    continue;
                                }
                                trimmed
                            }
                            Err(inquire::InquireError::OperationInterrupted) => {
                                show_cursor();
                                std::process::exit(130);
                            }
                            Err(inquire::InquireError::OperationCanceled) => {
                                clear_lines(2);
                                current_step = WizardStep::SelectModel { harness };
                                continue;
                            }
                            Err(e) => return Err(e.into()),
                        };
                        clear_lines(2);
                        (id, true)
                    } else {
                        clear_lines(2);
                        let model_id = default_models
                            .iter()
                            .find(|m| trimmed_model_choice.contains(&m.id))
                            .map(|m| m.id.clone())
                            .unwrap_or_else(|| {
                                trimmed_model_choice
                                    .split_whitespace()
                                    .next()
                                    .unwrap_or("")
                                    .to_string()
                            });
                        (model_id, false)
                    };

                let model_display = if is_custom {
                    format!("{} (custom)", selected_model)
                } else {
                    selected_model.clone()
                };

                print_breadcrumb("model", &model_display);
                println!("{}", trunk("│"));

                let profile = AgentProfile::new(harness.clone(), Some(selected_model));
                current_step = WizardStep::ConfigureApiKey { profile };
            }

            WizardStep::ConfigureApiKey { profile } => {
                if let Some(key_name) = profile.primary_api_key_name() {
                    let is_env_set = if key_name == "GEMINI_API_KEY" {
                        std::env::var("GEMINI_API_KEY").is_ok()
                            || std::env::var("GOOGLE_API_KEY").is_ok()
                    } else {
                        std::env::var(key_name)
                            .map(|v| !v.trim().is_empty())
                            .unwrap_or(false)
                    };

                    let env_tag = if is_env_set {
                        muted("(detected in environment)")
                    } else {
                        muted("(not set in environment)")
                    };

                    let api_options = vec![
                        format!(" enter {} manually", key_name),
                        format!(" load from environment {}", env_tag),
                    ];

                    let api_choice = match Select::new("configure api key\n", api_options.clone())
                        .without_filtering()
                        .with_help_message(&select_help_message())
                        .with_render_config(get_spacetime_render_config())
                        .prompt()
                    {
                        Ok(c) => c,
                        Err(inquire::InquireError::OperationInterrupted) => {
                            show_cursor();
                            std::process::exit(130);
                        }
                        Err(inquire::InquireError::OperationCanceled) => {
                            // Esc on API key prompt -> clear 2 lines + 2 lines model breadcrumb
                            clear_lines(2 + 2);
                            current_step = WizardStep::SelectModel {
                                harness: profile.harness,
                            };
                            continue;
                        }
                        Err(e) => return Err(e.into()),
                    };

                    clear_lines(2);

                    if api_choice.contains("enter") {
                        let key_input = match Text::new(&format!("enter {}:", key_name))
                            .with_render_config(get_spacetime_render_config())
                            .prompt()
                        {
                            Ok(k) => k,
                            Err(inquire::InquireError::OperationInterrupted) => {
                                show_cursor();
                                std::process::exit(130);
                            }
                            Err(inquire::InquireError::OperationCanceled) => {
                                // Esc on manual key text input -> clear 1 line and re-prompt ConfigureApiKey
                                clear_lines(1);
                                current_step = WizardStep::ConfigureApiKey { profile };
                                continue;
                            }
                            Err(e) => return Err(e.into()),
                        };

                        let trimmed_key = key_input.trim();
                        if trimmed_key.is_empty() {
                            clear_lines(1);
                            current_step = WizardStep::ConfigureApiKey { profile };
                            continue;
                        }

                        clear_lines(1);
                        std::env::set_var(key_name, trimmed_key);
                        if key_name == "GEMINI_API_KEY" {
                            std::env::set_var("GOOGLE_API_KEY", trimmed_key);
                        }

                        let masked_key = if trimmed_key.len() > 8 {
                            format!(
                                "{}...{}",
                                &trimmed_key[..4],
                                &trimmed_key[trimmed_key.len() - 4..]
                            )
                        } else {
                            "***".to_string()
                        };

                        print_breadcrumb("api key", &format!("set manually ({})", masked_key));
                        println!("{}", trunk("│"));
                    } else {
                        // Load from environment
                        let (_, is_detected) = profile.check_env_status();
                        let status_display = if is_detected {
                            format!("detected ({})", key_name)
                        } else {
                            format!("{} not set", key_name)
                        };

                        print_breadcrumb("api key", &status_display);
                        println!("{}", trunk("│"));
                    }
                } else {
                    print_breadcrumb("api key", "not required");
                    println!("{}", trunk("│"));
                }

                current_step = WizardStep::SelectTasks { profile };
            }

            WizardStep::SelectTasks { profile } => {
                let mode_options = vec![
                    format!(
                        " run all tasks ({}/{} tasks)",
                        all_tasks.len(),
                        all_tasks.len()
                    ),
                    format!(" select tasks individually from tree..."),
                ];

                let mode_choice = match Select::new("select tasks to benchmark\n", mode_options)
                    .without_filtering()
                    .with_help_message(&select_help_message())
                    .with_render_config(get_spacetime_render_config())
                    .prompt()
                {
                    Ok(c) => c,
                    Err(inquire::InquireError::OperationInterrupted) => {
                        show_cursor();
                        std::process::exit(130);
                    }
                    Err(inquire::InquireError::OperationCanceled) => {
                        // Esc on task mode select -> clear 2 lines + 2 lines api key breadcrumb
                        clear_lines(2 + 2);
                        current_step = WizardStep::ConfigureApiKey { profile };
                        continue;
                    }
                    Err(e) => return Err(e.into()),
                };

                let selected_tasks = if mode_choice.trim().to_lowercase().contains("run all tasks") {
                    clear_lines(2);
                    all_tasks.to_vec()
                } else {
                    clear_lines(2);

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
                        Err(inquire::InquireError::OperationInterrupted) => {
                            show_cursor();
                            std::process::exit(130);
                        }
                        Err(inquire::InquireError::OperationCanceled) => {
                            clear_lines(2);
                            current_step = WizardStep::SelectTasks { profile };
                            continue;
                        }
                        Err(e) => return Err(e.into()),
                    };

                    clear_lines(2);

                    let mut tasks = Vec::new();
                    for item in selected_indices {
                        if let Some(t) = all_tasks.iter().find(|t| item.contains(&t.id)) {
                            tasks.push(t.clone());
                        }
                    }

                    if tasks.is_empty() {
                        current_step = WizardStep::SelectTasks { profile };
                        continue;
                    }

                    tasks
                };

                print_breadcrumb("tasks", &format!("{} tasks selected", selected_tasks.len()));
                println!("{}", trunk("│"));

                current_step = WizardStep::ConfirmBenchmark {
                    profile,
                    tasks: selected_tasks,
                };
            }

            WizardStep::ConfirmBenchmark { profile, tasks } => {
                let confirm_options = vec![
                    format!(" start benchmark ({} tasks)", tasks.len()),
                    format!(" go back & adjust selection"),
                    format!(" cancel"),
                ];

                let choice = match Select::new("start benchmark evaluation?\n", confirm_options)
                    .without_filtering()
                    .with_help_message(&select_help_message())
                    .with_render_config(get_spacetime_render_config())
                    .prompt()
                {
                    Ok(c) => c,
                    Err(inquire::InquireError::OperationInterrupted) => {
                        show_cursor();
                        std::process::exit(130);
                    }
                    Err(inquire::InquireError::OperationCanceled) => {
                        // Esc on confirm -> clear 2 lines + 2 lines tasks breadcrumb, go back to SelectTasks
                        clear_lines(2 + 2);
                        current_step = WizardStep::SelectTasks { profile };
                        continue;
                    }
                    Err(e) => return Err(e.into()),
                };

                clear_lines(2);

                if choice.contains("start benchmark") {
                    print_breadcrumb("suite", &format!("{} • {} tasks", profile.name(), tasks.len()));
                    println!("{}", trunk("│"));
                    return Ok(Some((profile, tasks)));
                } else if choice.contains("go back") {
                    clear_lines(2);
                    current_step = WizardStep::SelectTasks { profile };
                    continue;
                } else {
                    return Ok(None);
                }
            }
        }
    }
}

use std::time::{Duration, Instant};
use anyhow::Result;
use inquire::{Select, Text};

use crate::agent::profile::{AgentProfile, HarnessType};
use crate::tui::theme::{
    clear_lines, get_spacetime_render_config, muted, print_breadcrumb,
    select_help_message, select_help_message_with_hint, show_cursor, trunk,
};

pub fn prompt_agent_profile() -> Result<Option<AgentProfile>> {
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
        .map(|h| format!(" {}", h.to_string()))
        .collect();

    let mut last_sigint: Option<Instant> = None;

    'harness_loop: loop {
        let help_msg = if let Some(sigint_time) = last_sigint {
            if sigint_time.elapsed() < Duration::from_secs(3) {
                select_help_message_with_hint(Some("Press Ctrl+C again to exit"))
            } else {
                last_sigint = None;
                select_help_message_with_hint(None)
            }
        } else {
            select_help_message_with_hint(None)
        };

        let harness_choice = match Select::new("select agent harness\n", harness_options.clone())
            .without_filtering()
            .with_page_size(25)
            .with_help_message(&help_msg)
            .with_render_config(get_spacetime_render_config())
            .prompt()
        {
            Ok(choice) => {
                last_sigint = None;
                choice
            }
            Err(inquire::InquireError::OperationInterrupted) => {
                if let Some(sigint_time) = last_sigint {
                    if sigint_time.elapsed() < Duration::from_secs(3) {
                        show_cursor();
                        std::process::exit(130);
                    }
                }
                last_sigint = Some(Instant::now());
                clear_lines(harness_options.len() + 3);
                continue 'harness_loop;
            }
            Err(inquire::InquireError::OperationCanceled) => return Ok(None),
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
            let cmd = match Text::new("enter custom in-container agent command template:")
                .with_default("python3 /agent.py {prompt}")
                .with_help_message("use {prompt} where the benchmark prompt should be inserted")
                .with_render_config(get_spacetime_render_config())
                .prompt()
            {
                Ok(c) => c,
                Err(inquire::InquireError::OperationInterrupted) => {
                    show_cursor();
                    std::process::exit(130);
                }
                Err(inquire::InquireError::OperationCanceled) => {
                    clear_lines(3 + 2);
                    continue 'harness_loop;
                }
                Err(e) => return Err(e.into()),
            };
            return Ok(Some(AgentProfile::custom(cmd)));
        }

        let default_models = selected_harness.default_models();
        let mut model_options = Vec::new();

        for m in &default_models {
            let tag_colored = muted(&m.tag);
            model_options.push(format!(
                " {:<32} {}",
                m.id,
                tag_colored
            ));
        }
        model_options.push(format!(" custom model id..."));

        'model_loop: loop {
            let model_choice = match Select::new(
                &format!("select model for {}\n", selected_harness),
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
                    clear_lines(model_options.len() + 4 + 2);
                    continue 'harness_loop;
                }
                Err(e) => return Err(e.into()),
            };

            let trimmed_model_choice = model_choice.trim();
            let (selected_model, custom_text_used): (String, bool) = if trimmed_model_choice.contains("custom model id") {
                let id = match Text::new("enter target model id:")
                    .with_render_config(get_spacetime_render_config())
                    .prompt()
                {
                    Ok(id) => {
                        let trimmed = id.trim().to_string();
                        if trimmed.is_empty() {
                            clear_lines(3);
                            continue 'model_loop;
                        }
                        trimmed
                    }
                    Err(inquire::InquireError::OperationInterrupted) => {
                        show_cursor();
                        std::process::exit(130);
                    }
                    Err(inquire::InquireError::OperationCanceled) => {
                        clear_lines(3);
                        continue 'model_loop;
                    }
                    Err(e) => return Err(e.into()),
                };
                (id, true)
            } else {
                let model_id = default_models
                    .iter()
                    .find(|m| trimmed_model_choice.contains(&m.id))
                    .map(|m| m.id.clone())
                    .unwrap_or_else(|| {
                        trimmed_model_choice.split_whitespace().next().unwrap_or("").to_string()
                    });
                (model_id, false)
            };

            if custom_text_used {
                clear_lines(3);
            } else {
                clear_lines(2);
            }

            print_breadcrumb("model", &selected_model);
            println!("{}", trunk("│"));

            let profile = AgentProfile::new(selected_harness.clone(), Some(selected_model));

            if let Some(key_name) = profile.primary_api_key_name() {
                'api_loop: loop {
                    let is_env_set = if key_name == "GEMINI_API_KEY" {
                        std::env::var("GEMINI_API_KEY").is_ok() || std::env::var("GOOGLE_API_KEY").is_ok()
                    } else {
                        std::env::var(key_name).map(|v| !v.trim().is_empty()).unwrap_or(false)
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
                            clear_lines(api_options.len() + 4 + 2);
                            continue 'model_loop;
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
                                clear_lines(1);
                                continue 'api_loop;
                            }
                            Err(e) => return Err(e.into()),
                        };

                        let trimmed_key = key_input.trim();
                        if trimmed_key.is_empty() {
                            clear_lines(1);
                            continue 'api_loop;
                        }

                        clear_lines(1);
                        std::env::set_var(key_name, trimmed_key);
                        if key_name == "GEMINI_API_KEY" {
                            std::env::set_var("GOOGLE_API_KEY", trimmed_key);
                        }

                        let masked_key = if trimmed_key.len() > 8 {
                            format!("{}...{}", &trimmed_key[..4], &trimmed_key[trimmed_key.len() - 4..])
                        } else {
                            "***".to_string()
                        };

                        print_breadcrumb("api key", &format!("set manually ({})", masked_key));
                        println!("{}", trunk("│"));
                        return Ok(Some(profile));
                    } else {
                        let (_, is_detected) = profile.check_env_status();
                        let status_display = if is_detected {
                            format!("detected ({})", key_name)
                        } else {
                            format!("{} not set", key_name)
                        };

                        print_breadcrumb("api key", &status_display);
                        println!("{}", trunk("│"));
                        return Ok(Some(profile));
                    }
                }
            } else {
                print_breadcrumb("api key", "not required");
                println!("{}", trunk("│"));
                return Ok(Some(profile));
            }
        }
    }
}

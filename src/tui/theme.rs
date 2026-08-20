use inquire::ui::{Color, RenderConfig, StyleSheet, Styled};

// Exact theme color hex tokens
// Orange: #FB923C (Primary step icon ✱, spinner ◐, category tags)
// Mint Green: #6EE7B7 (December TUI success ● pass)
// Coral Red: #FCA5A5 (December TUI error ● fail)
// Soft Off-White: #E4E4E7 (Warm, glare-free readable text)
// Muted Grey: #71717A (Secondary descriptions, shortcuts, brackets)
// Dark Trunk Grey: #3F3F46 (Connecting tree lines │, ├─, └)

pub const ANSI_ORANGE: &str = "\x1b[38;2;251;146;60m";
pub const ANSI_MINT_GREEN: &str = "\x1b[38;2;110;231;183m";
pub const ANSI_CORAL_RED: &str = "\x1b[38;2;252;165;165m";
pub const ANSI_WHITE: &str = "\x1b[38;2;228;228;231m";
pub const ANSI_MUTED_GREY: &str = "\x1b[38;2;113;113;122m";
pub const ANSI_TRUNK_GREY: &str = "\x1b[38;2;63;63;70m";
pub const ANSI_ITALIC: &str = "\x1b[3m";
pub const ANSI_RESET: &str = "\x1b[0m";

pub fn orange(text: &str) -> String {
    format!("{}{}{}", ANSI_ORANGE, text, ANSI_RESET)
}

pub fn mint_green(text: &str) -> String {
    format!("{}{}{}", ANSI_MINT_GREEN, text, ANSI_RESET)
}

pub fn coral_red(text: &str) -> String {
    format!("{}{}{}", ANSI_CORAL_RED, text, ANSI_RESET)
}

pub fn white(text: &str) -> String {
    format!("{}{}{}", ANSI_WHITE, text, ANSI_RESET)
}

pub fn muted(text: &str) -> String {
    format!("{}{}{}", ANSI_MUTED_GREY, text, ANSI_RESET)
}

pub fn muted_italic(text: &str) -> String {
    format!("{}{}{}{}", ANSI_MUTED_GREY, ANSI_ITALIC, text, ANSI_RESET)
}

pub fn trunk(text: &str) -> String {
    format!("{}{}{}", ANSI_TRUNK_GREY, text, ANSI_RESET)
}

pub fn breadcrumb(label: &str, value: &str) -> String {
    format!("{}  {} › {}", muted("✱"), muted(label), white(value))
}

pub fn print_breadcrumb(label: &str, value: &str) {
    println!("{}", breadcrumb(label, value));
}

pub fn hide_cursor() {
    print!("\x1b[?25l");
}

pub fn show_cursor() {
    print!("\x1b[?25h");
}

pub fn clear_lines(count: usize) {
    use std::io::Write;
    if count > 0 {
        print!("\x1b[{}F\x1b[J", count);
        let _ = std::io::stdout().flush();
    }
}

pub fn select_help_message() -> String {
    select_help_message_with_hint(None)
}

pub fn select_help_message_with_hint(hint: Option<&str>) -> String {
    if let Some(h) = hint {
        format!(
            "\r \n[{} to move, {} to select]  {}\x1b[38;2;0;0;0m",
            orange("↑↓"),
            orange("enter"),
            muted(h)
        )
    } else {
        format!(
            "\r \n[{} to move, {} to select",
            orange("↑↓"),
            orange("enter")
        )
    }
}

pub fn multiselect_help_message() -> String {
    format!(
        "\r \n[{} to move, {} to select, {} to toggle all, {} to confirm",
        orange("↑↓"),
        orange("space"),
        orange("'a'"),
        orange("enter")
    )
}

/// Builds the custom Inquire RenderConfig matching the Spacetime Clack theme
pub fn get_spacetime_render_config() -> RenderConfig<'static> {
    let mut config = RenderConfig::empty();

    let orange_col = Color::rgb(251, 146, 60);
    let grey_col = Color::rgb(113, 113, 122);
    let white_col = Color::rgb(228, 228, 231);

    // Prompt prefix: soft orange ✱
    config.prompt_prefix = Styled::new("✱").with_fg(orange_col);
    config.answered_prompt_prefix = Styled::new("✱").with_fg(grey_col);

    // Select markers: clean ❭ cursor in soft orange (matching ✱)
    config.highlighted_option_prefix = Styled::new("❭").with_fg(orange_col);
    config.option = StyleSheet::new().with_fg(white_col);
    config.selected_option = Some(StyleSheet::new().with_fg(orange_col));

    // Checkbox markers for multi-select
    config.selected_checkbox = Styled::new("[x] ").with_fg(white_col);
    config.unselected_checkbox = Styled::new("[ ] ").with_fg(grey_col);

    // Scroll indicators: single space so no arrow icons appear
    config.scroll_up_prefix = Styled::new(" ");
    config.scroll_down_prefix = Styled::new(" ");

    // Help message in muted grey (brackets and action text)
    config.help_message = StyleSheet::new().with_fg(grey_col);
    config.default_value = StyleSheet::new().with_fg(grey_col);
    config.placeholder = StyleSheet::new().with_fg(grey_col);

    config
}

pub fn print_banner() {
    println!(
        "\n{} {} {}",
        orange("✱"),
        white("SPACETIME:"),
        muted("a benchmark for evaluating ai agents on terminal tasks")
    );
    println!("{}", trunk("│"));
}

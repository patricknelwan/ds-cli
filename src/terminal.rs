use std::{
    io::{self, Write},
    path::Path,
};

use rustyline::{
    At, Cmd, DefaultEditor, EventHandler, KeyCode, KeyEvent, Modifiers, Movement, Word,
};
use termimad::{CompoundStyle, MadSkin, rgb};

pub fn print_intro() {
    println!("ds interactive — /model to switch, /exit to quit");
}

pub fn print_response(markdown: &str) -> io::Result<()> {
    print_separator();
    println!();
    response_skin().print_text(&codex_markdown(markdown));
    println!();
    print_separator();
    io::stdout().flush()
}

fn response_skin() -> MadSkin {
    let mut skin = MadSkin::default_dark();
    skin.headers[0].set_fg(rgb(96, 165, 250));
    skin.headers[1].set_fg(rgb(94, 234, 212));
    skin.headers[2].set_fg(rgb(196, 181, 253));
    skin.bold.set_fg(rgb(248, 250, 252));
    skin.inline_code = CompoundStyle::with_fg(rgb(251, 191, 36));
    skin.code_block = CompoundStyle::with_fg(rgb(226, 232, 240)).into();
    skin.bullet.set_fg(rgb(94, 234, 212));
    skin
}

fn print_separator() {
    let (width, _) = termimad::terminal_size();
    println!(
        "{}",
        CompoundStyle::with_fg(rgb(71, 85, 105)).apply_to(separator(width))
    );
}

fn separator(width: u16) -> String {
    "─".repeat(width.max(1).into())
}

fn codex_markdown(markdown: &str) -> String {
    let mut formatted = String::with_capacity(markdown.len() + markdown.len() / 10);
    let mut in_code = false;
    let mut block_start = true;

    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code = !in_code;
        }
        if !line.trim().is_empty() && block_start && !in_code && !is_markdown_block(line) {
            formatted.push_str("- ");
        }
        formatted.push_str(line);
        formatted.push('\n');
        block_start = line.trim().is_empty();
    }
    formatted
}

fn is_markdown_block(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with('#')
        || line.starts_with('>')
        || line.starts_with("- ")
        || line.starts_with("* ")
        || line.starts_with("+ ")
        || line.starts_with("```")
        || line.chars().next().is_some_and(|c| c.is_ascii_digit())
}

pub fn print_working() {
    println!("\n• Working…");
}

pub fn prompt_for_key() -> Result<String, Box<dyn std::error::Error>> {
    let key = rpassword::prompt_password("DeepSeek API key: ")?;
    if key.is_empty() || key.contains(['\n', '\r']) {
        return Err("a non-empty, single-line API key is required".into());
    }
    Ok(key)
}

pub fn prompt_editor() -> Result<DefaultEditor, rustyline::error::ReadlineError> {
    let mut editor = DefaultEditor::new()?;
    editor.bind_sequence(
        KeyEvent(KeyCode::Delete, Modifiers::ALT),
        EventHandler::Simple(alt_delete_command()),
    );
    Ok(editor)
}

fn alt_delete_command() -> Cmd {
    Cmd::Kill(Movement::ForwardWord(1, At::AfterEnd, Word::Emacs))
}

pub fn read_prompt(
    editor: &mut DefaultEditor,
    model: &str,
) -> Result<String, rustyline::error::ReadlineError> {
    let prompt = format!("[{model}] > ");
    let input = editor.readline(&prompt)?;
    let input = input.trim().to_owned();
    if !input.is_empty() {
        editor.add_history_entry(&input)?;
    }
    Ok(input)
}

pub fn choose_model(
    models: &[String],
    current: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("Models:");
    for (index, model) in models.iter().enumerate() {
        println!(
            "  {}. {}{}",
            index + 1,
            model,
            if *model == current { " (current)" } else { "" }
        );
    }
    let choice = read_line("Choose model (Enter keeps current): ")?;
    if choice.is_empty() {
        return Ok(current.into());
    }
    choice
        .parse::<usize>()
        .ok()
        .and_then(|index| models.get(index.saturating_sub(1)))
        .cloned()
        .ok_or_else(|| "choose a listed model number".into())
}

pub fn confirm_write(path: &Path) -> io::Result<bool> {
    Ok(matches!(
        read_line(&format!("Allow write to {}? [y/N] ", path.display()))?.as_str(),
        "y" | "Y"
    ))
}

pub fn confirm_shell(command: &str) -> io::Result<bool> {
    println!("\n• Shell command requested (not sandboxed):");
    println!(
        "  └ {}",
        CompoundStyle::with_fg(rgb(100, 116, 139)).apply_to(command.escape_default())
    );
    Ok(matches!(
        read_line("Allow command? [y/N] ")?.as_str(),
        "y" | "Y"
    ))
}

fn read_line(label: &str) -> Result<String, io::Error> {
    print!("{label}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().into())
}

#[cfg(test)]
mod tests {
    use super::{At, Cmd, Movement, Word, alt_delete_command, codex_markdown, separator};

    #[test]
    fn separator_matches_terminal_width() {
        assert_eq!(separator(4), "────");
    }

    #[test]
    fn alt_delete_removes_the_next_word() {
        assert_eq!(
            alt_delete_command(),
            Cmd::Kill(Movement::ForwardWord(1, At::AfterEnd, Word::Emacs))
        );
    }

    #[test]
    fn prose_gets_codex_bullets_without_changing_lists_or_code() {
        let markdown = "First paragraph.\n\n- Existing item\n\n```sh\necho 5\n```";
        assert_eq!(
            codex_markdown(markdown),
            "- First paragraph.\n\n- Existing item\n\n```sh\necho 5\n```\n"
        );
    }
}

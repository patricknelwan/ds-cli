use std::{
    io::{self, Write},
    path::Path,
};

pub fn print_intro() {
    println!("ds interactive — /model to switch, /exit to quit");
}

pub fn start_response() -> io::Result<()> {
    println!();
    io::stdout().flush()
}

pub fn print_chunk(chunk: &str) -> io::Result<()> {
    print!("{chunk}");
    io::stdout().flush()
}

pub fn finish_response() -> io::Result<()> {
    println!("\n");
    io::stdout().flush()
}

pub fn print_working() {
    println!("[working…]");
}

pub fn has_visible_text(chunk: &str) -> bool {
    !chunk.trim().is_empty()
}

pub fn prompt_for_key() -> Result<String, Box<dyn std::error::Error>> {
    let key = rpassword::prompt_password("DeepSeek API key: ")?;
    if key.is_empty() || key.contains(['\n', '\r']) {
        return Err("a non-empty, single-line API key is required".into());
    }
    Ok(key)
}

pub fn read_prompt(model: &str) -> Result<String, io::Error> {
    read_line(&format!("[{model}] > "))
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
    println!(
        "Shell command requested (not sandboxed):\n{}",
        command.escape_default()
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
    use super::*;

    #[test]
    fn whitespace_does_not_start_a_response() {
        assert!(!has_visible_text(" \n"));
        assert!(has_visible_text("hello"));
    }
}

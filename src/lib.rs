mod config;
mod deepseek;
mod models;
mod terminal;
mod tools;

use std::env;

use serde_json::{Value, json};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let path = config::path()?;
    let mut config = config::load(&path)?;
    let env_api_key = env::var("DEEPSEEK_API_KEY")
        .ok()
        .filter(|key| !key.is_empty());
    let api_key = env_api_key
        .clone()
        .or_else(|| config.api_key.clone())
        .map(Ok)
        .unwrap_or_else(terminal::prompt_for_key)?;
    let models = models::fetch(&api_key)?;
    let env_model = env::var("DS_MODEL").ok();
    let mut model = env_model
        .filter(|model| models.contains(model))
        .or_else(|| {
            (!config.model.is_empty())
                .then_some(config.model.clone())
                .filter(|model| models.contains(model))
        })
        .unwrap_or_else(|| models[0].clone());

    if config.api_key.is_none() && env_api_key.is_none() {
        config.api_key = Some(api_key.clone());
        config::save(&path, &config)?;
    }

    terminal::print_intro();
    let workspace = tools::Workspace::current()?;
    let tool_definitions = tools::definitions();
    let mut messages = vec![json!({
        "role": "system",
        "content": "You are a helpful coding agent. Use the provided file tools to inspect the working directory when needed. Shell commands require explicit user approval."
    })];
    let first_prompt = env::args().skip(1).collect::<Vec<_>>().join(" ");
    if !first_prompt.is_empty() {
        ask(
            &api_key,
            &model,
            &first_prompt,
            &workspace,
            &tool_definitions,
            &mut messages,
        )?;
    }

    loop {
        let prompt = terminal::read_prompt(&model)?;
        match prompt.as_str() {
            "" => continue,
            "/exit" | "/quit" => break,
            "/model" => {
                model = terminal::choose_model(&models, &model)?;
                config.model = model.clone();
                config::save(&path, &config)?;
            }
            _ => ask(
                &api_key,
                &model,
                &prompt,
                &workspace,
                &tool_definitions,
                &mut messages,
            )?,
        }
    }
    Ok(())
}

fn ask(
    api_key: &str,
    model: &str,
    prompt: &str,
    workspace: &tools::Workspace,
    tool_definitions: &[Value],
    messages: &mut Vec<Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    const MAX_TOOL_ROUNDS: usize = 8; // ponytail: cap prevents an accidental endless tool loop; make configurable only if needed.

    let initial_message_count = messages.len();
    messages.push(json!({ "role": "user", "content": prompt }));
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let mut used_tools = false;
        for _ in 0..MAX_TOOL_ROUNDS {
            let response =
                deepseek::stream(api_key, model, messages, tool_definitions, |_| Ok(()))?;
            if !response.content.trim().is_empty() {
                terminal::print_response(&response.content)?;
            }

            let tool_calls = response.tool_calls;
            messages.push(assistant_message(response.content, &tool_calls));
            if tool_calls.is_empty() {
                return Ok(());
            }

            for call in tool_calls {
                if !used_tools {
                    terminal::print_working();
                    used_tools = true;
                }
                let result = workspace
                    .execute(&call, terminal::confirm_write, terminal::confirm_shell)
                    .unwrap_or_else(|error| format!("tool error: {error}"));
                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call.id,
                    "content": result
                }));
            }
        }
        Err("tool-call limit reached".into())
    })();
    if result.is_err() {
        messages.truncate(initial_message_count);
    }
    result
}

fn assistant_message(content: String, tool_calls: &[tools::ToolCall]) -> Value {
    let mut message = json!({
        "role": "assistant",
        "content": if content.is_empty() { Value::Null } else { Value::String(content) }
    });
    if !tool_calls.is_empty() {
        message["tool_calls"] = json!(
            tool_calls
                .iter()
                .map(|call| json!({
                    "id": call.id,
                    "type": "function",
                    "function": { "name": call.name, "arguments": call.arguments }
                }))
                .collect::<Vec<_>>()
        );
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_assistant_messages_omit_tool_calls() {
        let message = assistant_message("done".into(), &[]);
        assert!(message.get("tool_calls").is_none());
    }
}

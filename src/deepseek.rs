use std::io::{self, BufRead, BufReader};

use serde_json::{Value, json};

use crate::tools::ToolCall;

pub struct Response {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
}

pub fn stream(
    api_key: &str,
    model: &str,
    messages: &[Value],
    tools: &[Value],
    mut write_chunk: impl FnMut(&str) -> io::Result<()>,
) -> Result<Response, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::Client::new()
        .post("https://api.deepseek.com/chat/completions")
        .bearer_auth(api_key)
        .json(&json!({
            "model": model,
            "messages": messages,
            "tools": tools,
            "stream": true
        }))
        .send()?;
    let status = response.status();
    if !status.is_success() {
        let body: Value = response.json()?;
        return Err(body.to_string().into());
    }

    let mut content = String::new();
    let mut tool_calls = Vec::new();
    for line in BufReader::new(response).lines() {
        let line = line?;
        let Some(event) = line.strip_prefix("data: ") else {
            continue;
        };
        if event == "[DONE]" {
            break;
        }
        let chunk: Value = serde_json::from_str(event)?;
        if let Some(text) = chunk["choices"][0]["delta"]["content"].as_str() {
            write_chunk(text)?;
            content.push_str(text);
        }
        merge_tool_calls(&mut tool_calls, &chunk["choices"][0]["delta"]["tool_calls"])?;
    }
    if content.is_empty() && tool_calls.is_empty() {
        return Err("unexpected empty response from DeepSeek".into());
    }
    Ok(Response {
        content,
        tool_calls,
    })
}

fn merge_tool_calls(
    calls: &mut Vec<ToolCall>,
    updates: &Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(updates) = updates.as_array() else {
        return Ok(());
    };
    for update in updates {
        let index = update["index"].as_u64().ok_or("tool call has no index")? as usize;
        while calls.len() <= index {
            calls.push(ToolCall::default());
        }
        let call = &mut calls[index];
        if let Some(id) = update["id"].as_str() {
            call.id = id.into();
        }
        if let Some(name) = update["function"]["name"].as_str() {
            call.name = name.into();
        }
        if let Some(arguments) = update["function"]["arguments"].as_str() {
            call.arguments.push_str(arguments);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_fragmented_tool_call_arguments() {
        let mut calls = Vec::new();
        merge_tool_calls(
            &mut calls,
            &json!([{
                "index": 0,
                "id": "call_1",
                "function": { "name": "read_file", "arguments": "{\"path\":\"" }
            }]),
        )
        .unwrap();
        merge_tool_calls(
            &mut calls,
            &json!([{
                "index": 0,
                "function": { "arguments": "src/main.rs\"}" }
            }]),
        )
        .unwrap();
        assert_eq!(calls[0].arguments, r#"{"path":"src/main.rs"}"#);
    }
}

use serde_json::Value;

pub fn fetch(api_key: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::Client::new()
        .get("https://api.deepseek.com/models")
        .bearer_auth(api_key)
        .send()?;
    let status = response.status();
    let body: Value = response.json()?;
    if !status.is_success() {
        return Err(body.to_string().into());
    }

    let models = body["data"]
        .as_array()
        .ok_or("DeepSeek models response has no data")?
        .iter()
        .filter_map(|model| model["id"].as_str())
        .map(String::from)
        .collect::<Vec<_>>();
    if models.is_empty() {
        return Err("DeepSeek returned no models".into());
    }
    Ok(models)
}

#[cfg(test)]
mod tests {
    #[test]
    fn model_ids_are_read_from_api_response() {
        let body = serde_json::json!({"data": [{"id": "deepseek-chat"}]});
        let models = body["data"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|model| model["id"].as_str())
            .map(String::from)
            .collect::<Vec<_>>();
        assert_eq!(models, ["deepseek-chat"]);
    }
}

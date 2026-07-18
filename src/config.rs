use std::{env, fs, io, path::PathBuf};

pub struct Config {
    pub api_key: Option<String>,
    pub model: String,
}

pub fn path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .ok_or("could not find a home directory")?;
    Ok(base.join("ds").join("config"))
}

pub fn load(path: &PathBuf) -> Result<Config, io::Error> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(parse(&text)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Config {
            api_key: None,
            model: String::new(),
        }),
        Err(error) => Err(error),
    }
}

pub fn save(path: &PathBuf, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(path.parent().ok_or("invalid config path")?)?;
    let key = config
        .api_key
        .as_deref()
        .map(|key| format!("DEEPSEEK_API_KEY={key}\n"))
        .unwrap_or_default();
    fs::write(path, format!("{key}DS_MODEL={}\n", config.model))?;
    #[cfg(unix)]
    fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
    Ok(())
}

fn parse(text: &str) -> Config {
    let mut config = Config {
        api_key: None,
        model: String::new(),
    };
    for line in text.lines() {
        if let Some((name, value)) = line.split_once('=') {
            match name {
                "DEEPSEEK_API_KEY" => config.api_key = Some(value.into()),
                "DS_MODEL" if !value.is_empty() => config.model = value.into(),
                _ => {}
            }
        }
    }
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_supported_model_is_loaded() {
        let config = parse("DEEPSEEK_API_KEY=test\nDS_MODEL=deepseek-v4-pro\n");
        assert_eq!(config.api_key.as_deref(), Some("test"));
        assert_eq!(config.model, "deepseek-v4-pro");
    }
}

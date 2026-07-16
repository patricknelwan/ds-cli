use std::{
    fs, io,
    path::{Component, Path, PathBuf},
    process::Command,
};

use serde_json::{Value, json};

const MAX_FILE_SIZE: u64 = 1_000_000;
const MAX_DIRECTORY_ENTRIES: usize = 200;

#[derive(Default)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    pub fn current() -> Result<Self, io::Error> {
        Ok(Self {
            root: fs::canonicalize(std::env::current_dir()?)?,
        })
    }

    pub fn execute(
        &self,
        call: &ToolCall,
        confirm_write: impl FnOnce(&Path) -> io::Result<bool>,
        confirm_shell: impl FnOnce(&str) -> io::Result<bool>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let arguments: Value = serde_json::from_str(&call.arguments)?;
        match call.name.as_str() {
            "list_files" => self.list(arguments["path"].as_str().unwrap_or(".")),
            "read_file" => self.read(arguments["path"].as_str().ok_or("path is required")?),
            "write_file" => self.write(
                arguments["path"].as_str().ok_or("path is required")?,
                arguments["content"].as_str().ok_or("content is required")?,
                confirm_write,
            ),
            "run_shell" => self.shell(
                arguments["command"].as_str().ok_or("command is required")?,
                confirm_shell,
            ),
            _ => Err(format!("unknown tool: {}", call.name).into()),
        }
    }

    fn list(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let path = self.existing_path(input)?;
        let mut entries = fs::read_dir(path)?
            .filter_map(Result::ok)
            .map(|entry| {
                let kind = entry
                    .file_type()
                    .map(|kind| if kind.is_dir() { "/" } else { "" })?;
                Ok(format!("{}{}", entry.file_name().to_string_lossy(), kind))
            })
            .collect::<Result<Vec<_>, io::Error>>()?;
        entries.sort();
        let truncated = entries.len() > MAX_DIRECTORY_ENTRIES;
        entries.truncate(MAX_DIRECTORY_ENTRIES);
        if truncated {
            entries.push("... (directory listing truncated)".into());
        }
        Ok(entries.join("\n"))
    }

    fn read(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let path = self.existing_path(input)?;
        if !path.is_file() {
            return Err("path is not a file".into());
        }
        if fs::metadata(&path)?.len() > MAX_FILE_SIZE {
            return Err("file is larger than 1 MB".into());
        }
        Ok(fs::read_to_string(path)?)
    }

    fn write(
        &self,
        input: &str,
        content: &str,
        confirm_write: impl FnOnce(&Path) -> io::Result<bool>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let path = self.write_path(input)?;
        if !confirm_write(&path)? {
            return Ok("write cancelled by user".into());
        }
        fs::write(&path, content)?;
        Ok(format!(
            "wrote {} bytes to {}",
            content.len(),
            self.display_path(&path)
        ))
    }

    fn shell(
        &self,
        command: &str,
        confirm_shell: impl FnOnce(&str) -> io::Result<bool>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if !confirm_shell(command)? {
            return Ok("shell command cancelled by user".into());
        }
        let output = Command::new("sh")
            .args(["-c", command])
            .current_dir(&self.root)
            .output()?;
        Ok(format!(
            "status: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }

    fn existing_path(&self, input: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let path = fs::canonicalize(self.root.join(relative_path(input)?))?;
        if !path.starts_with(&self.root) {
            return Err("path is outside the working directory".into());
        }
        Ok(path)
    }

    fn write_path(&self, input: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let path = self.root.join(relative_path(input)?);
        if path.exists() {
            let path = fs::canonicalize(path)?;
            if !path.starts_with(&self.root) {
                return Err("path is outside the working directory".into());
            }
            return Ok(path);
        }
        let parent = path.parent().ok_or("invalid path")?;
        let parent = fs::canonicalize(parent)?;
        if !parent.starts_with(&self.root) {
            return Err("path is outside the working directory".into());
        }
        Ok(parent.join(path.file_name().ok_or("invalid path")?))
    }

    fn display_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .display()
            .to_string()
    }
}

pub fn definitions() -> Vec<Value> {
    vec![
        json!({
            "type": "function",
            "function": {
                "name": "list_files",
                "description": "List entries in a directory inside the working directory.",
                "parameters": {
                    "type": "object",
                    "properties": { "path": { "type": "string" } },
                    "required": []
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "run_shell",
                "description": "Run a shell command from the working directory. The user must approve every command. Commands can access outside the working directory, so use only when necessary.",
                "parameters": {
                    "type": "object",
                    "properties": { "command": { "type": "string" } },
                    "required": ["command"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "read_file",
                "description": "Read a UTF-8 text file inside the working directory.",
                "parameters": {
                    "type": "object",
                    "properties": { "path": { "type": "string" } },
                    "required": ["path"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "write_file",
                "description": "Write a UTF-8 text file inside the working directory. The user must approve every write.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "required": ["path", "content"]
                }
            }
        }),
    ]
}

fn relative_path(input: &str) -> Result<&Path, io::Error> {
    let path = Path::new(input);
    if path
        .components()
        .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
    {
        Ok(path)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "path must be relative",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_parent_directory_paths() {
        assert!(relative_path("../secret").is_err());
    }

    #[test]
    fn write_requires_confirmation_and_stays_in_root() {
        let root = std::env::temp_dir().join(format!("ds-cli-test-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let workspace = Workspace {
            root: fs::canonicalize(&root).unwrap(),
        };
        let call = ToolCall {
            name: "write_file".into(),
            arguments: r#"{"path":"note.txt","content":"hello"}"#.into(),
            ..Default::default()
        };
        assert_eq!(
            workspace
                .execute(&call, |_| Ok(false), |_| Ok(false))
                .unwrap(),
            "write cancelled by user"
        );
        assert!(!root.join("note.txt").exists());
        workspace
            .execute(&call, |_| Ok(true), |_| Ok(false))
            .unwrap();
        assert_eq!(fs::read_to_string(root.join("note.txt")).unwrap(), "hello");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn shell_requires_confirmation() {
        let workspace = Workspace::current().unwrap();
        let call = ToolCall {
            name: "run_shell".into(),
            arguments: r#"{"command":"printf ok"}"#.into(),
            ..Default::default()
        };
        assert_eq!(
            workspace
                .execute(&call, |_| Ok(false), |_| Ok(false))
                .unwrap(),
            "shell command cancelled by user"
        );
        assert!(
            workspace
                .execute(&call, |_| Ok(false), |_| Ok(true))
                .unwrap()
                .contains("stdout:\nok")
        );
    }
}

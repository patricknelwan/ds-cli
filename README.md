# ds

Hello! this is my deepseek cli writen in rust

```sh
cargo install --path . --force
ds
```

On first run, `ds` asks for your API key which you HAVE to get it yourself, then your API key is stored in `~/.config/ds/config` with owner-only permissions. Type `/model` to choose a model and `/exit` to quit. `DEEPSEEK_API_KEY` and `DS_MODEL` still override the saved settings for a single run.

`ds` can list and read files under the directory where it starts. It asks before every write and shell command. Shell commands start in that directory but are not sandboxed, so please approve only commands you understand.

## Architecture

```text
ds-cli/
├── Cargo.toml          # Rust package, binary name, and dependencies
├── README.md           # Usage and architecture documentation
└── src/
    ├── main.rs         # Process entry point and exit-code handling
    ├── lib.rs          # Application orchestration and agent tool loop
    ├── config.rs       # Reads and writes ~/.config/ds/config securely
    ├── deepseek.rs     # DeepSeek streaming API client and tool-call parsing
    ├── models.rs       # Supported DeepSeek model names
    ├── terminal.rs     # Console prompts, output, and write confirmation UI
    └── tools.rs        # Sandboxed file tools and confirmed shell execution
```

You can also start with an initial prompt:

```sh
ds "how tall is LeBron James?"
```

## Releases

GitHub Actions runs formatting, Clippy, tests, and a release build for pull
requests and `main`. Push a version tag to publish a Linux x86_64 archive and
its SHA-256 checksum as a GitHub Release:

```sh
git tag v0.1.0
git push origin v0.1.0
```

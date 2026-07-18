# ds

Hello! this is my deepseek cli writen in rust

```sh
cargo install --path . --force
ds
```

On first run, `ds` asks for your API key which you HAVE to get it yourself, then your API key is stored in `~/.config/ds/config` with owner-only permissions. Type `/model` to choose a model and `/exit` to quit. `DEEPSEEK_API_KEY` and `DS_MODEL` still override the saved settings for a single run.

`ds` can list and read files under the directory where it starts. It asks before every write and shell command. Shell commands start in that directory but are not sandboxed, so please approve only commands you understand.

Responses are rendered as colored Markdown with Codex-style bullets and tree
lines. Prompts support arrow-key editing, session history, `Alt+Backspace` or
`Ctrl+W` to delete the previous word, and `Alt+Delete` or `Alt+D` to delete the
next word.

## Local project context

Create a `.deep` directory in the project where `ds` starts. Every Markdown
file below it is loaded recursively at startup and provided to the model as
local project guidance:

```text
.deep/
├── project.md
├── conventions.md
└── frontend/
    └── rules.md
```

Only `.md` files are read. Files larger than 1 MB and non-UTF-8 files are
skipped. Restart `ds` after changing `.deep` files. If this context is local
only, add `.deep/` to your project’s `.gitignore`.

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
    ├── models.rs       # Fetches available DeepSeek model names
    ├── terminal.rs     # Readline prompts, formatted output, and confirmations
    └── tools.rs        # Sandboxed file tools and confirmed shell execution
```

You can also start with an initial prompt:

```sh
ds "how tall is LeBron James?"
```

## Releases

Pushing a `v*` tag publishes Linux x86_64 archives plus Debian (`.deb`) and
Fedora/RHEL (`.rpm`) packages to GitHub Releases. Each release includes a
`SHA256SUMS` file for verification.

After downloading a release package, install it with:

```sh
sudo apt install ./ds-cli_<version>-1_amd64.deb  # Debian/Ubuntu
sudo dnf install ./ds-cli-<version>-1.x86_64.rpm # Fedora/RHEL
```

## Releases

GitHub Actions runs formatting, Clippy, tests, and a release build for pull
requests and `main`. Push a version tag to publish a Linux x86_64 archive and
its SHA-256 checksum as a GitHub Release:

```sh
git tag v0.1.0
git push origin v0.1.0
```

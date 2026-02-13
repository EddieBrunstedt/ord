# Ord

**Process text through customizable LLM templates.**

Ord lets you create reusable prompts and run them from the command line. Read from clipboard, pipe from stdin, or pass text directly—get results back instantly.

## Why Ord?

| Step | Traditional workflow  | With Ord                            |
| ---- | --------------------- | ----------------------------------- |
| 1    | Copy text             | Copy text                           |
| 2    | Open ChatGPT/Claude   | Run `ord run grammar -f text -c -n` |
| 3    | Paste and type prompt | Paste result                        |
| 4    | Wait, copy result     |                                     |
| 5    | Switch back, paste    |                                     |

**Bind Ord commands to keyboard shortcuts for even faster workflows.**

## Quick Start

**1. Install** (requires [Rust](https://rustup.rs/))

```bash
git clone https://gitlab.com/EddieBrunstedt/ord && cd ord && cargo install --path .
```

**2. Set your API key**

```bash
export ANTHROPIC_API_KEY="your-key"   # or OPENAI_API_KEY or OPENROUTER_API_KEY
```

**3. Run your first template**

```bash
# Copy some text to your clipboard, then:
ord run grammar -f text -c -n
```

<details>
<summary><b>Troubleshooting: command not found</b></summary>

Ensure `~/.cargo/bin` is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add this to your shell config (`~/.bashrc`, `~/.zshrc`, etc.) to make it permanent.

</details>

On first run, Ord will offer to install example templates to get you started.

## Usage

```
ord run <TEMPLATE> [OPTIONS]       Run a template
ord list                           List available templates
ord info <TEMPLATE>                Show template details
ord --help                         Full help with examples
```

**Common flags:**

| Flag                         | Description                                |
| ---------------------------- | ------------------------------------------ |
| `-f, --from-clipboard <KEY>` | Use clipboard content for placeholder      |
| `-c, --copy-to-clipboard`    | Copy result to clipboard                   |
| `-n, --notify`               | Desktop notification on completion         |
| `-b, --browser`              | Open result in browser (rendered markdown) |

**Examples:**

```bash
ord run grammar -f text -c                          # Fix grammar from clipboard
ord run grammar -f text -c -n                       # Same, with notification
ord run summarize -f text -b                        # Summarize and view in browser
ord run commit_message text="$(git diff --staged)"  # Generate commit message from staged changes
```

## Templates

Templates are markdown files with YAML front matter. They live in `ord/templates/` under your OS config directory.

```markdown
---
description: What this template does
---

Your prompt here. Use {{:placeholder}} for dynamic content.
```

**Example — grammar.md:**

```markdown
---
description: Correct grammar and spelling
---

Correct grammar and spelling. Improve clarity where needed.
Preserve meaning and tone. Output only the corrected text.

Text:
{{:text}}
```

Run with: `ord run grammar -f text -c -n`

See [example_templates/](example_templates/) for more ready-to-use templates:
grammar, summarize, definition, shorten, eli5, summarize_logs, commit_message.

## Configuration

Ord stores its config in `ord/` under your OS config directory:

| OS      | Location                                    |
| ------- | ------------------------------------------- |
| Linux   | `~/.config/ord/` or `$XDG_CONFIG_HOME/ord/` |
| macOS   | `~/Library/Application Support/ord/`        |
| Windows | `%APPDATA%\ord\`                            |

**config.toml:**

```toml
provider = "anthropic"       # or "openai", "openrouter"
model = "claude-sonnet-4-5"  # any model from your provider
max_tokens = 4096            # optional, default 4096
```

If the configuration file is missing, Ord will automatically create one with default settings.

**API Keys:**

| Provider  | Environment Variable | Get a key                                                                      |
| --------- | -------------------- | ------------------------------------------------------------------------------ |
| Anthropic | `ANTHROPIC_API_KEY`  | [platform.claude.com/settings/keys](https://platform.claude.com/settings/keys) |
| OpenAI    | `OPENAI_API_KEY`     | [platform.openai.com/api-keys](https://platform.openai.com/api-keys)           |
| OpenRouter | `OPENROUTER_API_KEY` | [openrouter.ai/settings/keys](https://openrouter.ai/settings/keys)             |

## Tips

<details>
<summary><b>Keyboard shortcuts</b></summary>

Bind Ord commands to system keyboard shortcuts for instant access:

```bash
# Example: bind to Ctrl+Alt+G for grammar correction
ord run grammar -f text -c -n
```

How to set up shortcuts varies by OS/desktop environment—check your system settings.

</details>

<details>
<summary><b>Smart commit script</b></summary>

This script uses Ord to generate a high-quality commit message from your staged changes, then lets you commit, edit, or cancel—all in one flow.

Save as `~/bin/smart_commit` (or any directory in your `$PATH`) and make executable with `chmod +x`.

**Note:** This script is for Unix-like systems (Linux, macOS).  
Windows users can run it via WSL, or adapt it to PowerShell.

```bash
#!/bin/bash
set -e

if git diff --cached --quiet; then
    echo "No staged changes."
    exit 1
fi

msg=$(ord run commit_message text="$(git diff -U20 -W --stat --staged)")

echo "Proposed commit message:"
echo "------------------------"
echo "$msg"
echo "------------------------"

read -p "Commit (y), edit (e), or cancel (n)? " c

case "$c" in
    [Yy]) git commit -m "$msg" ;;
    [Ee]) git commit -e -m "$msg" ;;
    *)    echo "Cancelled." ;;
esac
```

</details>

## License

See [LICENSE](LICENSE) for details.

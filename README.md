<div align="center">
  <h1>Ord: AI-Powered Text Processing Tool</h1>
</div>

<div align="center">
  <img src="assets/logo_original.svg" width="30%" style="border-radius: 15px;" />
</div>

<hr/>

<div align="center">Ord is a command-line application that leverages OpenAI models for text processing through customizable templates. It offers a flexible and efficient way to generate, transform, and analyze text using AI capabilities.</div>

---

[[_TOC_]]

## Features

- **Template-based Processing**: Use your own pre-defined templates for various text processing tasks.
- **Clipboard Integration**: Read the input from the clipboard and write the results back to it.
- **Browser Output**: View results in your default web browser, with optional Markdown parsing.
- **Notifications**: Receive system notifications upon task completion.
- **Customizable**: Easily add new templates or modify existing ones.
- **Interactive Mode**: Run commands interactively when omitting arguments.

## Usage

❤️ For a better understanding of Ord's capabilities, please refer to the Examples section below.

Ord provides several commands to interact with templates and process text. Here's a quick overview:

### Basic Command Structure

```
ord [COMMAND] [OPTIONS] [ARGUMENTS]
```

### Main Commands

1. **generate** (alias: gen): Process text using a template
2. **list**: Display all available templates
3. **info**: Show details about a specific template

### Generate Command

The `generate` (alias: `gen`) command is the core functionality of Ord. Here's how to use it:

```
ord generate [OPTIONS] [TEMPLATE_TITLE] [ENTRIES]...
```

- `[TEMPLATE_TITLE]`: The name of the template to use
- `[ENTRIES]...`: Key-value pairs for template placeholders (e.g., `key=value`)

**Options**:

- `-f, --from-clipboard <PLACEHOLDER>`: Use clipboard content for a placeholder
- `-c, --copy-to-clipboard`: Copy the result to clipboard
- `-n, --notify`: Show a notification (status or content)
- `-b, --browser`: Display result in browser (markdown or plain)
- `--no-new-line`: Omit newline after the result

### Interactive Mode

If you omit the template title or entries, Ord will enter interactive mode, prompting you for the necessary information.

## Templates

Templates are stored in the `templates` directory within the Ord configuration folder. Each template is a text file with a YAML front matter for metadata and the template content below. All templates must include a description.

Example template structure:

```md
---
description: Generate SQL queries based on natural language descriptions
---

Create an SQL query for the following request:

Database schema:
{{:schema}}

Request:
{{:request}}

Please provide the SQL query along with a brief explanation of how it addresses the request.
```

You can create your own templates or use/modify templates from the [example_templates](https://gitlab.com/EddieBrunstedt/ord_new/blob/main/example_templates) folder.

## Examples

### Translate text

**translate.md**:

```md
---
description: Translate text from one language to another
---

Translate this text from {{:from}} to {{:to}}:

{{:text}}
```

```bash
$ ord gen translate "from=Swedish" "to=English" "text=Far, får får får? Nej, får får lamm."

# Father, do sheep give birth to sheep? No, sheep give birth to lambs.
```

---

### Correct clipboard text grammar and overwrite with the result

**grammar.md**:

```md
---
description: Correct grammar and spelling
---

Improve the following text by correcting grammar and spelling:

{{:text}}
```

With the text "Their going to the store, but you're hat is still here" in your clipboard:

```sh
$ ord gen grammar --from-clipboard="text" --copy-to-clipboard -n

# They're going to the store, but your hat is still here.
```

The result will be ready to paste when the process is complete.

🍏 The `-n` flag displays a notification when the process is complete.

---

### Find the meaning of a word or sentence and display it in your browser

**definition.md**:

```md
---
description: Get definition of a word/phrase in markdown format
---

Please provide the definition of a word or explain the meaning of the text using markdown syntax.
Ensure the entire response is formatted in markdown without using a markdown block.
The response should have a fitting H1 heading of maximum 4 words.

The word or phrase is:

{{:text}}
```

```bash
$ ord gen definition --from-clipboard="text" -b
```

🍏The `-b` flag displays displays the result in web browser.

### Summarize text and display the result in your browser

**summarize.md**:

```md
---
description: Summarize text
---

Summarize the following text using markdown syntax.
Ensure the entire response is formatted in markdown without using a markdown block.
Include a fitting H1 heading of maximum 4 words.
Summarize the main points in bullet points and provide a concise summary paragraph.

The text is:

{{:text}}
```

```bash
$ ord gen summarize --from-clipboard="text" -b
```

## Installation

Ensure you have Rust installed on your system before proceeding.
You can install Rust using [rustup](https://rustup.rs/).

1. Clone the repository:

```bash
$ git clone https://gitlab.com/EddieBrunstedt/ord
```

2. Navigate to the cloned directory:

```bash
$ cd ord
```

3. Build and install the project:

```bash
$ cargo install --path .
```

4. Verify the installation:

```bash
$ ord --version
```

If Ord doesn't run after these steps, ensure that the Rust binary installation directory is in your PATH.

<details>
<summary>How to add the Rust binary directory to your PATH</summary>

Rust installs binaries in a specific folder that needs to be in your system's PATH for globally installed applications like Ord to be accessible from anywhere.

On Windows, the Rust installation should handle this automatically.

On Linux and macOS:

1. Add the following line to your shell configuration file (`~/.bashrc`, `~/.zshrc`, or equivalent):

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

2. Reload your shell configuration:

```bash
source ~/.bashrc  # or ~/.zshrc, depending on your shell
```

3. Verify the installation:

Open a new terminal window and run:

```bash
ord --version
```

If Ord is correctly installed and your PATH is set up, you should see the version number.

</details>

## Configuration

Ord's configuration file is located at:

| OS      | Primary Location                   | Fallback Location                               |
| ------- | ---------------------------------- | ----------------------------------------------- |
| Linux   | `$XDG_CONFIG_HOME/ord/config.toml` | `~/.config/ord/config.toml`                     |
| macOS   | `$XDG_CONFIG_HOME/ord/config.toml` | `~/Library/Application Support/ord/config.toml` |
| Windows | `%APPDATA%\ord\config.toml`        | N/A                                             |

Note: On Linux and macOS, if `$XDG_CONFIG_HOME` is not set, Ord will use the fallback location.

The configuration file should contain your OpenAI API key and the desired model:

```toml
# Ord Configuration
api_key = "your-api-key-here"
model = "gpt-4o"
```

You can find or generate your API key at: https://platform.openai.com/api-keys

---

<div align="center">
  <h3>❤️</h3>
</div>

Ord's full potential is realized when combined with quick methods for executing commands without manual terminal input. This is particularly useful for commands that process clipboard text and either overwrite the clipboard or display results in a web browser.

- On Linux, I use OS global keyboard shortcuts to run frequently used commands.
- On macOS, I use [Raycast](https://www.raycast.com/) scripts to execute Ord commands.
- On Windows, I play video games.

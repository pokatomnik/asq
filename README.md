# 🚀 asq - Quick LLM Asker

## 🤷 What is this?

**asq** is a blazingly fast CLI tool to ask questions to Large Language Models (LLMs) right from your terminal.  
It's like having ChatGPT in your command line – but cooler, because it's written in Rust 🦀.

## 🎯 What is this project for?

- Get quick answers from LLMs without leaving the terminal.
- Manage multiple LLM providers (OpenAI, Gemini, DeepSeek, NVidia, any OpenAI-compatible endpoint).
- Use **smart prompts** with built-in pipe operators like `input`, `file`, `lowercase` to fetch data, edit text, pick files, and more.
- Keep your conversations with **chat history** and **long-term memory** (yes, it remembers stuff!).
- Perfect for developers who want to integrate LLM queries into scripts or workflows.

## 🛠️ Tech Stack

| Technology             | Description                                                                |
| ---------------------- | -------------------------------------------------------------------------- |
| **Rust** 🦀            | Systems programming language, compiled for speed                           |
| **clap**               | CLI argument parsing with derive macros                                    |
| **reqwest**            | HTTP client for talking to LLM APIs (blocking + socks proxy support)       |
| **dialoguer**          | Interactive CLI prompts (fuzzy select, input, editor, password)            |
| **dom_smoothie**       | Extracts readable content from HTML (useful for web article summarization) |
| **serde / serde_json** | JSON serialization for config and API responses                            |
| **spinners**           | Show a spinner while waiting for the LLM to respond                        |
| **open**               | Opens files in browser/editor                                              |
| **urlencoding**        | URL encoding for queries                                                   |
| **ignore**             | File walking with gitignore patterns                                       |
| **marcli**             | Render Markdown output in terminal                                         |

## 📖 Usage

### First time setup

```bash
asq onboard
```

This will ask you for:

- LLM providers you want to add (OpenAI, DeepSeek, Gemini, etc.)
- Your API token (stored as an environment variable via config)
- A path to your prompts directory

### Ask a question

```bash
asq
```

Without any subcommand, `asq` will:

1. Let you select a provider (or use the last one)
2. Open a file picker to choose a **prompt template** (`.md` file)
3. Fill in any prompts via input, file contents, fetch URLs, etc.
4. Send the prompt to the LLM and show the answer

### Continue a conversation

```bash
asq -c
# or
asq --continue
```

Keeps the previous chat history, so you can ask follow-up questions.

### Change output mode

```bash
asq -o plain   # Just plain text
asq -o markdown  # Rendered Markdown (default)
```

### Manage providers

```bash
asq providers list
asq providers add
asq providers delete
```

### Manage long-term memory

```bash
asq memories list
asq memories add
asq memories delete
```

### List available operators

```bash
asq operators list
```

### Prompt template syntax

Use `{{ ... }}` to embed dynamic content or pipe operators:

```markdown
Here are the project files:
{{ "Select files" | files_picker }}
Now, please explain the architecture.

Also, check this URL: {{ "https://example.com" | fetch | htm2text }}
```

Available operators: `input`, `editor`, `file`, `fetch`, `lowercase`, `select`, `multiselect`, `password`, `file_picker`, `files_picker`, `htm2text`, `remember`, `now`.

## Shell completions

`asq` can generate shell completion scripts for supported shells.  
To enable completions, add the following line to your shell startup file:

```sh
source <(asq completions -s SHELL)
```

Replace SHELL with one of the supported shells:

- `bash`
- `zsh`
- `fish`
- `elvish`
- `powershell`

### Examples:

#### Bash:

Add this line to `~/.bashrc`:

```sh
source <(asq completions -s bash)
```

#### Zsh

Add this line to `~/.zshrc`:

```sh
source <(asq completions -s zsh)
```

#### Fish

Add this line to your Fish config file, usually ~/.config/fish/config.fish:

```sh
source (asq completions -s fish | psub)
```

#### Elvish

Add this line to your Elvish config file, usually ~/.config/elvish/rc.elv:

```sh
eval (asq completions -s elvish | slurp)
```

#### PowerShell

Add the generated script to your PowerShell profile:

```sh
asq completions -s powershell | Out-String | Invoke-Expression
```

You can place this command in your PowerShell profile file so that completions are loaded automatically in every session.

## 📄 License

This project is licensed under the **MIT License**.
Feel free to use, modify, and share it – just don't blame us if your AI goes rogue 🤖.

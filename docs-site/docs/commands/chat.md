---
id: chat
title: Chat Command
sidebar_position: 6
---

# Chat Command

Start an interactive chat session with language models. The chat command provides a conversational interface for extended interactions, maintaining context across multiple exchanges.

## Overview

Interactive chat sessions enable back-and-forth conversations with LLMs, allowing for follow-up questions, clarifications, and iterative problem-solving. Chat sessions maintain conversation history and can integrate with vector databases for RAG capabilities.

## Usage

```bash
# Start basic chat session
lc chat

# Chat with specific model
lc chat -m gpt-4

# Chat with vector database context
lc chat -v knowledge-base

# Chat with system prompt
lc chat -s "You are a helpful coding assistant"

# Using aliases
lc c
lc c -m claude
```

## Subcommands

The `chat` command is a standalone command without subcommands. All functionality is controlled through global options and interactive commands within the chat session.

## Options

Chat uses the same global options as direct prompts:

| Short | Long             | Description                        | Default |
|-------|------------------|------------------------------------|----------|
| `-p`  | `--provider`     | Specify provider                   | None     |
| `-m`  | `--model`        | Specify model                      | None     |
| `-s`  | `--system`       | System prompt                      | None     |
|       | `--max-tokens`   | Maximum tokens in response         | None     |
|       | `--temperature`  | Sampling temperature (0.0-2.0)    | None     |
| `-a`  | `--attach`       | Attach files to conversation       | None     |
| `-t`  | `--tools`        | MCP tools to include               | None     |
| `-v`  | `--vectordb`     | Vector database for RAG            | None     |
| `-d`  | `--debug`        | Enable debug output                | False    |
| `-c`  | `--continue`     | Continue previous conversation     | False    |
|       | `--cid`          | Specific chat ID to continue       | None     |
| `-h`  | `--help`         | Print help                         | False    |

## Examples

### Interactive Chat Examples

**Start a Basic Chat**

```bash
lc chat
# Opens an interactive chat session with default settings

# Specify provider and model
lc chat --provider openai -m gpt-4

# With system prompt
lc chat -s "You are an expert developer assisting me"

# Continue previous session
lc chat -c

# Using aliases
lc c
lc c -m gpt-4
```

**RAG Integration**

```bash
# Chat with vector database context
lc chat -v knowledge-base

# Include MCP tools
lc chat -t fetch,search

# Combined workflow
lc chat -v docs -m gpt-4 -s "You are a helpful assistant"
```

### Conversation Flow

```bash
# Start session
lc chat -m gpt-4

# Example conversation:
# User: "Explain machine learning"
# AI: "Machine learning is a subset of AI..."
# User: "Give me a Python example"
# AI: "Here's a simple example using scikit-learn..."
# User: "How do I evaluate this model?"
# AI: "You can evaluate models using..."
```

### Specialized Chat Sessions

**Code Review**

```bash
lc chat -s "You are an expert code reviewer" -v codebase
# Attach files for review
lc chat -a "src/main.py" -s "Review this code"
```

**Research Assistant**

```bash
lc chat -v research-papers -m gpt-4 -s "You are a research assistant"
```

**Troubleshooting**

```bash
lc chat -v docs -t diagnostic -s "Help debug issues"
```

## Troubleshooting

### Common Issues

#### "Session expired"

- **Error**: Chat session has timed out
- **Solution**: Restart the chat session with `lc chat`
- **Resume**: Use `lc chat -c` to continue last session
- **Specific**: Use `lc chat --cid <ID>` for specific session

#### "Provider not available"

- **Error**: Selected provider is offline or misconfigured
- **Solution**: Verify provider configuration with `lc providers list`
- **Check**: Ensure API keys are set with `lc keys list`

#### "Model not found"

- **Error**: Specified model doesn't exist for provider
- **Solution**: Use `lc models -q <name>` to find available models
- **Alternative**: Try different model or provider

#### "Context too long"

- **Error**: Conversation exceeds model's context limit
- **Solution**: Start new session or use model with larger context
- **Manage**: Use shorter responses or summarize conversation

### Chat Interface Commands

Within a chat session, you can use these commands:

- `/exit` or `/quit` - End the chat session
- `/clear` - Clear conversation history
- `/help` - Show available commands
- `/model <name>` - Switch to different model
- `/system <prompt>` - Set new system prompt

### Performance Tips

1. **Use appropriate models**: Choose models based on task complexity
2. **Manage context**: Keep conversations focused to avoid context limits
3. **Leverage RAG**: Use vector databases for relevant context
4. **Batch related questions**: Ask related questions in same session

### Security Considerations

- Chat sessions may contain sensitive information
- Review conversation logs before sharing
- Use secure connections for remote providers
- Consider local models for sensitive topics

```bash
# Secure chat setup
lc chat -m local-model -v private-docs
```

### Best Practices

1. **Clear objectives**: Start with specific goals for the conversation
2. **Context setting**: Use system prompts to set appropriate context
3. **Iterative refinement**: Build on previous responses for better results
4. **Session management**: Use continuation features for long workflows

## See Also

- [Providers Command](providers.md)
- [Models Command](models.md)
- [MCP Command](mcp.md)
- [Vectors Command](vectors.md)
- [Logs Command](logs.md)

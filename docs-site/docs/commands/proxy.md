---
id: proxy
title: Proxy Command
sidebar_position: 13
---

# Proxy Command

The `proxy` command in the LLM Client is used to manage proxy servers that facilitate interaction between the client and language model providers. It provides a flexible means of routing requests through a specified network point.

## Overview

The command is essential for setting up and managing network proxies, offering capabilities such as filtering by provider and model, generating API keys, and more. It can be used to customize the endpoint access settings and integrate seamlessly with different network configurations.

## Usage

```bash
# Start the proxy with default settings
lc proxy --provider openai --model gpt-3.5-turbo

# Specify a different host and port
lc proxy --host 0.0.0.0 --port 8000

# Use short flags
lc proxy --provider openai -m gpt-3.5-turbo
```

## Subcommands

| Name   | Alias | Description                      |
|--------|-------|----------------------------------|
| `add`  | `a`   | Add a new proxy entry            |
| `remove`| `r`   | Remove an existing proxy entry   |
| `list` | `l`   | List all active proxy entries    |

## Options

| Short | Long            | Description                                        | Default |
|-------|-----------------|----------------------------------------------------|---------|
|       | `--provider`    | Filter by provider                                 | None    |
| `-m`  | `--model`       | Filter by specific model (can be provider:model or alias) | None    |
| `-h`  | `--host`        | Host to bind to                                    | 127.0.0.1 |
| `-p`  | `--port`        | Port to listen on                                  | 6789    |
| `-k`  | `--key`         | API key for authentication                         | None    |
| `-g`  | `--generate-key`| Generate a random API key                          | False   |
| `-h`  | `--help`        | Print help ⚠️ **Known Issue: conflicts with --host** | False   |

## Examples

### Start a Simple Proxy

```bash
lc proxy --provider openai --model gpt-3.5-turbo
```

### Generate and Use API Key

```bash
lc proxy -g --host 0.0.0.0 --port 8000
```

### Custom Provider and Model

```bash
lc proxy --provider custom-provider -m custom-model
```

## Troubleshooting

### Common Issues

#### Conflict with Existing Port

- **Error**: Port already in use.
- **Solution**: Use the `--port` flag to specify a different port.

#### Short-flag Collision (-h)

- **Issue**: The `proxy` command and help use `-h`.
- **Solution**: Use full `--help` flag where necessary.

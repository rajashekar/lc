---
id: logs
title: Logs Command
sidebar_position: 7
---

# Logs Command

Maintain a record of interactions with LLMs. The logs command allows you to view, analyze, and manage interaction histories to monitor usage patterns, troubleshoot errors, and gain insights.

## Overview

Access historical data for all LLM interactions including prompts, responses, metadata, and usage statistics. The logs command provides comprehensive logging capabilities with options to view recent activity, analyze patterns, and maintain data hygiene through selective purging.

## Usage

```bash
# Display all logs
lc logs show

# View recent interactions
lc logs recent

# Get database statistics
lc logs stats

# Using aliases
lc l sh
lc l r
lc l s
```

## Subcommands

| Name    | Alias | Description                            |
|---------|-------|----------------------------------------|
| `show`  | `sh`  | Display all logs (with --minimal flag)|
| `stats` | `s`   | Show database statistics               |
| `purge` | `p`   | Delete all logs (with --yes flag)     |
| `recent`| `r`   | Show recent logs and details           |
| `current`| `c`  | Show current session logs              |

## Options

| Short | Long       | Description                 | Default |
|-------|------------|-----------------------------|---------|
| `-c`  | `--count`  | Number of recent entries    | 10      |
|       | `--minimal`| Minimal output for show     | False   |
|       | `--yes`    | Confirm purge operation     | False   |
| `-h`  | `--help`   | Print help                  | False   |

## Examples

### View Logs

```bash
# Show all interaction logs
lc logs show

# Show logs with minimal output
lc logs show --minimal

# View recent interactions
lc logs recent

# Limit recent entries
lc logs recent --count 5

# Get database statistics
lc logs stats

# Using aliases
lc l sh --minimal
lc l r -c 3
```

### Recent Log Details

```bash
# Get last LLM response
lc logs recent answer
lc l r a

# Get last question/prompt
lc logs recent question
lc l r q

# Get model used in last interaction
lc logs recent model
lc l r m

# Get session ID of last interaction
lc logs recent session
lc l r s
```

### Current Session

```bash
# View current session logs
lc logs current
lc l c
```

### Log Management

```bash
# Purge all logs (requires confirmation)
lc logs purge --yes
lc l p --yes

# View statistics before purging
lc logs stats
# Total entries: 1,247
# Database size: 2.3 MB
# Oldest entry: 2023-12-01
# Latest entry: 2024-01-15
```

## Troubleshooting

### Common Issues

#### "No logs found"

- **Error**: Database contains no log entries
- **Solution**: Ensure logging is enabled and interactions have occurred
- **Check**: `lc logs stats` to verify database status

#### "Database locked"

- **Error**: Cannot access log database
- **Solution**: Ensure no other lc processes are running
- **Solution**: Check file permissions on log database

#### "Permission denied"

- **Error**: Cannot read/write log files
- **Solution**: Check file system permissions
- **Fix**: `chmod 755 ~/.config/lc/logs/`

### Performance Considerations

- Large log databases may slow down queries
- Use `--minimal` flag for faster display
- Regular purging helps maintain performance
- Consider archiving important logs before purging

### Log Analysis Workflow

```bash
# Daily review workflow
lc logs recent --count 20  # Review recent activity
lc logs stats              # Check database size
lc logs current            # Review current session

# Weekly maintenance
lc logs show --minimal | grep "error"  # Find errors
lc logs purge --yes        # Clean old logs (if needed)
```

### Security and Privacy

- Logs may contain sensitive prompts and responses
- Review logs before sharing or exporting
- Use secure file permissions for log directories
- Consider log rotation for long-term storage

```bash
# Secure log directory
chmod 700 ~/.config/lc/logs/
chmod 600 ~/.config/lc/logs/*.db
```

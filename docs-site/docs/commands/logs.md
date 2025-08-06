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
| `purge` | `p`   | Delete logs with configurable options |
| `recent`| `r`   | Show recent logs and details           |
| `current`| `c`  | Show current session logs              |

## Options

| Short | Long                    | Description                      | Default |
|-------|-------------------------|----------------------------------|---------|
| `-c`  | `--count`               | Number of recent entries         | 10      |
|       | `--minimal`             | Minimal output for show          | False   |
|       | `--yes`                 | Confirm full purge operation     | False   |
|       | `--older-than-days`     | Purge logs older than N days     | None    |
|       | `--keep-recent`         | Keep only N most recent entries  | None    |
|       | `--max-size-mb`         | Purge when database exceeds N MB| None    |
| `-h`  | `--help`                | Print help                       | False   |

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
# Smart purging with configurable options
lc logs purge --older-than-days 30        # Remove logs older than 30 days
lc logs purge --keep-recent 1000           # Keep only 1000 most recent entries
lc logs purge --max-size-mb 50             # Purge when database exceeds 50MB

# Combined purging strategies
lc logs purge --older-than-days 30 --keep-recent 1000 --max-size-mb 50

# Full purge (requires confirmation)
lc logs purge --yes
lc l p --yes

# View statistics before purging
lc logs stats
# Total entries: 1,247
# Database size: 2.3 MB
# Oldest entry: 2023-12-01
# Latest entry: 2024-01-15

# Using aliases for smart purging
lc l p --older-than-days 7    # Remove logs older than 1 week
lc l p --keep-recent 500       # Keep only 500 recent entries
```

### Database Sync Integration

The logs database is automatically included in sync operations:

```bash
# Sync logs database along with configuration files
lc sync to s3
lc sync from s3

# With encryption (recommended for sensitive chat logs)
lc sync to s3 --encrypted
lc sync from s3 --encrypted
```

**Note**: Large log databases will increase sync time and storage usage. Use purging strategies to maintain optimal sync performance.

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

- Large log databases may slow down queries and sync operations
- Use `--minimal` flag for faster display
- Regular purging helps maintain performance and reduces sync time
- Consider archiving important logs before purging
- Recommended purging strategy: `lc logs purge --older-than-days 30 --keep-recent 1000 --max-size-mb 50`

### Log Analysis Workflow

```bash
# Daily review workflow
lc logs recent --count 20  # Review recent activity
lc logs stats              # Check database size
lc logs current            # Review current session

# Weekly maintenance
lc logs show --minimal | grep "error"     # Find errors
lc logs purge --older-than-days 7         # Remove logs older than 1 week
lc logs purge --max-size-mb 25             # Keep database under 25MB

# Monthly cleanup (before sync)
lc logs stats                              # Check current size
lc logs purge --older-than-days 30 --keep-recent 1000 --max-size-mb 50
lc sync to s3 --encrypted                  # Sync cleaned database
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

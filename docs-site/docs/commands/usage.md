---
id: usage
title: Usage Command
sidebar_position: 8
---

# Usage Command

Analyze and visualize your LLM usage patterns with detailed statistics and terminal bar graphs. The usage command provides comprehensive analytics including token consumption, request patterns, and model usage trends over time.

## Overview

Track your LLM interactions with rich analytics including daily, weekly, monthly, and yearly breakdowns. View token usage, request counts, and identify your most-used models with beautiful terminal bar charts.

## Usage

```bash
# Show overall usage statistics with charts
lc usage

# Show usage for the last 30 days
lc usage --days 30

# Show only token usage (hide request counts)
lc usage --tokens

# Show only request counts (hide token usage)
lc usage --requests

# Using aliases
lc u
lc u -d 7 -t
```

## Subcommands

| Name      | Alias    | Description                           |
|-----------|----------|---------------------------------------|
| `daily`   | `d`      | Show daily usage statistics           |
| `weekly`  | `w`      | Show weekly usage statistics          |
| `monthly` | `m`      | Show monthly usage statistics         |
| `yearly`  | `y`      | Show yearly usage statistics          |
| `models`  | `models` | Show top models by usage              |

## Options

| Short | Long          | Description                              | Default |
|-------|---------------|------------------------------------------|---------|
| `-d`  | `--days`      | Show usage for the last N days           | All     |
| `-t`  | `--tokens`    | Show only token usage                    | False   |
| `-r`  | `--requests`  | Show only request counts                 | False   |
| `-n`  | `--limit`     | Maximum number of items to show in charts| 10      |
| `-h`  | `--help`      | Print help                               | False   |

## Examples

### Overview Statistics

```bash
# Show comprehensive usage overview
lc usage

# Output:
# 📊 Usage Overview
# 
# Total Requests: 1,247
# Total Tokens: 2.3M
# Input Tokens: 1.1M
# Output Tokens: 1.2M
# Date Range: 2024-01-01 to 2024-01-15 (15 days)
# 
# 📈 Averages per Request
# Total Tokens: 1.8k
# Input Tokens: 883
# Output Tokens: 963
# 
# 🤖 Top Models by Token Usage
#   gpt-4-turbo    │██████████████████████████████████████████████████ 1.2M (523 req)
#   claude-3-sonnet│████████████████████████████████████████████       892.3k (401 req)
#   gpt-3.5-turbo  │██████████████████████████████                     234.1k (323 req)
```

### Time-based Analysis

```bash
# Daily usage for the last 14 days
lc usage daily --count 14
lc u d -n 14

# Weekly usage for the last 8 weeks
lc usage weekly --count 8
lc u w -n 8

# Monthly usage for the current year
lc usage monthly --count 12
lc u m -n 12

# Yearly usage trends
lc usage yearly
lc u y
```

### Model Analysis

```bash
# Top 10 models by token usage
lc usage models --count 10
lc u models -n 10

# Top 5 models by request count
lc usage models --requests --count 5
lc u models -r -n 5
```

### Filtered Views

```bash
# Show only token statistics for the last week
lc usage --days 7 --tokens
lc u -d 7 -t

# Show only request counts for the last month
lc usage --days 30 --requests
lc u -d 30 -r

# Daily token usage for the last 30 days
lc usage daily --tokens --count 30
lc u d -t -n 30
```

### Bar Chart Examples

The usage command displays beautiful terminal bar charts:

```bash
# Daily usage chart
lc usage daily

# Output:
# 📅 Daily Usage
#   2024-01-01 │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 45.2k (23 req)
#   2024-01-02 │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓    42.1k (19 req)
#   2024-01-03 │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 48.7k (25 req)

# Model usage chart
lc usage models

# Output:
# 🤖 Top Models by Usage
#   gpt-4-turbo    │██████████████████████████████████████████████████ 1.2M (523 req)
#   claude-3-sonnet│████████████████████████████████████████████       892.3k (401 req)
#   gpt-3.5-turbo  │██████████████████████████████                     234.1k (323 req)
```

## Use Cases

### Cost Analysis

```bash
# Analyze token usage to estimate costs
lc usage --tokens
lc usage models --tokens --count 5

# Review daily patterns to identify high-usage days
lc usage daily --count 30
```

### Usage Optimization

```bash
# Identify most-used models
lc usage models

# Compare request patterns over time
lc usage weekly --count 12
lc usage monthly --count 6
```

### Reporting and Monitoring

```bash
# Generate usage reports for different time periods
lc usage --days 7    # Weekly report
lc usage --days 30   # Monthly report
lc usage --days 90   # Quarterly report

# Monitor recent activity
lc usage daily --count 7
```

## Integration with Other Commands

The usage command works seamlessly with other lc commands:

```bash
# View detailed logs after usage analysis
lc usage
lc logs recent --count 10

# Purge old logs to manage database size
lc usage
lc logs purge --older-than-days 90

# Sync usage data across devices
lc sync to s3 --encrypted
```

## Troubleshooting

### Common Issues

#### "No usage data found"

- **Error**: No logged interactions in the database
- **Solution**: Ensure you have used lc for chat or direct prompts
- **Check**: `lc logs stats` to verify database status

#### "Try expanding the time range"

- **Error**: No data found for the specified time period
- **Solution**: Remove or increase the `--days` parameter
- **Example**: `lc usage --days 90` instead of `--days 7`

#### Charts appear empty or truncated

- **Issue**: Terminal width too narrow for bar charts
- **Solution**: Increase terminal width or use smaller `--limit` values
- **Example**: `lc usage --limit 5` for fewer items

### Performance Considerations

- Large databases may take longer to analyze
- Use `--days` parameter to limit analysis scope
- Consider purging old logs for better performance: `lc logs purge --older-than-days 90`

### Data Accuracy

- Token counts depend on the LLM provider's response data
- Some older entries may not have token information
- Streaming responses show as "[Streamed Response]" in logs but are counted for usage statistics

## Privacy and Security

- Usage statistics are calculated locally from your log database
- No data is sent to external services for analysis
- Consider the sensitivity of usage patterns when sharing reports
- Use encrypted sync for sensitive usage data: `lc sync to s3 --encrypted`

## Tips and Best Practices

### Regular Monitoring

```bash
# Daily usage check
lc usage daily --count 7

# Weekly model review
lc usage models --count 5

# Monthly cost analysis
lc usage monthly --tokens --count 3
```

### Optimization Strategies

```bash
# Identify high-token models
lc usage models --tokens

# Find usage patterns
lc usage daily --count 30

# Compare time periods
lc usage weekly --count 8
lc usage monthly --count 6
```

### Automation and Scripting

```bash
# Get total token count for scripting
lc usage | grep "Total Tokens:" | awk '{print $3}'

# Export usage data (combine with other tools)
lc usage > usage_report.txt
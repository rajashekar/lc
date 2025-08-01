---
id: templates
title: Templates Command
sidebar_position: 11
---

# Templates Command

Manage reusable prompt templates for standardized interactions with LLMs. Templates allow you to save commonly used prompts and system messages for consistent results.

## Overview

Templates provide a way to standardize your prompts and maintain consistency across different LLM interactions. You can create templates for various use cases like code review, email writing, data analysis, and more. Templates can include both user prompts and system messages.

## Usage

```bash
# Add a new template
lc templates add <name>

# Delete a template
lc templates delete <name>

# List all templates
lc templates list

# Using aliases
lc t a code-review
lc t d old-template
lc t l
```

## Subcommands

| Name     | Alias | Description             |
|----------|-------|-------------------------|
| `add`    | `a`   | Add a new template      |
| `delete` | `d`   | Remove a template       |
| `list`   | `l`   | List all templates      |

## Options

| Short | Long     | Description | Default |
|-------|----------|-------------|---------|
| `-h`  | `--help` | Print help  | False   |

## Examples

### Create Templates

```bash
# Add a code review template
lc templates add code-review
# Will prompt for template content

# Add an email template
lc templates add professional-email

# Using aliases
lc t a data-analysis
```

### List Templates

```bash
lc templates list
# Output:
# Available templates:
#   • code-review
#   • professional-email
#   • data-analysis
#   • bug-report

# Short form
lc t l
```

### Use Templates

```bash
# Use a template in a prompt
lc -s code-review "Review this Python function"

# Use template with specific model
lc -m gpt-4 -s professional-email "Write a follow-up email"
```

### Template Examples

**Code Review Template:**

```
You are an expert code reviewer. Analyze the following code for:
1. Logic errors and bugs
2. Performance issues
3. Security vulnerabilities
4. Code style and best practices
5. Maintainability concerns

Provide specific suggestions for improvement.
```

**Data Analysis Template:**

```
You are a data analyst. For the given dataset or query:
1. Identify patterns and trends
2. Highlight anomalies or outliers
3. Suggest relevant visualizations
4. Provide actionable insights
5. Recommend next steps for analysis

Be specific and data-driven in your response.
```

## Troubleshooting

### Common Issues

#### "Template not found"

- **Error**: Specified template doesn't exist
- **Solution**: Use `lc templates list` to see available templates
- **Solution**: Check spelling of template name

#### "Template already exists"

- **Error**: Trying to create template with existing name
- **Solution**: Use a different name or delete the existing template first

#### "No templates found"

- **Error**: No templates have been created yet
- **Solution**: Create your first template with `lc templates add <name>`

### Best Practices

1. **Use descriptive names**: Choose clear, specific names for templates
2. **Include instructions**: Be explicit about what you want the LLM to do
3. **Specify format**: Include desired output format in templates
4. **Version control**: Keep important templates in version control
5. **Regular cleanup**: Remove unused or outdated templates

### Template Organization

```bash
# Organize by category
lc t a code-python-review
lc t a code-javascript-review
lc t a email-formal
lc t a email-casual
lc t a analysis-financial
lc t a analysis-marketing
```

### Integration with Workflow

```bash
# Save frequently used prompts as templates
lc templates add summarize

# Use in daily workflow
lc -s summarize "Summarize this meeting transcript"
lc -s summarize "Summarize this research paper"
```

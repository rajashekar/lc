---
id: vectors
title: Vectors Command
sidebar_position: 10
---

# Vectors Command

Manage vector databases for semantic search and RAG (Retrieval-Augmented Generation) applications. The vectors command provides database lifecycle management including creation, deletion, and inspection.

## Overview

Vector databases store embeddings generated from text documents, enabling semantic search capabilities. The vectors command helps manage these databases, view their contents, and maintain storage efficiency.

## Usage

```bash
# List all vector databases
lc vectors list

# Show database information
lc vectors info <database>

# Delete a database
lc vectors delete <database>

# Using aliases
lc v list
lc v info docs
lc v delete old-db
```

## Subcommands

| Name     | Alias | Description                        |
|----------|-------|------------------------------------|
| `list`   | `l`   | List all vector databases          |
| `delete` | `d`   | Delete a vector database           |
| `info`   | `i`   | Show information about a database  |

## Options

| Short | Long     | Description | Default |
|-------|----------|-------------|---------|
| `-h`  | `--help` | Print help  | False   |

## Examples

### Database Management

**List Databases**

```bash
lc vectors list
# Output:
#   • project-docs (1,247 vectors)
#   • knowledge-base (523 vectors)
#   • research-papers (89 vectors)

# Short form
lc v l
```

**Database Information**

```bash
lc vectors info project-docs
# Output:
# Database: project-docs
# Vectors: 1,247
# Dimensions: 1536
# Model: text-embedding-3-small
# Size: 12.3 MB
# Created: 2024-01-15
# Last updated: 2024-01-20

lc v i project-docs
```

**Delete Database**

```bash
lc vectors delete old-project
# Will prompt for confirmation

lc v d old-project
```

### Complete RAG Workflow

```bash
# Step 1: Create embeddings
lc embed -f "docs/*.md" --vectordb project-docs

# Step 2: Verify database
lc vectors info project-docs

# Step 3: Search similar content
lc similar -v project-docs "deployment process"

# Step 4: Use in chat
lc -v project-docs "How do I deploy the application?"

# Step 5: Cleanup when done
lc vectors delete project-docs
```

## See Also

- [Embed Command](embed.md)
- [Similar Command](similar.md)

## Troubleshooting

### Common Issues

#### "Database not found"

- **Error**: Specified vector database doesn't exist
- **Solution**: Use `lc vectors list` to see available databases
- **Create**: Use `lc embed -v <name>` to create new database

#### "Permission denied"

- **Error**: Cannot access vector database files
- **Solution**: Check file permissions on database directory
- **Fix**: `chmod 755 ~/.config/lc/vectors/`

#### "Database corrupt"

- **Error**: Vector database file is corrupted
- **Solution**: Delete and recreate the database
- **Backup**: Export important data before deletion

### Best Practices

1. **Consistent Models**: Use the same embedding model for all vectors in a database
2. **Regular Cleanup**: Remove unused databases to save disk space
3. **Meaningful Names**: Use descriptive names for databases
4. **Size Management**: Monitor database sizes for performance

### Performance Considerations

- Large databases (>10k vectors) may have slower search times
- Consider splitting large databases by topic or date
- Regular maintenance improves query performance
- Monitor disk space usage

```bash
# Check database sizes
du -sh ~/.config/lc/vectors/*

# Performance monitoring
lc vectors list  # Shows vector counts
lc vectors info <db>  # Shows detailed stats
```

### Database Location

Vector databases are stored locally:

- **Linux/macOS**: `~/.config/lc/vectors/`
- **Windows**: `%APPDATA%\lc\vectors\`

Each database is a directory containing:

- Vector data files
- Metadata and configuration
- Search indices

### Security and Backup

```bash
# Backup vector databases
tar -czf vectors-backup.tar.gz ~/.config/lc/vectors/

# Restore from backup
cd ~/.config/lc/
tar -xzf vectors-backup.tar.gz

# Secure database directory
chmod 700 ~/.config/lc/vectors/
```

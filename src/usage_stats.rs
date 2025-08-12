use anyhow::Result;
use chrono::{DateTime, Utc, Duration, Datelike};
use colored::Colorize;
use std::collections::HashMap;
use crate::database::{Database, ChatEntry};

#[derive(Debug, Clone)]
pub struct UsageStats {
    pub total_tokens: u64,
    pub total_requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub model_usage: Vec<(String, u64, u64)>, // (model, requests, tokens)
    pub daily_usage: Vec<(String, u64, u64)>, // (date, requests, tokens)
    pub weekly_usage: Vec<(String, u64, u64)>, // (week, requests, tokens)
    pub monthly_usage: Vec<(String, u64, u64)>, // (month, requests, tokens)
    pub yearly_usage: Vec<(String, u64, u64)>, // (year, requests, tokens)
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

#[derive(Debug, Clone)]
pub enum TimeFrame {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

pub struct UsageAnalyzer {
    db: Database,
}

impl UsageAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            db: Database::new()?,
        })
    }

    pub fn get_usage_stats(&self, days_back: Option<u32>) -> Result<UsageStats> {
        let entries = if let Some(days) = days_back {
            let cutoff_date = Utc::now() - Duration::days(days as i64);
            self.get_entries_since(cutoff_date)?
        } else {
            self.db.get_all_logs()?
        };

        if entries.is_empty() {
            return Ok(UsageStats {
                total_tokens: 0,
                total_requests: 0,
                input_tokens: 0,
                output_tokens: 0,
                model_usage: Vec::new(),
                daily_usage: Vec::new(),
                weekly_usage: Vec::new(),
                monthly_usage: Vec::new(),
                yearly_usage: Vec::new(),
                date_range: None,
            });
        }

        let mut total_input_tokens = 0u64;
        let mut total_output_tokens = 0u64;
        let mut model_stats: HashMap<String, (u64, u64)> = HashMap::new(); // (requests, tokens)
        let mut daily_stats: HashMap<String, (u64, u64)> = HashMap::new();
        let mut weekly_stats: HashMap<String, (u64, u64)> = HashMap::new();
        let mut monthly_stats: HashMap<String, (u64, u64)> = HashMap::new();
        let mut yearly_stats: HashMap<String, (u64, u64)> = HashMap::new();

        let mut earliest_date = entries[0].timestamp;
        let mut latest_date = entries[0].timestamp;

        for entry in &entries {
            // Update date range
            if entry.timestamp < earliest_date {
                earliest_date = entry.timestamp;
            }
            if entry.timestamp > latest_date {
                latest_date = entry.timestamp;
            }

            // Calculate tokens
            let input_tokens = entry.input_tokens.unwrap_or(0) as u64;
            let output_tokens = entry.output_tokens.unwrap_or(0) as u64;
            let total_entry_tokens = input_tokens + output_tokens;

            total_input_tokens += input_tokens;
            total_output_tokens += output_tokens;

            // Model usage
            let model_entry = model_stats.entry(entry.model.clone()).or_insert((0, 0));
            model_entry.0 += 1; // requests
            model_entry.1 += total_entry_tokens; // tokens

            // Time-based usage
            let date = entry.timestamp.date_naive();
            let daily_key = date.format("%Y-%m-%d").to_string();
            let daily_entry = daily_stats.entry(daily_key).or_insert((0, 0));
            daily_entry.0 += 1;
            daily_entry.1 += total_entry_tokens;

            // Weekly usage (ISO week)
            let year = entry.timestamp.year();
            let week = entry.timestamp.iso_week().week();
            let weekly_key = format!("{}-W{:02}", year, week);
            let weekly_entry = weekly_stats.entry(weekly_key).or_insert((0, 0));
            weekly_entry.0 += 1;
            weekly_entry.1 += total_entry_tokens;

            // Monthly usage
            let monthly_key = date.format("%Y-%m").to_string();
            let monthly_entry = monthly_stats.entry(monthly_key).or_insert((0, 0));
            monthly_entry.0 += 1;
            monthly_entry.1 += total_entry_tokens;

            // Yearly usage
            let yearly_key = year.to_string();
            let yearly_entry = yearly_stats.entry(yearly_key).or_insert((0, 0));
            yearly_entry.0 += 1;
            yearly_entry.1 += total_entry_tokens;
        }

        // Convert to sorted vectors
        let mut model_usage: Vec<(String, u64, u64)> = model_stats
            .into_iter()
            .map(|(model, (requests, tokens))| (model, requests, tokens))
            .collect();
        model_usage.sort_by(|a, b| b.2.cmp(&a.2)); // Sort by tokens descending

        let mut daily_usage: Vec<(String, u64, u64)> = daily_stats
            .into_iter()
            .map(|(date, (requests, tokens))| (date, requests, tokens))
            .collect();
        daily_usage.sort_by(|a, b| a.0.cmp(&b.0)); // Sort by date ascending

        let mut weekly_usage: Vec<(String, u64, u64)> = weekly_stats
            .into_iter()
            .map(|(week, (requests, tokens))| (week, requests, tokens))
            .collect();
        weekly_usage.sort_by(|a, b| a.0.cmp(&b.0));

        let mut monthly_usage: Vec<(String, u64, u64)> = monthly_stats
            .into_iter()
            .map(|(month, (requests, tokens))| (month, requests, tokens))
            .collect();
        monthly_usage.sort_by(|a, b| a.0.cmp(&b.0));

        let mut yearly_usage: Vec<(String, u64, u64)> = yearly_stats
            .into_iter()
            .map(|(year, (requests, tokens))| (year, requests, tokens))
            .collect();
        yearly_usage.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(UsageStats {
            total_tokens: total_input_tokens + total_output_tokens,
            total_requests: entries.len() as u64,
            input_tokens: total_input_tokens,
            output_tokens: total_output_tokens,
            model_usage,
            daily_usage,
            weekly_usage,
            monthly_usage,
            yearly_usage,
            date_range: Some((earliest_date, latest_date)),
        })
    }

    fn get_entries_since(&self, cutoff_date: DateTime<Utc>) -> Result<Vec<ChatEntry>> {
        // This would need a custom query in the database
        // For now, we'll filter after getting all entries
        let all_entries = self.db.get_all_logs()?;
        Ok(all_entries
            .into_iter()
            .filter(|entry| entry.timestamp >= cutoff_date)
            .collect())
    }
}

pub struct BarChart;

impl BarChart {
    pub fn render_horizontal(
        title: &str,
        data: &[(String, u64, u64)],
        value_type: &str, // "tokens" or "requests"
        max_width: usize,
        max_items: usize,
    ) {
        if data.is_empty() {
            println!("{} No data available", "ℹ️".blue());
            return;
        }

        println!("\n{}", title.bold().blue());
        
        let display_data: Vec<_> = data.iter().take(max_items).collect();
        let max_value = display_data
            .iter()
            .map(|(_, requests, tokens)| {
                if value_type == "tokens" { *tokens } else { *requests }
            })
            .max()
            .unwrap_or(1);

        let max_label_width = display_data
            .iter()
            .map(|(label, _, _)| label.len())
            .max()
            .unwrap_or(10);

        for (label, requests, tokens) in display_data {
            let value = if value_type == "tokens" { *tokens } else { *requests };
            let bar_width = if max_value > 0 {
                ((value as f64 / max_value as f64) * max_width as f64) as usize
            } else {
                0
            };

            let bar = "█".repeat(bar_width);
            let formatted_value = if value_type == "tokens" {
                Self::format_tokens(*tokens)
            } else {
                format!("{}", requests)
            };

            println!(
                "  {:width$} │{:bar_width$} {} ({})",
                label.bold(),
                bar.green(),
                formatted_value.yellow(),
                if value_type == "tokens" {
                    format!("{} req", requests)
                } else {
                    Self::format_tokens(*tokens)
                },
                width = max_label_width,
                bar_width = max_width
            );
        }
    }

    pub fn render_time_series(
        title: &str,
        data: &[(String, u64, u64)],
        value_type: &str,
        max_width: usize,
        max_items: usize,
    ) {
        if data.is_empty() {
            println!("{} No data available", "ℹ️".blue());
            return;
        }

        println!("\n{}", title.bold().blue());
        
        let display_data: Vec<_> = data.iter().rev().take(max_items).rev().collect();
        let max_value = display_data
            .iter()
            .map(|(_, requests, tokens)| {
                if value_type == "tokens" { *tokens } else { *requests }
            })
            .max()
            .unwrap_or(1);

        let max_label_width = display_data
            .iter()
            .map(|(label, _, _)| label.len())
            .max()
            .unwrap_or(10);

        for (label, requests, tokens) in display_data {
            let value = if value_type == "tokens" { *tokens } else { *requests };
            let bar_width = if max_value > 0 {
                ((value as f64 / max_value as f64) * max_width as f64) as usize
            } else {
                0
            };

            let bar = "▓".repeat(bar_width);
            let formatted_value = if value_type == "tokens" {
                Self::format_tokens(*tokens)
            } else {
                format!("{}", requests)
            };

            println!(
                "  {:width$} │{:bar_width$} {} ({})",
                label.bold(),
                bar.cyan(),
                formatted_value.yellow(),
                if value_type == "tokens" {
                    format!("{} req", requests)
                } else {
                    Self::format_tokens(*tokens)
                },
                width = max_label_width,
                bar_width = max_width
            );
        }
    }

    fn format_tokens(tokens: u64) -> String {
        if tokens >= 1_000_000 {
            format!("{:.1}M", tokens as f64 / 1_000_000.0)
        } else if tokens >= 1_000 {
            format!("{:.1}k", tokens as f64 / 1_000.0)
        } else {
            format!("{}", tokens)
        }
    }
}

pub fn display_usage_overview(stats: &UsageStats) {
    println!("\n{}", "📊 Usage Overview".bold().blue());
    println!();

    // Basic stats
    println!("{} {}", "Total Requests:".bold(), stats.total_requests.to_string().green());
    println!("{} {}", "Total Tokens:".bold(), BarChart::format_tokens(stats.total_tokens).green());
    println!("{} {}", "Input Tokens:".bold(), BarChart::format_tokens(stats.input_tokens).cyan());
    println!("{} {}", "Output Tokens:".bold(), BarChart::format_tokens(stats.output_tokens).yellow());

    if let Some((earliest, latest)) = stats.date_range {
        let duration = latest.signed_duration_since(earliest);
        println!(
            "{} {} to {} ({} days)",
            "Date Range:".bold(),
            earliest.format("%Y-%m-%d").to_string().dimmed(),
            latest.format("%Y-%m-%d").to_string().dimmed(),
            duration.num_days().max(1)
        );
    }

    // Average tokens per request
    if stats.total_requests > 0 {
        let avg_tokens = stats.total_tokens / stats.total_requests;
        let avg_input = stats.input_tokens / stats.total_requests;
        let avg_output = stats.output_tokens / stats.total_requests;
        println!();
        println!("{}", "📈 Averages per Request".bold().blue());
        println!("{} {}", "Total Tokens:".bold(), BarChart::format_tokens(avg_tokens).green());
        println!("{} {}", "Input Tokens:".bold(), BarChart::format_tokens(avg_input).cyan());
        println!("{} {}", "Output Tokens:".bold(), BarChart::format_tokens(avg_output).yellow());
    }
}
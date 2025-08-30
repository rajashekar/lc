//! Usage statistics commands

use crate::analytics::usage_stats::{display_usage_overview, BarChart, UsageAnalyzer};
use crate::cli::UsageCommands;
use anyhow::Result;
use colored::Colorize;

/// Handle usage-related commands
pub async fn handle(
    command: Option<UsageCommands>,
    days: Option<u64>,
    tokens_only: bool,
    requests_only: bool,
    limit: Option<usize>,
) -> Result<()> {
    // Convert types to match what the analytics module expects
    let days_u32 = days.map(|d| d as u32);
    let limit_val = limit.unwrap_or(10);

    let analyzer = UsageAnalyzer::new()?;
    let stats = analyzer.get_usage_stats(days_u32)?;

    if stats.total_requests == 0 {
        println!("{} No usage data found", "ℹ️".blue());
        if days.is_some() {
            println!("Try expanding the time range or check if you have any logged interactions.");
        }
        return Ok(());
    }

    match command {
        Some(UsageCommands::Daily { count }) => {
            let value_type = determine_value_type(tokens_only, requests_only);

            BarChart::render_time_series(
                "📅 Daily Usage",
                &stats.daily_usage,
                value_type,
                50,
                count.min(limit_val),
            );
        }
        Some(UsageCommands::Weekly { count }) => {
            let value_type = determine_value_type(tokens_only, requests_only);

            BarChart::render_time_series(
                "📊 Weekly Usage",
                &stats.weekly_usage,
                value_type,
                50,
                count.min(limit_val),
            );
        }
        Some(UsageCommands::Monthly { count }) => {
            let value_type = determine_value_type(tokens_only, requests_only);

            BarChart::render_time_series(
                "📈 Monthly Usage",
                &stats.monthly_usage,
                value_type,
                50,
                count.min(limit_val),
            );
        }
        Some(UsageCommands::Yearly { count }) => {
            let value_type = determine_value_type(tokens_only, requests_only);

            BarChart::render_time_series(
                "📊 Yearly Usage",
                &stats.yearly_usage,
                value_type,
                50,
                count.min(limit_val),
            );
        }
        Some(UsageCommands::Models { count }) => {
            let value_type = determine_value_type(tokens_only, requests_only);

            BarChart::render_horizontal(
                "🤖 Top Models by Usage",
                &stats.model_usage,
                value_type,
                50,
                count.min(limit_val),
            );
        }
        None => {
            // Default: show overview and top charts
            display_usage_overview(&stats);

            if !tokens_only && !requests_only {
                // Show both tokens and requests by default
                BarChart::render_horizontal(
                    "🤖 Top Models by Token Usage",
                    &stats.model_usage,
                    "tokens",
                    50,
                    limit_val.min(5),
                );

                BarChart::render_time_series(
                    "📅 Recent Daily Usage (Tokens)",
                    &stats.daily_usage,
                    "tokens",
                    50,
                    limit_val.min(14),
                );
            } else if tokens_only {
                BarChart::render_horizontal(
                    "🤖 Top Models by Token Usage",
                    &stats.model_usage,
                    "tokens",
                    50,
                    limit_val.min(10),
                );

                BarChart::render_time_series(
                    "📅 Recent Daily Token Usage",
                    &stats.daily_usage,
                    "tokens",
                    50,
                    limit_val.min(14),
                );
            } else if requests_only {
                BarChart::render_horizontal(
                    "🤖 Top Models by Request Count",
                    &stats.model_usage,
                    "requests",
                    50,
                    limit_val.min(10),
                );

                BarChart::render_time_series(
                    "📅 Recent Daily Request Count",
                    &stats.daily_usage,
                    "requests",
                    50,
                    limit_val.min(14),
                );
            }
        }
    }

    Ok(())
}

/// Determine which value type to display based on flags
fn determine_value_type(tokens_only: bool, requests_only: bool) -> &'static str {
    if tokens_only {
        "tokens"
    } else if requests_only {
        "requests"
    } else {
        "tokens" // Default to tokens when neither flag is set
    }
}

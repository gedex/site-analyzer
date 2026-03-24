use std::process;

use anyhow::Context;
use clap::Parser;
use colored::Colorize;
use site_analyzer::{Analyzer, AnalyzerConfig};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// aw — Analyze Website
///
/// Detects technologies used by a website: CMS, JavaScript libraries,
/// web servers, CDN, analytics tools, and much more.
#[derive(Debug, Parser)]
#[command(name = "aw", version, about, long_about = None)]
#[command(arg_required_else_help = true)]
struct Args {
    /// URL to analyze (e.g. https://example.com or just example.com)
    url: String,

    /// Output as JSON
    #[arg(short, long)]
    json: bool,

    /// Pretty-print JSON output
    #[arg(short, long, default_value_t = true)]
    pretty: bool,

    /// Skip DNS resolution
    #[arg(long)]
    no_dns: bool,

    /// Skip TLS certificate analysis
    #[arg(long)]
    no_tls: bool,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Set up tracing
    let filter = if args.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("warn")
    };
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(filter)
        .init();

    if let Err(e) = run(args).await {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}

async fn run(args: Args) -> anyhow::Result<()> {
    let config = AnalyzerConfig {
        resolve_dns: !args.no_dns,
        fetch_tls: !args.no_tls,
        timeout_secs: 15,
        min_confidence: 50,
    };

    let analyzer = Analyzer::with_config(config).context("Failed to create analyzer")?;

    if !args.json {
        eprint!("{}", "Analyzing ".dimmed());
        eprint!("{}", args.url.cyan().bold());
        eprintln!("{}", " ...".dimmed());
    }

    let result = analyzer
        .analyze(&args.url)
        .await
        .context("Analysis failed")?;

    if args.json {
        // JSON output
        let json = if args.pretty {
            serde_json::to_string_pretty(&result)?
        } else {
            serde_json::to_string(&result)?
        };
        println!("{}", json);
    } else {
        // Human-readable output
        print_human_readable(&result);
    }

    Ok(())
}

fn print_human_readable(result: &site_analyzer::SiteAnalysis) {
    println!();
    println!(
        "{}  {}",
        "🌐".bold(),
        result.domain.cyan().bold()
    );
    println!("{}", "─".repeat(60).dimmed());

    if result.categories.is_empty() {
        println!("{}", "No technologies detected.".yellow());
        return;
    }

    // Sort categories for consistent output
    let mut cats: Vec<(&String, &Vec<site_analyzer::Technology>)> =
        result.categories.iter().collect();
    cats.sort_by_key(|(k, _)| k.as_str());

    for (category, technologies) in &cats {
        println!();
        println!("{}", category.bold().underline());
        for tech in *technologies {
            let mut line = format!("  {} {}", "▸".green(), tech.technology.white());
            if let Some(v) = &tech.version {
                line.push_str(&format!(" {}", format!("v{}", v).dimmed().to_string()));
            }
            if let Some(loc) = &tech.location {
                line.push_str(&format!(" {}", format!("({})", loc).dimmed().to_string()));
            }
            println!("{}", line);
        }
    }

    println!();
    println!(
        "{} {} {}",
        "─".repeat(60).dimmed(),
        format!("{} categories", result.categories.len()).dimmed(),
        format!(
            "· {} technologies",
            result.categories.values().map(|v| v.len()).sum::<usize>()
        )
        .dimmed()
    );
    println!();
}

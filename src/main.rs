use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::io::{self, Write};
use std::time::Duration;
use tracing::info;

mod analysis;
mod errors;
mod metrics;
mod parse;
mod patterns;
mod rpc;
mod view;

use errors::ScoutError;

#[derive(Copy, Clone, Eq, PartialEq, ValueEnum, Debug)]
enum Cluster {
    Mainnet,
    Devnet,
    Testnet,
}

impl Cluster {
    fn url(self) -> &'static str {
        match self {
            Cluster::Mainnet => "https://api.mainnet-beta.solana.com",
            Cluster::Devnet => "https://api.devnet.solana.com",
            Cluster::Testnet => "https://api.testnet.solana.com",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Cluster::Mainnet => "mainnet",
            Cluster::Devnet => "devnet",
            Cluster::Testnet => "testnet",
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "wallet-scout-b")]
#[command(about = "A zero-copy CLI wallet scout for Solana")]
#[command(version)]
struct Args {
    /// Wallet address (base58). If omitted, you'll be prompted.
    #[arg(long)]
    address: Option<String>,

    /// Cluster selection (default: devnet). If omitted, you'll be prompted.
    #[arg(long, value_enum)]
    cluster: Option<Cluster>,

    /// Output JSON instead of a table
    #[arg(long)]
    json: bool,

    /// Show debug information with raw offsets and data
    #[arg(long)]
    debug: bool,

    /// Run wallet analysis and show insights
    #[arg(long)]
    analyze: bool,

    /// Show only high-risk wallets
    #[arg(long)]
    risk_level: Option<String>,

    /// Custom RPC URL (overrides cluster selection)
    #[arg(long)]
    rpc_url: Option<String>,

    /// Global timeout in seconds for the entire operation
    #[arg(long, default_value = "60")]
    global_timeout: u64,

    /// Write metrics to file
    #[arg(long)]
    metrics_file: Option<String>,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Show summary dashboard (default behavior)
    #[arg(long)]
    summary: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(&args.log_level)
        .init();

    info!(
        "Starting wallet scout with global timeout: {}s",
        args.global_timeout
    );

    // Set up global timeout
    let global_timeout = Duration::from_secs(args.global_timeout);
    let timeout_result =
        tokio::time::timeout(global_timeout, async { run_wallet_scout(args).await }).await;

    match timeout_result {
        Ok(result) => result,
        Err(_) => {
            eprintln!(
                "Operation timed out after {} seconds",
                global_timeout.as_secs()
            );
            std::process::exit(124); // Standard timeout exit code
        }
    }
}

async fn run_wallet_scout(args: Args) -> Result<()> {
    let mut metrics = metrics::MetricsCollector::new();

    // Prompt for address if missing
    let address = match args.address {
        Some(a) => a,
        None => {
            print!("Enter wallet address (base58): ");
            io::stdout().flush().ok();
            let mut s = String::new();
            io::stdin().read_line(&mut s)?;
            s.trim().to_string()
        }
    };

    // Validate address early
    let _pk = bs58::decode(&address)
        .into_vec()
        .map_err(|_| ScoutError::InvalidPubkey(address.clone()))?;
    if _pk.len() != 32 {
        return Err(ScoutError::InvalidPubkey("pubkey must be 32 bytes".to_string()).into());
    }

    // Prompt for cluster if missing and no custom RPC
    let rpc_url = match args.rpc_url {
        Some(url) => url,
        None => {
            let cluster = match args.cluster {
                Some(c) => c,
                None => {
                    println!("Select network: [1] mainnet  [2] devnet  [3] testnet  (default: 2)");
                    print!("Choice: ");
                    io::stdout().flush().ok();
                    let mut s = String::new();
                    io::stdin().read_line(&mut s)?;
                    match s.trim() {
                        "1" => Cluster::Mainnet,
                        "3" => Cluster::Testnet,
                        _ => Cluster::Devnet,
                    }
                }
            };
            cluster.url().to_string()
        }
    };

    println!("Connecting to {}...", rpc_url);

    // Initialize RPC client
    let rpc = rpc::Rpc::new(&rpc_url)?;

    // 1) Get SOL balance
    println!("Fetching SOL balance...");
    let sol = rpc.get_balance(&address).await?;

    // 2) Get token accounts
    println!("Fetching SPL token accounts...");
    let token_accounts = rpc.get_token_accounts_by_owner_base64(&address).await?;

    // 3) Parse zero-copy views
    let mut parsed = Vec::with_capacity(token_accounts.len());
    for item in token_accounts {
        let parse_start = std::time::Instant::now();

        let bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &item.data_base64,
        )
        .map_err(|e| ScoutError::Decode {
            field: "base64_data".to_string(),
            expected: "valid base64".to_string(),
            got: format!("invalid base64: {}", e),
        })?;

        let view = parse::parse_spl_token_account(&bytes)?;

        // Record parse time
        metrics.record_parse_time(parse_start.elapsed());

        // Show debug info if requested
        if args.debug {
            view::print_debug(&view);
            println!();
        }

        parsed.push(view::TokenRow::from_view(item.pubkey, &view));
    }

    // 4) Analysis (if requested)
    if args.analyze {
        println!("Running wallet analysis...");
        let analyzer = analysis::WalletAnalyzer::new();
        let insights = analyzer.analyze(&parsed);

        if args.json {
            println!("{}", serde_json::to_string_pretty(&insights)?);
        } else {
            print_analysis(&insights)?;
        }
    } else if args.summary || !args.json {
        // 5) Summary dashboard output (default behavior)
        let analyzer = analysis::WalletAnalyzer::new();
        let insights = analyzer.analyze(&parsed);
        view::print_summary(sol, &parsed, &insights)?;
    } else {
        // 6) JSON output only when explicitly requested
        let output = view::WalletOut {
            schema_version: "1.0.0".to_string(),
            sol,
            total_tokens: parsed.len(),
            tokens: parsed,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    }

    // Write metrics if requested
    if let Some(metrics_file) = args.metrics_file {
        let final_metrics = metrics.finalize();
        let metrics_json = serde_json::to_string_pretty(&final_metrics)?;
        std::fs::write(&metrics_file, metrics_json)?;
        info!("Metrics written to {}", metrics_file);
    }

    Ok(())
}

/// Print wallet analysis in human-readable format
fn print_analysis(insights: &analysis::WalletInsights) -> anyhow::Result<()> {
    println!("┌─────────────────────────────────────────────────────────────────────────────────┐");
    println!("│ Wallet Analysis Report                                                          │");
    println!("└─────────────────────────────────────────────────────────────────────────────────┘");
    println!();

    // Wallet type and risk level
    println!("🔍 Wallet Type: {:?}", insights.wallet_type);
    println!("⚠️  Risk Level: {:?}", insights.risk_level);
    println!();

    // Statistics
    println!("📊 Statistics:");
    println!("  • Total Accounts: {}", insights.statistics.total_accounts);
    println!("  • Total Amount: {}", insights.statistics.total_amount);
    println!("  • Unique Mints: {}", insights.statistics.unique_mints);
    println!("  • Max Amount: {}", insights.statistics.max_amount);
    println!("  • Avg Amount: {}", insights.statistics.avg_amount);
    println!("  • Amount Variance: {:.2}", insights.statistics.variance);
    println!();

    // Account distribution
    println!("📈 Account Distribution:");
    println!(
        "  • Empty: {} ({:.1}%)",
        insights.statistics.empty_accounts,
        (insights.statistics.empty_accounts as f64 / insights.statistics.total_accounts as f64)
            * 100.0
    );
    println!(
        "  • Small: {} ({:.1}%)",
        insights.statistics.small_accounts,
        (insights.statistics.small_accounts as f64 / insights.statistics.total_accounts as f64)
            * 100.0
    );
    println!(
        "  • Medium: {} ({:.1}%)",
        insights.statistics.medium_accounts,
        (insights.statistics.medium_accounts as f64 / insights.statistics.total_accounts as f64)
            * 100.0
    );
    println!(
        "  • Large: {} ({:.1}%)",
        insights.statistics.large_accounts,
        (insights.statistics.large_accounts as f64 / insights.statistics.total_accounts as f64)
            * 100.0
    );
    println!();

    // Detected patterns
    if !insights.patterns.is_empty() {
        println!("🎯 Detected Patterns:");
        for pattern in &insights.patterns {
            println!(
                "  • {} (confidence: {:.1}%, risk: {:?})",
                pattern.name,
                pattern.confidence * 100.0,
                pattern.risk_level
            );
            for evidence in &pattern.evidence {
                println!("    - {}", evidence);
            }
        }
        println!();
    }

    // Summary
    println!("📝 Summary:");
    println!("  {}", insights.summary);
    println!();

    // Recommendations
    if !insights.recommendations.is_empty() {
        println!("💡 Recommendations:");
        for recommendation in &insights.recommendations {
            println!("  {}", recommendation);
        }
        println!();
    }

    Ok(())
}

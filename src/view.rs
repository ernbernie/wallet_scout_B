use serde::Serialize;
use crate::parse::TokenAccountView;
use proptest::prelude::*;

/// Display model for a token account row
/// Only allocate here - convert from zero-copy views to owned strings
#[derive(Serialize, Debug)]
pub struct TokenRow {
    pub account: String,
    pub mint: String,
    pub owner: String,
    pub amount_raw: u64,
    pub state: u8,
    pub delegated_amount: u64,
    pub delegate: Option<String>,
    pub close_authority: Option<String>,
}

impl TokenRow {
    pub fn from_view(account: String, view: &TokenAccountView<'_>) -> Self {
        Self {
            account,
            mint: bs58::encode(view.mint).into_string(),
            owner: bs58::encode(view.owner).into_string(),
            amount_raw: view.amount,
            state: view.state,
            delegated_amount: view.delegated_amount,
            delegate: view.delegate.map(|d| bs58::encode(d).into_string()),
            close_authority: view.close_authority.map(|ca| bs58::encode(ca).into_string()),
        }
    }
}

/// Complete wallet output for JSON serialization
#[derive(Serialize, Debug)]
pub struct WalletOut {
    pub schema_version: String,
    pub sol: f64,
    pub tokens: Vec<TokenRow>,
    pub total_tokens: usize,
}


impl Arbitrary for TokenRow {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            any::<String>(),
            any::<String>(),
            any::<String>(),
            any::<u64>(),
            any::<u8>(),
            any::<u64>(),
            any::<Option<String>>(),
            any::<Option<String>>(),
        )
            .prop_map(|(account, mint, owner, amount_raw, state, delegated_amount, delegate, close_authority)| {
                TokenRow {
                    account,
                    mint,
                    owner,
                    amount_raw,
                    state,
                    delegated_amount,
                    delegate,
                    close_authority,
                }
            })
            .boxed()
    }
}

/// Print summary dashboard with executive summary and key statistics
pub fn print_summary(sol: f64, _tokens: &[TokenRow], insights: &crate::analysis::WalletInsights) -> anyhow::Result<()> {
    println!("┌─────────────────────────────────────────────────────────────────────────────────┐");
    println!("│ Wallet Scout - Summary Dashboard                                                │");
    println!("└─────────────────────────────────────────────────────────────────────────────────┘");
    println!();
    
    // Executive Summary
    println!("🔍 Executive Summary");
    println!("{}", insights.summary);
    println!();
    
    // Key Statistics Table
    println!("📊 Key Statistics");
    println!("┌─────────────────────────────────────────────────────────────────────────────────┐");
    println!("│ Total Accounts: {:<8} │ Unique Mints: {:<8} │ Total Value: {:<15} │", 
        insights.statistics.total_accounts, 
        insights.statistics.unique_mints,
        format_number(insights.statistics.total_amount));
    println!("│ SOL Balance: {:<12} │ Max Account: {:<15} │ Avg Account: {:<15} │", 
        format_sol(sol),
        format_number(insights.statistics.max_amount),
        format_number(insights.statistics.avg_amount));
    println!("│ Wallet Type: {:<12} │ Risk Level: {:<12} │ Empty Accounts: {:<8} │", 
        format_wallet_type(&insights.wallet_type),
        format_risk_level(&insights.risk_level),
        insights.statistics.empty_accounts);
    println!("└─────────────────────────────────────────────────────────────────────────────────┘");
    println!();
    
    // Risk Assessment
    println!("⚠️  Risk Assessment: {} - {}", 
        format_risk_level(&insights.risk_level),
        get_risk_description(&insights.risk_level));
    
    // Recommendations
    if !insights.recommendations.is_empty() {
        println!("💡 Recommendation: {}", insights.recommendations[0]);
    }
    
    println!();
    
    Ok(())
}

/// Format large numbers with commas
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Format SOL balance
fn format_sol(sol: f64) -> String {
    if sol >= 1_000_000.0 {
        format!("{:.1}M SOL", sol / 1_000_000.0)
    } else if sol >= 1_000.0 {
        format!("{:.1}K SOL", sol / 1_000.0)
    } else {
        format!("{:.6} SOL", sol)
    }
}

/// Format wallet type for display
fn format_wallet_type(wallet_type: &crate::analysis::WalletType) -> String {
    match wallet_type {
        crate::analysis::WalletType::Personal => "Personal".to_string(),
        crate::analysis::WalletType::Distribution => "Distribution".to_string(),
        crate::analysis::WalletType::Airdrop => "Airdrop".to_string(),
        crate::analysis::WalletType::Exchange => "Exchange".to_string(),
        crate::analysis::WalletType::HighValue => "High Value".to_string(),
        crate::analysis::WalletType::Suspicious => "Suspicious".to_string(),
        crate::analysis::WalletType::Unknown => "Unknown".to_string(),
    }
}

/// Format risk level for display
fn format_risk_level(risk_level: &crate::analysis::RiskLevel) -> String {
    match risk_level {
        crate::analysis::RiskLevel::VeryLow => "Very Low".to_string(),
        crate::analysis::RiskLevel::Low => "Low".to_string(),
        crate::analysis::RiskLevel::Medium => "Medium".to_string(),
        crate::analysis::RiskLevel::High => "High".to_string(),
        crate::analysis::RiskLevel::Critical => "Critical".to_string(),
    }
}

/// Get risk description
fn get_risk_description(risk_level: &crate::analysis::RiskLevel) -> String {
    match risk_level {
        crate::analysis::RiskLevel::VeryLow => "Minimal risk indicators detected",
        crate::analysis::RiskLevel::Low => "Low risk indicators detected",
        crate::analysis::RiskLevel::Medium => "Moderate risk indicators detected",
        crate::analysis::RiskLevel::High => "High risk indicators detected",
        crate::analysis::RiskLevel::Critical => "Critical risk indicators detected",
    }.to_string()
}

/// Print debug information showing offsets and raw data
pub fn print_debug(view: &TokenAccountView<'_>) {
    println!("┌─────────────────────────────────────────────────────────────────────────────────┐");
    println!("│ Debug: SPL Token Account Raw Data                                             │");
    println!("└─────────────────────────────────────────────────────────────────────────────────┘");
    println!();
    
    println!("Raw data length: {} bytes", view.raw.len());
    println!();
    
    println!("Field Offsets:");
    println!("  Mint (0..32):           {}", hex::encode(&view.raw[0..32.min(view.raw.len())]));
    println!("  Owner (32..64):         {}", hex::encode(&view.raw[32..64.min(view.raw.len())]));
    println!("  Amount (64..72):        {}", hex::encode(&view.raw[64..72.min(view.raw.len())]));
    
    if view.raw.len() > 72 {
        println!("  Delegate Option (72):   {:02x}", view.delegate_option);
    }
    
    if view.raw.len() > 105 {
        println!("  State (105):           {:02x}", view.state);
    }
    
    if view.raw.len() > 106 {
        println!("  Is Native Option (106): {:02x}", view.is_native_option);
    }
    
    if view.raw.len() > 115 {
        println!("  Delegated Amount (115..123): {}", hex::encode(&view.raw[115..123.min(view.raw.len())]));
    }
    
    if view.raw.len() > 123 {
        println!("  Close Authority Option (123): {:02x}", view.close_authority_option);
    }
    
    println!();
    println!("Parsed Values:");
    println!("  Amount: {}", view.amount);
    println!("  State: {}", view.state);
    println!("  Delegated Amount: {}", view.delegated_amount);
    
    if let Some(delegate) = view.delegate {
        println!("  Delegate: {}", bs58::encode(delegate).into_string());
    }
    
    if let Some(close_authority) = view.close_authority {
        println!("  Close Authority: {}", bs58::encode(close_authority).into_string());
    }
}

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

/// Print human-readable table output
pub fn print_table(sol: f64, rows: &[TokenRow]) -> anyhow::Result<()> {
    println!("┌─────────────────────────────────────────────────────────────────────────────────┐");
    println!("│ Wallet Scout - Solana Account Analysis                                        │");
    println!("└─────────────────────────────────────────────────────────────────────────────────┘");
    println!();
    
    println!("SOL Balance: {:.9} SOL", sol);
    println!();
    
    if rows.is_empty() {
        println!("No SPL token accounts found.");
        return Ok(());
    }
    
    println!("SPL Token Accounts ({} found):", rows.len());
    println!();
    
    // Print header
    println!("{:<44}  {:<44}  {:<44}  {:>16}  {:>16}", 
        "Account", "Mint", "Owner", "Amount (raw)", "Delegated");
    println!("{}", "─".repeat(150));
    
    // Print rows
    for row in rows {
        let _delegate_info = if let Some(ref delegate) = row.delegate {
            format!("{}", delegate)
        } else {
            "None".to_string()
        };
        
        println!("{:<44}  {:<44}  {:<44}  {:>16}  {:>16}", 
            row.account, 
            row.mint, 
            row.owner, 
            row.amount_raw,
            row.delegated_amount);
    }
    
    println!();
    println!("Total SPL token accounts: {}", rows.len());
    
    Ok(())
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

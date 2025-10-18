use wallet_scout_b::patterns::*;
use wallet_scout_b::view::TokenRow;
use wallet_scout_b::analysis::{Pattern, RiskLevel};

/// Test distribution wallet pattern detection
#[test]
fn test_distribution_wallet_pattern() {
    let pattern = DistributionWalletPattern;
    
    // Create a clear distribution wallet (100 accounts, same mint, varying amounts)
    let mut tokens = Vec::new();
    let mint = "6kbwsSY4hL6WVadLRLnWV2irkMN2AvFZVAS8McKJmAtJ";
    
    for i in 0..100 {
        tokens.push(TokenRow {
            account: format!("Account{}", i),
            mint: mint.to_string(),
            owner: "Owner123".to_string(),
            amount_raw: if i % 10 == 0 { 0 } else { (i as u64 + 1) * 2_000_000_000 },
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }
    
    let result = pattern.detect(&tokens);
    assert!(result.is_some(), "Should detect distribution wallet pattern");
    
    let pattern_match = result.unwrap();
    assert_eq!(pattern_match.risk_level, RiskLevel::Medium);
    assert!(pattern_match.confidence > 0.0);
    assert!(pattern_match.evidence.iter().any(|e| e.contains("100 accounts")));
}

/// Test that distribution pattern doesn't trigger for small wallets
#[test]
fn test_distribution_wallet_pattern_small_wallet() {
    let pattern = DistributionWalletPattern;
    
    // Create a small wallet (should not trigger distribution pattern)
    let mut tokens = Vec::new();
    let mint = "6kbwsSY4hL6WVadLRLnWV2irkMN2AvFZVAS8McKJmAtJ";
    
    for i in 0..10 {  // Only 10 accounts
        tokens.push(TokenRow {
            account: format!("Account{}", i),
            mint: mint.to_string(),
            owner: "Owner123".to_string(),
            amount_raw: 1_000_000,
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }
    
    let result = pattern.detect(&tokens);
    assert!(result.is_none(), "Should not detect distribution pattern for small wallet");
}

/// Test airdrop wallet pattern detection
#[test]
fn test_airdrop_wallet_pattern() {
    let pattern = AirdropWalletPattern;
    
    // Create a clear airdrop wallet (50 accounts, same mint, small amounts)
    let mut tokens = Vec::new();
    let mint = "AirdropMint123";
    
    for i in 0..50 {
        tokens.push(TokenRow {
            account: format!("AirdropAccount{}", i),
            mint: mint.to_string(),
            owner: "AirdropOwner".to_string(),
            amount_raw: if i % 5 == 0 { 1 } else { 100_000 }, // Small amounts
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }
    
    let result = pattern.detect(&tokens);
    assert!(result.is_some(), "Should detect airdrop wallet pattern");
    
    let pattern_match = result.unwrap();
    assert_eq!(pattern_match.risk_level, RiskLevel::Low);
    assert!(pattern_match.confidence > 0.5);
    assert!(pattern_match.evidence.iter().any(|e| e.contains("50 accounts")));
}

/// Test exchange wallet pattern detection
#[test]
fn test_exchange_wallet_pattern() {
    let pattern = ExchangeWalletPattern;
    
    // Create an exchange-like wallet (many different mints)
    let mut tokens = Vec::new();
    
    for i in 0..20 {
        tokens.push(TokenRow {
            account: format!("ExchangeAccount{}", i),
            mint: format!("Mint{}", i), // Different mint for each account
            owner: "ExchangeOwner".to_string(),
            amount_raw: 1_000_000,
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }
    
    let result = pattern.detect(&tokens);
    assert!(result.is_some(), "Should detect exchange wallet pattern");
    
    let pattern_match = result.unwrap();
    assert_eq!(pattern_match.risk_level, RiskLevel::Medium);
    assert!(pattern_match.confidence > 0.5);
    assert!(pattern_match.evidence.iter().any(|e| e.contains("20 unique mints")));
}

/// Test suspicious activity pattern detection
#[test]
fn test_suspicious_activity_pattern() {
    let pattern = SuspiciousActivityPattern;
    
    // Create a suspicious wallet (extremely high account count)
    let mut tokens = Vec::new();
    
    for i in 0..1500 {  // Very high account count
        tokens.push(TokenRow {
            account: format!("SuspiciousAccount{}", i),
            mint: "SuspiciousMint".to_string(),
            owner: "SuspiciousOwner".to_string(),
            amount_raw: 0, // Empty accounts
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }
    
    let result = pattern.detect(&tokens);
    assert!(result.is_some(), "Should detect suspicious activity pattern");
    
    let pattern_match = result.unwrap();
    assert_eq!(pattern_match.risk_level, RiskLevel::High);
    assert!(pattern_match.confidence > 0.5);
    assert!(pattern_match.evidence.iter().any(|e| e.contains("1500 accounts")));
}

/// Test high value wallet pattern detection
#[test]
fn test_high_value_wallet_pattern() {
    let pattern = HighValueWalletPattern;
    
    // Create a high-value wallet
    let mut tokens = Vec::new();
    
    for i in 0..5 {
        tokens.push(TokenRow {
            account: format!("HighValueAccount{}", i),
            mint: format!("HighValueMint{}", i),
            owner: "HighValueOwner".to_string(),
            amount_raw: 10_000_000_000_000, // Very high amount
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }
    
    let result = pattern.detect(&tokens);
    assert!(result.is_some(), "Should detect high value wallet pattern");
    
    let pattern_match = result.unwrap();
    assert_eq!(pattern_match.risk_level, RiskLevel::Medium);
    assert!(pattern_match.confidence > 0.5);
    assert!(pattern_match.evidence.iter().any(|e| e.contains("Total value")));
}

/// Test pattern detection with empty wallet
#[test]
fn test_pattern_detection_empty_wallet() {
    let patterns = vec![
        Box::new(DistributionWalletPattern) as Box<dyn Pattern>,
        Box::new(AirdropWalletPattern) as Box<dyn Pattern>,
        Box::new(ExchangeWalletPattern) as Box<dyn Pattern>,
        Box::new(SuspiciousActivityPattern) as Box<dyn Pattern>,
        Box::new(HighValueWalletPattern) as Box<dyn Pattern>,
    ];
    
    let empty_tokens = Vec::new();
    
    for pattern in patterns {
        let result = pattern.detect(&empty_tokens);
        assert!(result.is_none(), "Should not detect any patterns in empty wallet");
    }
}

/// Test pattern detection with single account
#[test]
fn test_pattern_detection_single_account() {
    let patterns = vec![
        Box::new(DistributionWalletPattern) as Box<dyn Pattern>,
        Box::new(AirdropWalletPattern) as Box<dyn Pattern>,
        Box::new(ExchangeWalletPattern) as Box<dyn Pattern>,
    ];
    
    let single_token = vec![TokenRow {
        account: "SingleAccount".to_string(),
        mint: "SingleMint".to_string(),
        owner: "SingleOwner".to_string(),
        amount_raw: 1_000_000,
        state: 0,
        delegated_amount: 0,
        delegate: None,
        close_authority: None,
    }];
    
    for pattern in patterns {
        let result = pattern.detect(&single_token);
        assert!(result.is_none(), "Should not detect patterns with single account");
    }
}

use wallet_scout_b::analysis::WalletAnalyzer;
use wallet_scout_b::view::TokenRow;

/// Test full analysis pipeline with distribution wallet
#[test]
fn test_analysis_pipeline_distribution_wallet() {
    let analyzer = WalletAnalyzer::new();
    
    // Create a distribution wallet (322 accounts like your real case)
    let mut tokens = Vec::new();
    let mint = "6kbwsSY4hL6WVadLRLnWV2irkMN2AvFZVAS8McKJmAtJ";
    
    for i in 0..322 {
        tokens.push(TokenRow {
            account: format!("DistributionAccount{}", i),
            mint: mint.to_string(),
            owner: "DistributionOwner".to_string(),
            amount_raw: if i % 10 == 0 { 0 } else { (i as u64 + 1) * 1_000_000 },
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }
    
    let insights = analyzer.analyze(&tokens);
    
    // Should classify as distribution wallet
    assert_eq!(insights.wallet_type, wallet_scout_b::analysis::WalletType::Distribution);
    assert!(insights.patterns.iter().any(|p| p.name == "Distribution Wallet"));
    
    // Check statistics
    assert_eq!(insights.statistics.total_accounts, 322);
    assert_eq!(insights.statistics.unique_mints, 1);
    assert!(insights.statistics.total_amount > 0);
    
    // Check summary contains expected information
    assert!(insights.summary.contains("Distribution wallet detected"));
    assert!(insights.summary.contains("322 accounts"));
    
    // Check recommendations
    assert!(!insights.recommendations.is_empty());
    assert!(insights.recommendations.iter().any(|r| r.contains("distribution wallet")));
}

/// Test full analysis pipeline with airdrop wallet
#[test]
fn test_analysis_pipeline_airdrop_wallet() {
    let analyzer = WalletAnalyzer::new();
    
    // Create an airdrop wallet
    let mut tokens = Vec::new();
    let mint = "AirdropMint123";
    
    for i in 0..100 {
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
    
    let insights = analyzer.analyze(&tokens);
    
    // Should classify as airdrop wallet
    assert_eq!(insights.wallet_type, wallet_scout_b::analysis::WalletType::Airdrop);
    assert!(insights.patterns.iter().any(|p| p.name == "Airdrop Wallet"));
    
    // Check statistics
    assert_eq!(insights.statistics.total_accounts, 100);
    assert_eq!(insights.statistics.unique_mints, 1);
    
    // Check summary
    assert!(insights.summary.contains("Airdrop wallet detected"));
    assert!(insights.summary.contains("100 accounts"));
}

/// Test full analysis pipeline with exchange wallet
#[test]
fn test_analysis_pipeline_exchange_wallet() {
    let analyzer = WalletAnalyzer::new();
    
    // Create an exchange-like wallet
    let mut tokens = Vec::new();
    
    for i in 0..50 {
        tokens.push(TokenRow {
            account: format!("ExchangeAccount{}", i),
            mint: format!("Mint{}", i % 10), // 10 different mints
            owner: "ExchangeOwner".to_string(),
            amount_raw: 1_000_000,
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }
    
    let insights = analyzer.analyze(&tokens);
    
    // Should classify as exchange wallet
    assert_eq!(insights.wallet_type, wallet_scout_b::analysis::WalletType::Exchange);
    assert!(insights.patterns.iter().any(|p| p.name == "Exchange Wallet"));
    
    // Check statistics
    assert_eq!(insights.statistics.total_accounts, 50);
    assert_eq!(insights.statistics.unique_mints, 10);
    
    // Check summary
    assert!(insights.summary.contains("Exchange-like wallet detected"));
}

/// Test full analysis pipeline with high-value wallet
#[test]
fn test_analysis_pipeline_high_value_wallet() {
    let analyzer = WalletAnalyzer::new();
    
    // Create a high-value wallet
    let mut tokens = Vec::new();
    
    for i in 0..10 {
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
    
    let insights = analyzer.analyze(&tokens);
    
    // Should classify as high-value wallet
    assert_eq!(insights.wallet_type, wallet_scout_b::analysis::WalletType::HighValue);
    assert!(insights.patterns.iter().any(|p| p.name == "High Value Wallet"));
    
    // Check statistics
    assert_eq!(insights.statistics.total_accounts, 10);
    assert_eq!(insights.statistics.unique_mints, 10);
    assert!(insights.statistics.total_amount > 100_000_000_000_000);
    
    // Check summary
    assert!(insights.summary.contains("High-value wallet detected"));
}

/// Test full analysis pipeline with suspicious wallet
#[test]
fn test_analysis_pipeline_suspicious_wallet() {
    let analyzer = WalletAnalyzer::new();
    
    // Create a suspicious wallet
    let mut tokens = Vec::new();
    
    for i in 0..2000 {  // Extremely high account count
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
    
    let insights = analyzer.analyze(&tokens);
    
    // Should classify as suspicious wallet
    assert_eq!(insights.wallet_type, wallet_scout_b::analysis::WalletType::Suspicious);
    assert!(insights.patterns.iter().any(|p| p.name == "Suspicious Activity"));
    
    // Check statistics
    assert_eq!(insights.statistics.total_accounts, 2000);
    assert_eq!(insights.statistics.unique_mints, 1);
    
    // Check summary
    assert!(insights.summary.contains("Suspicious activity detected"));
}

/// Test full analysis pipeline with personal wallet
#[test]
fn test_analysis_pipeline_personal_wallet() {
    let analyzer = WalletAnalyzer::new();
    
    // Create a personal wallet (few accounts, mixed mints)
    let mut tokens = Vec::new();
    let mints = vec!["Mint1", "Mint2", "Mint3"];
    
    for i in 0..5 {
        tokens.push(TokenRow {
            account: format!("PersonalAccount{}", i),
            mint: mints[i % mints.len()].to_string(),
            owner: "PersonalOwner".to_string(),
            amount_raw: 1_000_000,
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }
    
    let insights = analyzer.analyze(&tokens);
    
    // Should classify as personal wallet
    assert_eq!(insights.wallet_type, wallet_scout_b::analysis::WalletType::Personal);
    
    // Check statistics
    assert_eq!(insights.statistics.total_accounts, 5);
    assert_eq!(insights.statistics.unique_mints, 3);
    
    // Check summary
    assert!(insights.summary.contains("Personal wallet detected"));
}

/// Test analysis with empty wallet
#[test]
fn test_analysis_pipeline_empty_wallet() {
    let analyzer = WalletAnalyzer::new();
    let empty_tokens = Vec::new();
    
    let insights = analyzer.analyze(&empty_tokens);
    
    // Should handle empty wallet gracefully
    assert_eq!(insights.statistics.total_accounts, 0);
    assert_eq!(insights.statistics.total_amount, 0);
    assert_eq!(insights.statistics.unique_mints, 0);
    
    // Should not crash or panic
    assert!(!insights.summary.is_empty());
}

/// Test that analysis results are deterministic
#[test]
fn test_analysis_deterministic() {
    let analyzer = WalletAnalyzer::new();
    
    // Create test data
    let mut tokens = Vec::new();
    let mint = "TestMint123";
    
    for i in 0..100 {
        tokens.push(TokenRow {
            account: format!("TestAccount{}", i),
            mint: mint.to_string(),
            owner: "TestOwner".to_string(),
            amount_raw: (i as u64 + 1) * 1_000_000,
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }
    
    // Run analysis multiple times
    let insights1 = analyzer.analyze(&tokens);
    let insights2 = analyzer.analyze(&tokens);
    
    // Results should be identical
    assert_eq!(insights1.wallet_type, insights2.wallet_type);
    assert_eq!(insights1.risk_level, insights2.risk_level);
    assert_eq!(insights1.statistics.total_accounts, insights2.statistics.total_accounts);
    assert_eq!(insights1.statistics.total_amount, insights2.statistics.total_amount);
    assert_eq!(insights1.summary, insights2.summary);
}

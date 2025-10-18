use wallet_scout_b::analysis::WalletAnalyzer;
use wallet_scout_b::view::{print_summary, TokenRow};

/// Test summary output with distribution wallet
#[test]
fn test_summary_output_distribution_wallet() {
    let analyzer = WalletAnalyzer::new();

    // Create a distribution wallet (100 accounts, same mint, varying amounts)
    let mut tokens = Vec::new();
    let mint = "6kbwsSY4hL6WVadLRLnWV2irkMN2AvFZVAS8McKJmAtJ";

    for i in 0..100 {
        tokens.push(TokenRow {
            account: format!("Account{}", i),
            mint: mint.to_string(),
            owner: "Owner123".to_string(),
            amount_raw: if i % 10 == 0 {
                0
            } else {
                (i as u64 + 1) * 2_000_000_000
            },
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }

    let insights = analyzer.analyze(&tokens);
    let sol_balance = 1.5; // 1.5 SOL

    // Test that summary output doesn't panic
    let result = print_summary(sol_balance, &tokens, &insights);
    assert!(result.is_ok(), "Summary output should not panic");

    // Verify insights contain expected data
    assert_eq!(insights.statistics.total_accounts, 100);
    assert_eq!(insights.statistics.unique_mints, 1);
    assert!(!insights.summary.is_empty());
}

/// Test summary output with personal wallet
#[test]
fn test_summary_output_personal_wallet() {
    let analyzer = WalletAnalyzer::new();

    // Create a personal wallet (few accounts, different mints)
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
    let sol_balance = 0.5; // 0.5 SOL

    // Test that summary output doesn't panic
    let result = print_summary(sol_balance, &tokens, &insights);
    assert!(result.is_ok(), "Summary output should not panic");

    // Verify insights contain expected data
    assert_eq!(insights.statistics.total_accounts, 5);
    assert_eq!(insights.statistics.unique_mints, 3);
    assert!(!insights.summary.is_empty());
}

/// Test summary output with empty wallet
#[test]
fn test_summary_output_empty_wallet() {
    let analyzer = WalletAnalyzer::new();
    let empty_tokens = Vec::new();

    let insights = analyzer.analyze(&empty_tokens);
    let sol_balance = 0.0; // 0 SOL

    // Test that summary output doesn't panic
    let result = print_summary(sol_balance, &empty_tokens, &insights);
    assert!(result.is_ok(), "Summary output should not panic");

    // Verify insights contain expected data
    assert_eq!(insights.statistics.total_accounts, 0);
    assert_eq!(insights.statistics.unique_mints, 0);
    assert!(!insights.summary.is_empty());
}

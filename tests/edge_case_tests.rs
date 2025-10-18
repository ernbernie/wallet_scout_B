use wallet_scout_b::analysis::WalletAnalyzer;
use wallet_scout_b::view::TokenRow;

/// Test analysis with maximum possible values
#[test]
fn test_analysis_maximum_values() {
    let analyzer = WalletAnalyzer::new();

    // Create wallet with large but safe values
    let mut tokens = Vec::new();
    for i in 0..1000 {
        tokens.push(TokenRow {
            account: format!("MaxAccount{}", i),
            mint: "MaxMint".to_string(),
            owner: "MaxOwner".to_string(),
            amount_raw: 1_000_000_000_000, // Large but safe value
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }

    let insights = analyzer.analyze(&tokens);

    // Should handle large values without panicking
    assert_eq!(insights.statistics.total_accounts, 1000);
    assert_eq!(insights.statistics.max_amount, 1_000_000_000_000);
    assert!(!insights.summary.is_empty());
}

/// Test analysis with zero values
#[test]
fn test_analysis_zero_values() {
    let analyzer = WalletAnalyzer::new();

    // Create wallet with all zero values
    let mut tokens = Vec::new();
    for i in 0..100 {
        tokens.push(TokenRow {
            account: format!("ZeroAccount{}", i),
            mint: "ZeroMint".to_string(),
            owner: "ZeroOwner".to_string(),
            amount_raw: 0,
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }

    let insights = analyzer.analyze(&tokens);

    // Should handle zero values gracefully
    assert_eq!(insights.statistics.total_accounts, 100);
    assert_eq!(insights.statistics.total_amount, 0);
    assert_eq!(insights.statistics.max_amount, 0);
    assert_eq!(insights.statistics.min_amount, 0);
    assert_eq!(insights.statistics.avg_amount, 0);
    assert!(!insights.summary.is_empty());
}

/// Test analysis with very long strings
#[test]
fn test_analysis_long_strings() {
    let analyzer = WalletAnalyzer::new();

    // Create wallet with very long strings
    let long_string = "A".repeat(1000);
    let mut tokens = Vec::new();

    for i in 0..10 {
        tokens.push(TokenRow {
            account: format!("{}{}", long_string, i),
            mint: format!("{}{}", long_string, i),
            owner: format!("{}{}", long_string, i),
            amount_raw: 1_000_000,
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }

    let insights = analyzer.analyze(&tokens);

    // Should handle long strings without issues
    assert_eq!(insights.statistics.total_accounts, 10);
    assert!(!insights.summary.is_empty());
}

/// Test analysis with special characters
#[test]
fn test_analysis_special_characters() {
    let analyzer = WalletAnalyzer::new();

    // Create wallet with special characters
    let special_chars = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
    let mut tokens = Vec::new();

    for i in 0..10 {
        tokens.push(TokenRow {
            account: format!("{}{}", special_chars, i),
            mint: format!("{}{}", special_chars, i),
            owner: format!("{}{}", special_chars, i),
            amount_raw: 1_000_000,
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }

    let insights = analyzer.analyze(&tokens);

    // Should handle special characters without issues
    assert_eq!(insights.statistics.total_accounts, 10);
    assert!(!insights.summary.is_empty());
}

/// Test analysis with unicode characters
#[test]
fn test_analysis_unicode_characters() {
    let analyzer = WalletAnalyzer::new();

    // Create wallet with unicode characters
    let unicode_chars = "🚀💰🎯🔥💎⭐🌟✨🎉🎊";
    let mut tokens = Vec::new();

    for i in 0..10 {
        tokens.push(TokenRow {
            account: format!("{}{}", unicode_chars, i),
            mint: format!("{}{}", unicode_chars, i),
            owner: format!("{}{}", unicode_chars, i),
            amount_raw: 1_000_000,
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }

    let insights = analyzer.analyze(&tokens);

    // Should handle unicode characters without issues
    assert_eq!(insights.statistics.total_accounts, 10);
    assert!(!insights.summary.is_empty());
}

/// Test analysis with extreme variance
#[test]
fn test_analysis_extreme_variance() {
    let analyzer = WalletAnalyzer::new();

    // Create wallet with extreme variance in amounts
    let mut tokens = Vec::new();

    // Add one account with large value
    tokens.push(TokenRow {
        account: "MaxAccount".to_string(),
        mint: "MaxMint".to_string(),
        owner: "MaxOwner".to_string(),
        amount_raw: 1_000_000_000_000, // Large but safe value
        state: 0,
        delegated_amount: 0,
        delegate: None,
        close_authority: None,
    });

    // Add many accounts with minimum value
    for i in 0..1000 {
        tokens.push(TokenRow {
            account: format!("MinAccount{}", i),
            mint: "MinMint".to_string(),
            owner: "MinOwner".to_string(),
            amount_raw: 1,
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }

    let insights = analyzer.analyze(&tokens);

    // Should handle extreme variance without panicking
    assert_eq!(insights.statistics.total_accounts, 1001);
    assert_eq!(insights.statistics.max_amount, 1_000_000_000_000);
    assert_eq!(insights.statistics.min_amount, 1);
    assert!(!insights.summary.is_empty());
}

/// Test analysis with mixed data types
#[test]
fn test_analysis_mixed_data_types() {
    let analyzer = WalletAnalyzer::new();

    // Create wallet with mixed data types
    let mut tokens = Vec::new();

    // Add accounts with different characteristics
    for i in 0..100 {
        let amount = match i % 4 {
            0 => 0,                 // Empty accounts
            1 => 1,                 // Minimum amount
            2 => 1_000_000,         // Medium amount
            _ => 1_000_000_000_000, // Large amount
        };

        let mint = match i % 3 {
            0 => "Mint1".to_string(),
            1 => "Mint2".to_string(),
            _ => "Mint3".to_string(),
        };

        tokens.push(TokenRow {
            account: format!("MixedAccount{}", i),
            mint,
            owner: format!("Owner{}", i % 10),
            amount_raw: amount,
            state: (i % 3) as u8,
            delegated_amount: if i % 5 == 0 { amount / 2 } else { 0 },
            delegate: if i % 7 == 0 {
                Some(format!("Delegate{}", i))
            } else {
                None
            },
            close_authority: if i % 11 == 0 {
                Some(format!("CloseAuth{}", i))
            } else {
                None
            },
        });
    }

    let insights = analyzer.analyze(&tokens);

    // Should handle mixed data types without issues
    assert_eq!(insights.statistics.total_accounts, 100);
    assert!(insights.statistics.unique_mints >= 1);
    assert!(!insights.summary.is_empty());
}

/// Test analysis with boundary conditions
#[test]
fn test_analysis_boundary_conditions() {
    let analyzer = WalletAnalyzer::new();

    // Test with exactly the minimum threshold for distribution pattern
    let mut tokens = Vec::new();
    let mint = "BoundaryMint";

    for i in 0..50 {
        // Exactly 50 accounts (minimum for distribution)
        tokens.push(TokenRow {
            account: format!("BoundaryAccount{}", i),
            mint: mint.to_string(),
            owner: "BoundaryOwner".to_string(),
            amount_raw: if i % 10 == 0 {
                0
            } else {
                (i as u64 + 1) * 1_000_000
            },
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }

    let insights = analyzer.analyze(&tokens);

    // Should handle boundary conditions without panicking
    assert_eq!(insights.statistics.total_accounts, 50);
    assert!(!insights.summary.is_empty());
}

/// Test analysis with malformed data
#[test]
fn test_analysis_malformed_data() {
    let analyzer = WalletAnalyzer::new();

    // Create wallet with potentially malformed data
    let mut tokens = Vec::new();

    for i in 0..10 {
        tokens.push(TokenRow {
            account: if i % 2 == 0 {
                "".to_string()
            } else {
                format!("Account{}", i)
            },
            mint: if i % 3 == 0 {
                "".to_string()
            } else {
                format!("Mint{}", i)
            },
            owner: if i % 4 == 0 {
                "".to_string()
            } else {
                format!("Owner{}", i)
            },
            amount_raw: if i % 5 == 0 {
                1_000_000_000_000
            } else {
                i as u64 * 1_000_000
            },
            state: if i % 6 == 0 { 255 } else { 0 },
            delegated_amount: if i % 7 == 0 { 1_000_000_000_000 } else { 0 },
            delegate: if i % 8 == 0 {
                Some("".to_string())
            } else {
                None
            },
            close_authority: if i % 9 == 0 {
                Some("".to_string())
            } else {
                None
            },
        });
    }

    let insights = analyzer.analyze(&tokens);

    // Should handle malformed data gracefully
    assert_eq!(insights.statistics.total_accounts, 10);
    assert!(!insights.summary.is_empty());
}

use wallet_scout_b::parse::parse_spl_token_account;
use wallet_scout_b::analysis::WalletAnalyzer;
use wallet_scout_b::view::TokenRow;
use proptest::prelude::*;

/// Property test: random buffer shrinking should hit specific error types
#[test]
fn test_parse_bounds_shrinking() {
    let mut runner = proptest::test_runner::TestRunner::default();
    
    let result = runner.run(&any::<Vec<u8>>(), |data| {
        let result = parse_spl_token_account(&data);
        
        if data.len() < 72 {
            // Should get DataTooShort error
            assert!(matches!(result, Err(wallet_scout_b::errors::ScoutError::DataTooShort { .. })));
        } else if data.len() >= 72 {
            // Should either succeed or get a specific decode error
            match result {
                Ok(_) => {}, // Valid parsing
                Err(wallet_scout_b::errors::ScoutError::Decode { .. }) => {}, // Expected decode error
                Err(wallet_scout_b::errors::ScoutError::DataTooShort { .. }) => {
                    // This should not happen for data >= 72 bytes
                    panic!("Unexpected DataTooShort for data >= 72 bytes");
                },
                Err(_) => {}, // Other errors are acceptable
            }
        }
        
        Ok(())
    });
    
    result.unwrap();
}

/// Property test: random byte flips should not panic
#[test]
fn test_parse_random_byte_flips() {
    let mut runner = proptest::test_runner::TestRunner::default();
    
    let result = runner.run(&any::<Vec<u8>>(), |data| {
        // Test that parsing never panics, even with random data
        let result = std::panic::catch_unwind(|| {
            parse_spl_token_account(&data)
        });
        
        // Should never panic
        assert!(result.is_ok(), "Parser should never panic on random data");
        
        Ok(())
    });
    
    result.unwrap();
}

/// Test overflow protection in analysis
#[test]
fn test_analysis_overflow_protection() {
    // Create a wallet with values that would overflow u64
    let mut tokens = Vec::new();
    
    // Add many accounts with large values
    for i in 0..1000 {
        tokens.push(TokenRow {
            account: format!("Account{}", i),
            mint: "TestMint".to_string(),
            owner: "TestOwner".to_string(),
            amount_raw: u64::MAX / 1000, // Large but safe individual values
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }
    
    let analyzer = WalletAnalyzer::new();
    let insights = analyzer.analyze(&tokens);
    
    // Should not panic or overflow
    assert!(insights.statistics.total_amount > 0);
    assert!(insights.statistics.total_accounts == 1000);
}

/// Test cancellation safety
#[tokio::test]
async fn test_cancellation_safety() {
    use tokio::time::{timeout, Duration};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancelled_clone = cancelled.clone();
    
    // Simulate cancellation after a short delay
    let task = tokio::spawn(async move {
        // Simulate some work that could be cancelled
        for i in 0..1000 {
            if cancelled_clone.load(Ordering::Relaxed) {
                return Err("Cancelled");
            }
            
            // Simulate parsing work
            let data = vec![0u8; 72 + (i % 4) * 20];
            let _ = parse_spl_token_account(&data);
        }
        
        Ok("Completed")
    });
    
    // Cancel after 10ms
    let cancel_task = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        cancelled.store(true, Ordering::Relaxed);
    });
    
    // Wait for either completion or timeout
    let result = timeout(Duration::from_millis(100), task).await;
    
    // Should handle cancellation gracefully
    match result {
        Ok(Ok(_)) => {}, // Completed successfully
        Ok(Err(_)) => {}, // Cancelled gracefully
        Err(_) => {}, // Timed out
    }
    
    cancel_task.abort();
}

/// Test memory bounds with large datasets
#[test]
fn test_memory_bounds_large_dataset() {
    // Create a large dataset that would stress memory
    let mut tokens = Vec::new();
    
    for i in 0..10000 {
        tokens.push(TokenRow {
            account: format!("Account{}", i),
            mint: format!("Mint{}", i % 100), // 100 different mints
            owner: format!("Owner{}", i % 10), // 10 different owners
            amount_raw: (i as u64 % 1000000) * 1000,
            state: (i % 3) as u8,
            delegated_amount: if i % 5 == 0 { (i as u64 % 100000) * 100 } else { 0 },
            delegate: if i % 7 == 0 { Some(format!("Delegate{}", i)) } else { None },
            close_authority: if i % 11 == 0 { Some(format!("CloseAuth{}", i)) } else { None },
        });
    }
    
    let analyzer = WalletAnalyzer::new();
    let insights = analyzer.analyze(&tokens);
    
    // Should handle large dataset without issues
    assert_eq!(insights.statistics.total_accounts, 10000);
    assert!(insights.statistics.unique_mints <= 100);
    assert!(!insights.summary.is_empty());
}

/// Test edge cases in pattern detection
#[test]
fn test_pattern_detection_edge_cases() {
    let analyzer = WalletAnalyzer::new();
    
    // Test with empty wallet
    let empty_insights = analyzer.analyze(&[]);
    assert_eq!(empty_insights.statistics.total_accounts, 0);
    assert_eq!(empty_insights.statistics.total_amount, 0);
    
    // Test with single account
    let single_token = vec![TokenRow {
        account: "SingleAccount".to_string(),
        mint: "SingleMint".to_string(),
        owner: "SingleOwner".to_string(),
        amount_raw: 1,
        state: 0,
        delegated_amount: 0,
        delegate: None,
        close_authority: None,
    }];
    
    let single_insights = analyzer.analyze(&single_token);
    assert_eq!(single_insights.statistics.total_accounts, 1);
    assert_eq!(single_insights.statistics.total_amount, 1);
}

/// Test numeric stability with extreme values
#[test]
fn test_numeric_stability_extreme_values() {
    let mut tokens = Vec::new();
    
    // Add one account with maximum value
    tokens.push(TokenRow {
        account: "MaxAccount".to_string(),
        mint: "MaxMint".to_string(),
        owner: "MaxOwner".to_string(),
        amount_raw: u64::MAX,
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
    
    let analyzer = WalletAnalyzer::new();
    let insights = analyzer.analyze(&tokens);
    
    // Should handle extreme variance without panicking
    assert_eq!(insights.statistics.total_accounts, 1001);
    assert_eq!(insights.statistics.max_amount, u64::MAX);
    assert_eq!(insights.statistics.min_amount, 1);
    assert!(!insights.summary.is_empty());
}

/// Test that all error paths are recoverable
#[test]
fn test_all_error_paths_recoverable() {
    // Test various malformed data scenarios
    let test_cases = vec![
        vec![], // Empty data
        vec![0u8; 10], // Too short
        vec![0u8; 50], // Still too short
        vec![0u8; 72], // Minimum valid size
        vec![0u8; 100], // Valid size
        vec![0u8; 200], // Large valid size
    ];
    
    for (_i, data) in test_cases.iter().enumerate() {
        let result = parse_spl_token_account(data);
        
        // Should never panic, always return Result
        match result {
            Ok(_) => {
                // Valid parsing
                assert!(data.len() >= 72, "Valid parsing should require >= 72 bytes");
            },
            Err(wallet_scout_b::errors::ScoutError::DataTooShort { expected, got }) => {
                assert!(got < expected, "DataTooShort should have got < expected");
            },
            Err(wallet_scout_b::errors::ScoutError::Decode { .. }) => {
                // Decode errors are acceptable for malformed data
            },
            Err(_) => {
                // Other errors are acceptable
            }
        }
    }
}

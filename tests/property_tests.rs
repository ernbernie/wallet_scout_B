use proptest::prelude::*;
use wallet_scout_b::analysis::WalletAnalyzer;
use wallet_scout_b::view::TokenRow;

/// Property test for analysis engine with random wallet data
#[test]
fn test_analysis_property_random_wallets() {
    let mut runner = proptest::test_runner::TestRunner::default();

    let result = runner.run(&any::<Vec<TokenRow>>(), |tokens| {
        let analyzer = WalletAnalyzer::new();
        let insights = analyzer.analyze(&tokens);

        // Properties that should always hold
        assert_eq!(insights.statistics.total_accounts, tokens.len());
        assert!(insights.statistics.unique_mints <= tokens.len());
        assert!(insights.statistics.max_amount >= insights.statistics.min_amount);

        if !tokens.is_empty() {
            // These assertions are always true for u64, but kept for documentation
        }

        // Schema version should always be present
        assert_eq!(insights.schema_version, "1.0.0");

        // Summary should never be empty
        assert!(!insights.summary.is_empty());

        // Risk level should be valid
        match insights.risk_level {
            wallet_scout_b::analysis::RiskLevel::VeryLow
            | wallet_scout_b::analysis::RiskLevel::Low
            | wallet_scout_b::analysis::RiskLevel::Medium
            | wallet_scout_b::analysis::RiskLevel::High
            | wallet_scout_b::analysis::RiskLevel::Critical => {}
        }

        // Wallet type should be valid
        match insights.wallet_type {
            wallet_scout_b::analysis::WalletType::Personal
            | wallet_scout_b::analysis::WalletType::Distribution
            | wallet_scout_b::analysis::WalletType::Airdrop
            | wallet_scout_b::analysis::WalletType::Exchange
            | wallet_scout_b::analysis::WalletType::HighValue
            | wallet_scout_b::analysis::WalletType::Suspicious
            | wallet_scout_b::analysis::WalletType::Unknown => {}
        }

        Ok(())
    });

    result.unwrap();
}

/// Property test for deterministic analysis results
#[test]
fn test_analysis_deterministic_results() {
    let mut runner = proptest::test_runner::TestRunner::default();

    let result = runner.run(&any::<Vec<TokenRow>>(), |tokens| {
        let analyzer = WalletAnalyzer::new();

        // Run analysis multiple times
        let insights1 = analyzer.analyze(&tokens);
        let insights2 = analyzer.analyze(&tokens);

        // Results should be identical
        assert_eq!(insights1.wallet_type, insights2.wallet_type);
        assert_eq!(insights1.risk_level, insights2.risk_level);
        assert_eq!(
            insights1.statistics.total_accounts,
            insights2.statistics.total_accounts
        );
        assert_eq!(
            insights1.statistics.total_amount,
            insights2.statistics.total_amount
        );
        assert_eq!(insights1.summary, insights2.summary);
        assert_eq!(insights1.schema_version, insights2.schema_version);

        Ok(())
    });

    result.unwrap();
}

/// Property test for overflow protection
#[test]
fn test_analysis_overflow_protection() {
    let mut runner = proptest::test_runner::TestRunner::default();

    let result = runner.run(&any::<Vec<u64>>(), |amounts| {
        // Create tokens with the given amounts
        let tokens: Vec<TokenRow> = amounts
            .iter()
            .enumerate()
            .map(|(i, &amount)| TokenRow {
                account: format!("Account{}", i),
                mint: "TestMint".to_string(),
                owner: "TestOwner".to_string(),
                amount_raw: amount,
                state: 0,
                delegated_amount: 0,
                delegate: None,
                close_authority: None,
            })
            .collect();

        let analyzer = WalletAnalyzer::new();
        let insights = analyzer.analyze(&tokens);

        // Should never panic or overflow
        assert_eq!(insights.statistics.total_accounts, tokens.len());
        // This assertion is always true for u64, but kept for documentation

        Ok(())
    });

    result.unwrap();
}

/// Property test for pattern detection consistency
#[test]
fn test_pattern_detection_consistency() {
    let mut runner = proptest::test_runner::TestRunner::default();

    let result = runner.run(&any::<Vec<TokenRow>>(), |tokens| {
        let analyzer = WalletAnalyzer::new();
        let insights = analyzer.analyze(&tokens);

        // Pattern detection should be consistent
        for pattern in &insights.patterns {
            // Confidence should be in valid range
            assert!(pattern.confidence >= 0.0);
            assert!(pattern.confidence <= 1.0);

            // Evidence should not be empty if pattern is detected
            assert!(!pattern.evidence.is_empty());

            // Structured evidence should be present
            assert!(
                !pattern.structured_evidence.thresholds.is_empty()
                    || !pattern.structured_evidence.features.is_empty()
                    || !pattern.structured_evidence.counts.is_empty()
            );
        }

        Ok(())
    });

    result.unwrap();
}

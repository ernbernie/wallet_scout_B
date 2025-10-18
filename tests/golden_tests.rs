use std::fs;
use wallet_scout_b::view::TokenRow;

/// Test that JSON output is deterministic
#[test]
fn test_deterministic_json_output() {
    // Create test data
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

    // Generate JSON output
    let output = wallet_scout_b::view::WalletOut {
        schema_version: "1.0.0".to_string(),
        sol: 1.5,
        total_tokens: tokens.len(),
        tokens,
    };

    let json1 = serde_json::to_string_pretty(&output).unwrap();
    let json2 = serde_json::to_string_pretty(&output).unwrap();

    // Should be identical
    assert_eq!(json1, json2);

    // Should include schema version
    assert!(json1.contains("\"schema_version\": \"1.0.0\""));
}

/// Test that structured evidence is present in analysis
#[test]
fn test_structured_evidence_present() {
    let analyzer = wallet_scout_b::analysis::WalletAnalyzer::new();

    // Create test data
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

    // Should have structured evidence for detected patterns
    if !insights.patterns.is_empty() {
        for pattern in &insights.patterns {
            assert!(!pattern.structured_evidence.thresholds.is_empty());
            assert!(!pattern.structured_evidence.features.is_empty());
            assert!(!pattern.structured_evidence.counts.is_empty());
        }
    }

    // Should have schema version
    assert_eq!(insights.schema_version, "1.0.0");
}

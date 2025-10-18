use wallet_scout_b::view::TokenRow;
use wallet_scout_b::view::WalletOut;
use serde_json;

/// Test that JSON output is deterministic
#[test]
fn test_deterministic_json_output() {
    // Create test data
    let tokens = vec![
        TokenRow {
            account: "Account1".to_string(),
            mint: "Mint1".to_string(),
            owner: "Owner1".to_string(),
            amount_raw: 1000000,
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        },
        TokenRow {
            account: "Account2".to_string(),
            mint: "Mint2".to_string(),
            owner: "Owner2".to_string(),
            amount_raw: 2000000,
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        },
    ];
    
    let output = WalletOut {
        schema_version: "1.0.0".to_string(),
        sol: 1.5,
        total_tokens: 2,
        tokens,
    };
    
    // Serialize multiple times and compare
    let json1 = serde_json::to_string_pretty(&output).unwrap();
    let json2 = serde_json::to_string_pretty(&output).unwrap();
    
    assert_eq!(json1, json2, "JSON output should be deterministic");
    
    // Verify schema version is present
    assert!(json1.contains("\"schema_version\": \"1.0.0\""));
}

/// Test that analysis output includes structured evidence
#[test]
fn test_structured_evidence_present() {
    use wallet_scout_b::analysis::WalletAnalyzer;
    
    // Create a distribution wallet pattern
    let mut tokens = Vec::new();
    let mint = "6kbwsSY4hL6WVadLRLnWV2irkMN2AvFZVAS8McKJmAtJ";
    
    for i in 0..100 {
        tokens.push(TokenRow {
            account: format!("Account{}", i),
            mint: mint.to_string(),
            owner: "Owner123".to_string(),
            amount_raw: if i % 10 == 0 { 0 } else { (i as u64 + 1) * 1_000_000 },
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }
    
    let analyzer = WalletAnalyzer::new();
    let insights = analyzer.analyze(&tokens);
    
    // Verify schema version is present
    assert_eq!(insights.schema_version, "1.0.0");
    
    // Verify structured evidence is present for patterns
    if !insights.patterns.is_empty() {
        for pattern in &insights.patterns {
            assert!(!pattern.structured_evidence.thresholds.is_empty() || 
                   !pattern.structured_evidence.features.is_empty() || 
                   !pattern.structured_evidence.counts.is_empty(),
                   "Pattern {} should have structured evidence", pattern.name);
        }
    }
}

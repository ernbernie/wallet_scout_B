use crate::view::TokenRow;
use crate::analysis::{Pattern, PatternMatch, RiskLevel, StructuredEvidence};
use std::collections::BTreeMap;

/// Detects distribution wallets (many accounts, same mint, varying amounts)
pub struct DistributionWalletPattern;

impl Pattern for DistributionWalletPattern {
    fn name(&self) -> &str {
        "Distribution Wallet"
    }
    
    fn detect(&self, tokens: &[TokenRow]) -> Option<PatternMatch> {
        if tokens.len() < 50 {
            return None;
        }
        
        // Check if all accounts have the same mint
        let first_mint = &tokens[0].mint;
        let same_mint = tokens.iter().all(|t| t.mint == *first_mint);
        
        if !same_mint {
            return None;
        }
        
        // Calculate amount variance with overflow protection
        let amounts: Vec<u64> = tokens.iter().map(|t| t.amount_raw).collect();
        let total_amount: u128 = amounts.iter().map(|&x| x as u128).sum();
        let avg_amount = (total_amount / tokens.len() as u128) as u64;
        
        let variance = if tokens.len() > 1 {
            let mean = avg_amount as f64;
            let sum_squared_diff: f64 = amounts.iter()
                .map(|&x| (x as f64 - mean).powi(2))
                .sum();
            sum_squared_diff / (tokens.len() - 1) as f64
        } else {
            0.0
        };
        
        // Check for varying amounts (high variance indicates distribution)
        let has_varying_amounts = variance > (avg_amount as f64 * 0.1);
        
        // Check for some large amounts (indicating distribution preparation)
        let has_large_amounts = amounts.iter().any(|&x| x > 1_000_000_000);
        
        if has_varying_amounts && has_large_amounts {
            let confidence = (tokens.len() as f64 / 1000.0).min(1.0);
            let evidence = vec![
                format!("{} accounts with same mint", tokens.len()),
                format!("High variance in amounts (variance: {:.2})", variance),
                format!("Contains large amounts (max: {})", amounts.iter().max().unwrap_or(&0)),
            ];
            
            let mut thresholds = BTreeMap::new();
            thresholds.insert("min_accounts".to_string(), 50.0);
            thresholds.insert("variance_threshold".to_string(), avg_amount as f64 * 0.1);
            thresholds.insert("large_amount_threshold".to_string(), 1_000_000_000.0);
            
            let mut features = BTreeMap::new();
            features.insert("account_count".to_string(), tokens.len() as f64);
            features.insert("variance".to_string(), variance);
            features.insert("avg_amount".to_string(), avg_amount as f64);
            features.insert("max_amount".to_string(), *amounts.iter().max().unwrap_or(&0) as f64);
            
            let mut counts = BTreeMap::new();
            counts.insert("total_accounts".to_string(), tokens.len());
            counts.insert("large_amount_accounts".to_string(), amounts.iter().filter(|&&x| x > 1_000_000_000).count());
            
            Some(PatternMatch {
                confidence,
                evidence,
                structured_evidence: StructuredEvidence {
                    thresholds,
                    features,
                    counts,
                },
                risk_level: RiskLevel::Medium,
            })
        } else {
            None
        }
    }
}

/// Detects airdrop wallets (many small accounts, same mint)
pub struct AirdropWalletPattern;

impl Pattern for AirdropWalletPattern {
    fn name(&self) -> &str {
        "Airdrop Wallet"
    }
    
    fn detect(&self, tokens: &[TokenRow]) -> Option<PatternMatch> {
        if tokens.len() < 20 {
            return None;
        }
        
        // Check if all accounts have the same mint
        let first_mint = &tokens[0].mint;
        let same_mint = tokens.iter().all(|t| t.mint == *first_mint);
        
        if !same_mint {
            return None;
        }
        
        // Check for small amounts (typical of airdrops)
        let small_amounts = tokens.iter().filter(|t| t.amount_raw < 100_000_000).count();
        let very_small_amounts = tokens.iter().filter(|t| t.amount_raw <= 1_000_000).count();
        let one_token_accounts = tokens.iter().filter(|t| t.amount_raw == 1).count();
        
        // Airdrop wallets typically have many small amounts
        let small_ratio = small_amounts as f64 / tokens.len() as f64;
        let very_small_ratio = very_small_amounts as f64 / tokens.len() as f64;
        
        if small_ratio > 0.7 || very_small_ratio > 0.5 || one_token_accounts > 10 {
            let confidence = (small_ratio * 0.5 + very_small_ratio * 0.3 + (one_token_accounts as f64 / tokens.len() as f64) * 0.2).min(1.0);
            let evidence = vec![
                format!("{} accounts with same mint", tokens.len()),
                format!("{}% small amounts (< 100M tokens)", (small_ratio * 100.0) as u32),
                format!("{} one-token accounts", one_token_accounts),
            ];
            
            let mut thresholds = BTreeMap::new();
            thresholds.insert("min_accounts".to_string(), 20.0);
            thresholds.insert("small_amount_threshold".to_string(), 100_000_000.0);
            thresholds.insert("very_small_threshold".to_string(), 1_000_000.0);
            thresholds.insert("one_token_threshold".to_string(), 10.0);
            
            let mut features = BTreeMap::new();
            features.insert("account_count".to_string(), tokens.len() as f64);
            features.insert("small_ratio".to_string(), small_ratio);
            features.insert("very_small_ratio".to_string(), very_small_ratio);
            
            let mut counts = BTreeMap::new();
            counts.insert("total_accounts".to_string(), tokens.len());
            counts.insert("small_amount_accounts".to_string(), small_amounts);
            counts.insert("very_small_accounts".to_string(), very_small_amounts);
            counts.insert("one_token_accounts".to_string(), one_token_accounts);
            
            Some(PatternMatch {
                confidence,
                evidence,
                structured_evidence: StructuredEvidence {
                    thresholds,
                    features,
                    counts,
                },
                risk_level: RiskLevel::Low,
            })
        } else {
            None
        }
    }
}

/// Detects exchange-like wallets (many different mints, high transaction volume)
pub struct ExchangeWalletPattern;

impl Pattern for ExchangeWalletPattern {
    fn name(&self) -> &str {
        "Exchange Wallet"
    }
    
    fn detect(&self, tokens: &[TokenRow]) -> Option<PatternMatch> {
        if tokens.len() < 10 {
            return None;
        }
        
        // Count unique mints
        let unique_mints: indexmap::IndexSet<_> = tokens.iter().map(|t| &t.mint).collect();
        let mint_diversity = unique_mints.len() as f64 / tokens.len() as f64;
        
        // Check for high mint diversity (exchanges have many different tokens)
        if mint_diversity > 0.3 && unique_mints.len() > 5 {
            let confidence = (mint_diversity * 0.7 + (unique_mints.len() as f64 / 100.0).min(0.3)).min(1.0);
            let evidence = vec![
                format!("{} unique mints out of {} accounts", unique_mints.len(), tokens.len()),
                format!("{:.1}% mint diversity", mint_diversity * 100.0),
            ];
            
            let mut thresholds = BTreeMap::new();
            thresholds.insert("min_accounts".to_string(), 10.0);
            thresholds.insert("mint_diversity_threshold".to_string(), 0.3);
            thresholds.insert("min_unique_mints".to_string(), 5.0);
            
            let mut features = BTreeMap::new();
            features.insert("account_count".to_string(), tokens.len() as f64);
            features.insert("unique_mint_count".to_string(), unique_mints.len() as f64);
            features.insert("mint_diversity".to_string(), mint_diversity);
            
            let mut counts = BTreeMap::new();
            counts.insert("total_accounts".to_string(), tokens.len());
            counts.insert("unique_mints".to_string(), unique_mints.len());
            
            Some(PatternMatch {
                confidence,
                evidence,
                structured_evidence: StructuredEvidence {
                    thresholds,
                    features,
                    counts,
                },
                risk_level: RiskLevel::Medium,
            })
        } else {
            None
        }
    }
}

/// Detects suspicious activity patterns
pub struct SuspiciousActivityPattern;

impl Pattern for SuspiciousActivityPattern {
    fn name(&self) -> &str {
        "Suspicious Activity"
    }
    
    fn detect(&self, tokens: &[TokenRow]) -> Option<PatternMatch> {
        let mut suspicious_indicators = Vec::new();
        
        // Check for extremely high account count
        if tokens.len() > 1000 {
            suspicious_indicators.push(format!("Extremely high account count: {}", tokens.len()));
        }
        
        // Check for extremely high total value with overflow protection
        let total_value: u128 = tokens.iter().map(|t| t.amount_raw as u128).sum();
        if total_value > 1_000_000_000_000_000 {
            suspicious_indicators.push(format!("Extremely high total value: {}", total_value));
        }
        
        // Check for unusual mint patterns (all same mint with very high count)
        if tokens.len() > 500 {
            let first_mint = &tokens[0].mint;
            let same_mint = tokens.iter().all(|t| t.mint == *first_mint);
            if same_mint {
                suspicious_indicators.push(format!("{} accounts with identical mint", tokens.len()));
            }
        }
        
        // Check for empty accounts (potential dust attack preparation)
        let empty_accounts = tokens.iter().filter(|t| t.amount_raw == 0).count();
        let empty_ratio = empty_accounts as f64 / tokens.len() as f64;
        if empty_ratio > 0.8 && tokens.len() > 100 {
            suspicious_indicators.push(format!("{}% empty accounts ({} out of {})", 
                (empty_ratio * 100.0) as u32, empty_accounts, tokens.len()));
        }
        
        if suspicious_indicators.len() >= 2 {
            let confidence = (suspicious_indicators.len() as f64 / 4.0).min(1.0);
            
            let mut thresholds = BTreeMap::new();
            thresholds.insert("max_account_count".to_string(), 1000.0);
            thresholds.insert("max_total_value".to_string(), 1_000_000_000_000_000.0);
            thresholds.insert("max_same_mint_accounts".to_string(), 500.0);
            thresholds.insert("max_empty_ratio".to_string(), 0.8);
            thresholds.insert("min_suspicious_indicators".to_string(), 2.0);
            
            let mut features = BTreeMap::new();
            features.insert("account_count".to_string(), tokens.len() as f64);
            features.insert("total_value".to_string(), total_value as f64);
            features.insert("empty_ratio".to_string(), empty_ratio);
            features.insert("suspicious_indicators".to_string(), suspicious_indicators.len() as f64);
            
            let mut counts = BTreeMap::new();
            counts.insert("total_accounts".to_string(), tokens.len());
            counts.insert("empty_accounts".to_string(), empty_accounts);
            counts.insert("suspicious_indicators".to_string(), suspicious_indicators.len());
            
            Some(PatternMatch {
                confidence,
                evidence: suspicious_indicators,
                structured_evidence: StructuredEvidence {
                    thresholds,
                    features,
                    counts,
                },
                risk_level: RiskLevel::High,
            })
        } else {
            None
        }
    }
}

/// Detects high-value wallets
pub struct HighValueWalletPattern;

impl Pattern for HighValueWalletPattern {
    fn name(&self) -> &str {
        "High Value Wallet"
    }
    
    fn detect(&self, tokens: &[TokenRow]) -> Option<PatternMatch> {
        let total_value: u128 = tokens.iter().map(|t| t.amount_raw as u128).sum();
        let max_amount = tokens.iter().map(|t| t.amount_raw).max().unwrap_or(0);
        
        // Check for high total value
        let high_total_value = total_value > 10_000_000_000_000; // 10 trillion tokens
        
        // Check for individual high-value accounts
        let high_individual_accounts = tokens.iter().filter(|t| t.amount_raw > 1_000_000_000_000).count();
        
        if high_total_value || high_individual_accounts > 0 {
            let confidence = if high_total_value { 0.9 } else { 0.7 };
            let evidence = vec![
                format!("Total value: {}", total_value),
                format!("Max individual amount: {}", max_amount),
                format!("High-value accounts: {}", high_individual_accounts),
            ];
            
            let mut thresholds = BTreeMap::new();
            thresholds.insert("high_total_value_threshold".to_string(), 10_000_000_000_000.0);
            thresholds.insert("high_individual_threshold".to_string(), 1_000_000_000_000.0);
            
            let mut features = BTreeMap::new();
            features.insert("total_value".to_string(), total_value as f64);
            features.insert("max_amount".to_string(), max_amount as f64);
            features.insert("high_individual_accounts".to_string(), high_individual_accounts as f64);
            
            let mut counts = BTreeMap::new();
            counts.insert("total_accounts".to_string(), tokens.len());
            counts.insert("high_value_accounts".to_string(), high_individual_accounts);
            
            Some(PatternMatch {
                confidence,
                evidence,
                structured_evidence: StructuredEvidence {
                    thresholds,
                    features,
                    counts,
                },
                risk_level: RiskLevel::Medium,
            })
        } else {
            None
        }
    }
}

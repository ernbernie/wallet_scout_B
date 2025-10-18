use crate::view::TokenRow;
use crate::patterns::{DistributionWalletPattern, AirdropWalletPattern, ExchangeWalletPattern, SuspiciousActivityPattern, HighValueWalletPattern};
use serde::Serialize;
use indexmap::IndexSet;

/// Core analysis framework for wallet pattern detection
pub struct WalletAnalyzer {
    patterns: Vec<Box<dyn Pattern>>,
}

impl WalletAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            patterns: Vec::new(),
        };
        
        // Register all pattern detectors
        analyzer.register_pattern(Box::new(DistributionWalletPattern));
        analyzer.register_pattern(Box::new(AirdropWalletPattern));
        analyzer.register_pattern(Box::new(ExchangeWalletPattern));
        analyzer.register_pattern(Box::new(SuspiciousActivityPattern));
        analyzer.register_pattern(Box::new(HighValueWalletPattern));
        
        analyzer
    }
    
    fn register_pattern(&mut self, pattern: Box<dyn Pattern>) {
        self.patterns.push(pattern);
    }
    
    /// Analyze a wallet and return comprehensive insights
    pub fn analyze(&self, tokens: &[TokenRow]) -> WalletInsights {
        let statistics = self.calculate_statistics(tokens);
        let detected_patterns = self.detect_patterns(tokens);
        let wallet_type = self.classify_wallet_type(&detected_patterns, &statistics);
        let risk_level = self.assess_risk_level(&detected_patterns, &statistics);
        let summary = self.generate_summary(&wallet_type, &detected_patterns, &statistics);
        let recommendations = self.generate_recommendations(&wallet_type, &risk_level, &detected_patterns);
        
        WalletInsights {
            schema_version: "1.0.0".to_string(),
            wallet_type,
            risk_level,
            patterns: detected_patterns,
            statistics,
            summary,
            recommendations,
        }
    }
    
    fn detect_patterns(&self, tokens: &[TokenRow]) -> Vec<DetectedPattern> {
        let mut detected = Vec::new();
        
        for pattern in &self.patterns {
            if let Some(match_result) = pattern.detect(tokens) {
                detected.push(DetectedPattern {
                    name: pattern.name().to_string(),
                    confidence: match_result.confidence,
                    evidence: match_result.evidence,
                    structured_evidence: match_result.structured_evidence,
                    risk_level: match_result.risk_level,
                });
            }
        }
        
        detected
    }
    
    fn calculate_statistics(&self, tokens: &[TokenRow]) -> WalletStatistics {
        if tokens.is_empty() {
            return WalletStatistics::default();
        }
        
        let total_accounts = tokens.len();
        // Use u128 to prevent overflow during aggregation
        let total_amount: u128 = tokens.iter().map(|t| t.amount_raw as u128).sum();
        let unique_mints = tokens.iter().map(|t| &t.mint).collect::<IndexSet<_>>().len();
        
        let amounts: Vec<u64> = tokens.iter().map(|t| t.amount_raw).collect();
        let max_amount = amounts.iter().max().copied().unwrap_or(0);
        let min_amount = amounts.iter().min().copied().unwrap_or(0);
        let avg_amount = if total_accounts > 0 { 
            (total_amount / total_accounts as u128) as u64 
        } else { 0 };
        
        // Calculate variance for amount distribution
        let variance = if total_accounts > 1 {
            let mean = avg_amount as f64;
            let sum_squared_diff: f64 = amounts.iter()
                .map(|&x| (x as f64 - mean).powi(2))
                .sum();
            sum_squared_diff / (total_accounts - 1) as f64
        } else {
            0.0
        };
        
        // Count accounts by amount ranges
        let empty_accounts = tokens.iter().filter(|t| t.amount_raw == 0).count();
        let small_accounts = tokens.iter().filter(|t| t.amount_raw > 0 && t.amount_raw < 1_000_000).count();
        let medium_accounts = tokens.iter().filter(|t| t.amount_raw >= 1_000_000 && t.amount_raw < 1_000_000_000).count();
        let large_accounts = tokens.iter().filter(|t| t.amount_raw >= 1_000_000_000).count();
        
        WalletStatistics {
            total_accounts,
            total_amount: total_amount as u64, // Safe downcast after u128 arithmetic
            unique_mints,
            max_amount,
            min_amount,
            avg_amount,
            variance,
            empty_accounts,
            small_accounts,
            medium_accounts,
            large_accounts,
        }
    }
    
    fn classify_wallet_type(&self, patterns: &[DetectedPattern], stats: &WalletStatistics) -> WalletType {
        // Check for specific patterns first
        if patterns.iter().any(|p| p.name == "Distribution Wallet") {
            return WalletType::Distribution;
        }
        if patterns.iter().any(|p| p.name == "Airdrop Wallet") {
            return WalletType::Airdrop;
        }
        if patterns.iter().any(|p| p.name == "Exchange Wallet") {
            return WalletType::Exchange;
        }
        if patterns.iter().any(|p| p.name == "Suspicious Activity") {
            return WalletType::Suspicious;
        }
        if patterns.iter().any(|p| p.name == "High Value Wallet") {
            return WalletType::HighValue;
        }
        
        // Fallback to statistical classification
        if stats.total_accounts > 100 {
            WalletType::Distribution
        } else if stats.unique_mints > 10 {
            WalletType::Exchange
        } else if stats.total_amount > 1_000_000_000_000 {
            WalletType::HighValue
        } else {
            WalletType::Personal
        }
    }
    
    fn assess_risk_level(&self, patterns: &[DetectedPattern], stats: &WalletStatistics) -> RiskLevel {
        // Check for high-risk patterns
        if patterns.iter().any(|p| p.risk_level == RiskLevel::Critical) {
            return RiskLevel::Critical;
        }
        
        if patterns.iter().any(|p| p.risk_level == RiskLevel::High) {
            return RiskLevel::High;
        }
        
        // Statistical risk assessment
        if stats.total_amount > 10_000_000_000_000 || stats.total_accounts > 1000 {
            RiskLevel::Medium
        } else if stats.total_accounts > 10 {
            RiskLevel::Low
        } else {
            RiskLevel::VeryLow
        }
    }
    
    fn generate_summary(&self, wallet_type: &WalletType, _patterns: &[DetectedPattern], stats: &WalletStatistics) -> String {
        match wallet_type {
            WalletType::Distribution => {
                format!("Distribution wallet detected: {} accounts holding {} total tokens across {} unique mints. This appears to be a token distribution or airdrop preparation wallet.", 
                    stats.total_accounts, stats.total_amount, stats.unique_mints)
            },
            WalletType::Airdrop => {
                format!("Airdrop wallet detected: {} accounts with small token amounts. This wallet appears to be preparing for or has completed an airdrop distribution.", 
                    stats.total_accounts)
            },
            WalletType::Exchange => {
                format!("Exchange-like wallet detected: {} accounts with {} unique token types. This wallet shows characteristics of an exchange or trading platform.", 
                    stats.total_accounts, stats.unique_mints)
            },
            WalletType::Suspicious => {
                format!("Suspicious activity detected: {} accounts with unusual patterns. This wallet shows characteristics that may indicate suspicious or automated activity.", 
                    stats.total_accounts)
            },
            WalletType::HighValue => {
                format!("High-value wallet detected: {} accounts with {} total tokens. This wallet contains significant token value.", 
                    stats.total_accounts, stats.total_amount)
            },
            WalletType::Personal => {
                format!("Personal wallet detected: {} accounts with {} unique token types. This appears to be a typical personal wallet.", 
                    stats.total_accounts, stats.unique_mints)
            },
            WalletType::Unknown => {
                format!("Unknown wallet type: {} accounts. Unable to classify this wallet based on available patterns.", 
                    stats.total_accounts)
            },
        }
    }
    
    fn generate_recommendations(&self, wallet_type: &WalletType, risk_level: &RiskLevel, patterns: &[DetectedPattern]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        match risk_level {
            RiskLevel::Critical => {
                recommendations.push("⚠️  CRITICAL RISK: This wallet shows critical risk indicators. Exercise extreme caution.".to_string());
            },
            RiskLevel::High => {
                recommendations.push("🔴 HIGH RISK: This wallet shows high-risk patterns. Monitor closely.".to_string());
            },
            RiskLevel::Medium => {
                recommendations.push("🟡 MEDIUM RISK: This wallet shows moderate risk indicators. Proceed with caution.".to_string());
            },
            _ => {}
        }
        
        match wallet_type {
            WalletType::Distribution => {
                recommendations.push("📊 This is a distribution wallet. Monitor for airdrop announcements or token distributions.".to_string());
            },
            WalletType::Airdrop => {
                recommendations.push("🎁 This appears to be an airdrop wallet. Check for recent airdrop announcements.".to_string());
            },
            WalletType::Exchange => {
                recommendations.push("🏦 This wallet shows exchange-like characteristics. Verify if this is a legitimate exchange.".to_string());
            },
            WalletType::Suspicious => {
                recommendations.push("🚨 Suspicious activity detected. Avoid interacting with this wallet.".to_string());
            },
            _ => {}
        }
        
        if patterns.iter().any(|p| p.name == "High Value Wallet") {
            recommendations.push("💰 High-value wallet detected. Consider security implications.".to_string());
        }
        
        recommendations
    }
}

/// Trait for pattern detection
pub trait Pattern {
    fn name(&self) -> &str;
    fn detect(&self, tokens: &[TokenRow]) -> Option<PatternMatch>;
}

/// Result of pattern detection
pub struct PatternMatch {
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub structured_evidence: StructuredEvidence,
    pub risk_level: RiskLevel,
}

/// Structured evidence for machine-readable analysis
#[derive(Debug, Clone, Serialize)]
pub struct StructuredEvidence {
    pub thresholds: std::collections::BTreeMap<String, f64>,
    pub features: std::collections::BTreeMap<String, f64>,
    pub counts: std::collections::BTreeMap<String, usize>,
}

/// Detected pattern with metadata
#[derive(Debug, Clone, Serialize)]
pub struct DetectedPattern {
    pub name: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub structured_evidence: StructuredEvidence,
    pub risk_level: RiskLevel,
}

/// Comprehensive wallet analysis results
#[derive(Debug, Clone, Serialize)]
pub struct WalletInsights {
    pub schema_version: String,
    pub wallet_type: WalletType,
    pub risk_level: RiskLevel,
    pub patterns: Vec<DetectedPattern>,
    pub statistics: WalletStatistics,
    pub summary: String,
    pub recommendations: Vec<String>,
}

/// Wallet type classification
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum WalletType {
    Personal,
    Distribution,
    Airdrop,
    Exchange,
    HighValue,
    Suspicious,
    Unknown,
}

/// Risk level assessment
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum RiskLevel {
    VeryLow,
    Low,
    Medium,
    High,
    Critical,
}

/// Statistical analysis of wallet
#[derive(Debug, Clone, Serialize)]
pub struct WalletStatistics {
    pub total_accounts: usize,
    pub total_amount: u64,
    pub unique_mints: usize,
    pub max_amount: u64,
    pub min_amount: u64,
    pub avg_amount: u64,
    pub variance: f64,
    pub empty_accounts: usize,
    pub small_accounts: usize,
    pub medium_accounts: usize,
    pub large_accounts: usize,
}

impl Default for WalletStatistics {
    fn default() -> Self {
        Self {
            total_accounts: 0,
            total_amount: 0,
            unique_mints: 0,
            max_amount: 0,
            min_amount: 0,
            avg_amount: 0,
            variance: 0.0,
            empty_accounts: 0,
            small_accounts: 0,
            medium_accounts: 0,
            large_accounts: 0,
        }
    }
}

# Wallet Analysis Demo

## New Features Added

### 1. Smart Rule-Based Analysis
- **Distribution Wallet Detection**: Identifies wallets with many accounts, same mint, varying amounts
- **Airdrop Wallet Detection**: Detects wallets with many small accounts, same mint
- **Exchange Wallet Detection**: Identifies wallets with high mint diversity
- **Suspicious Activity Detection**: Flags unusual patterns, extreme values
- **High Value Wallet Detection**: Identifies wallets with significant token value

### 2. Comprehensive Statistics
- Total accounts, amounts, unique mints
- Amount distribution (empty, small, medium, large accounts)
- Variance analysis for amount distribution
- Risk level assessment

### 3. Human-Readable Insights
- Wallet type classification
- Risk level assessment
- Pattern detection with confidence scores
- Evidence-based recommendations
- Natural language summaries

## Usage Examples

### Basic Analysis
```bash
cargo run -- --address <WALLET_ADDRESS> --analyze
```

### JSON Output
```bash
cargo run -- --address <WALLET_ADDRESS> --analyze --json
```

### Debug Mode with Analysis
```bash
cargo run -- --address <WALLET_ADDRESS> --analyze --debug
```

## What the Analysis Detects

### Distribution Wallet (Your 322-account case)
- **Pattern**: Many accounts (322), same mint, varying amounts
- **Classification**: Distribution wallet
- **Risk Level**: Medium
- **Summary**: "Distribution wallet detected: 322 accounts holding X total tokens across 1 unique mints. This appears to be a token distribution or airdrop preparation wallet."
- **Recommendations**: "📊 This is a distribution wallet. Monitor for airdrop announcements or token distributions."

### Airdrop Wallet
- **Pattern**: Many small accounts, same mint, 1-token accounts
- **Classification**: Airdrop wallet
- **Risk Level**: Low
- **Summary**: "Airdrop wallet detected: X accounts with small token amounts. This wallet appears to be preparing for or has completed an airdrop distribution."

### Exchange Wallet
- **Pattern**: High mint diversity, many different tokens
- **Classification**: Exchange wallet
- **Risk Level**: Medium
- **Summary**: "Exchange-like wallet detected: X accounts with Y unique token types. This wallet shows characteristics of an exchange or trading platform."

### Suspicious Activity
- **Pattern**: Extremely high account count, unusual patterns
- **Classification**: Suspicious
- **Risk Level**: High
- **Summary**: "Suspicious activity detected: X accounts with unusual patterns. This wallet shows characteristics that may indicate suspicious or automated activity."

## Architecture Benefits

### 1. Non-Disruptive Integration
- Zero-copy parser remains untouched
- Analysis runs after data collection
- Optional feature (--analyze flag)
- No performance impact on core functionality

### 2. Composable Design
- Each pattern detector is independent
- Easy to add new patterns
- Easy to test individual components
- Easy to combine results

### 3. Pure Rust Implementation
- No external dependencies
- No API costs
- No rate limits
- Works offline
- Fast and reliable

### 4. Transparent Logic
- Clear pattern detection rules
- Evidence-based recommendations
- Easy to debug and extend
- No black box algorithms

## Next Steps

1. **Test with real wallet data** using the --analyze flag
2. **Add more pattern detectors** as needed
3. **Enhance risk assessment** with additional criteria
4. **Add export functionality** for analysis results
5. **Integrate with other tools** in your workflow

The analysis framework provides powerful insights while maintaining the solid Rust foundations and zero-copy performance of your wallet scout!

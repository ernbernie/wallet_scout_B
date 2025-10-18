# Wallet Scout B

A zero-copy CLI wallet scout for Solana that provides clean, actionable insights about wallet activity.

## Quick Start

```bash
# Basic usage - shows summary dashboard
cargo run

# JSON output
cargo run -- --json

# Detailed analysis
cargo run -- --analyze

# Help
cargo run -- --help
```

## Features

- **Zero-copy parsing** - High performance SPL token account parsing
- **Smart analysis** - Detects distribution, airdrop, exchange, and suspicious wallet patterns
- **Clean output** - Summary dashboard instead of overwhelming transaction lists
- **Risk assessment** - Identifies wallet types and risk levels
- **Deterministic JSON** - Consistent output for automation

## Output Modes

- **Default**: Summary dashboard with key statistics and risk assessment
- **`--json`**: Machine-readable JSON output
- **`--analyze`**: Detailed analysis with pattern detection
- **`--summary`**: Explicit summary mode (same as default)

## Example Output

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ Wallet Scout - Summary Dashboard                                                │
└─────────────────────────────────────────────────────────────────────────────────┘

🔍 Executive Summary
This appears to be a distribution wallet with 5,953 SPL token accounts...

📊 Key Statistics
┌─────────────────────────────────────────────────────────────────────────────────┐
│ Total Accounts: 5,953    │ Unique Mints: 1      │ Total Value: 1,200,000,000   │
│ SOL Balance: 1.5 SOL    │ Max Account: 50M     │ Avg Account: 200,000         │
│ Wallet Type: Distribution │ Risk Level: Medium   │ Empty Accounts: 595          │
└─────────────────────────────────────────────────────────────────────────────────┘

⚠️  Risk Assessment: Medium - Moderate risk indicators detected
💡 Recommendation: Monitor for airdrop announcements or token distributions
```

## Performance

- Zero allocations in parse hot path
- Sub-millisecond parsing per account
- Deterministic JSON output
- Global timeout protection

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

## License

MIT
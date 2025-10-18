# Wallet Scout B

A zero-copy CLI wallet scout for Solana that inspects wallet addresses and displays SPL token accounts with minimal allocations.

## Features

- **Zero-copy parsing**: Parses SPL Token account data directly from `&[u8]` with borrowed views
- **Batch RPC calls**: Uses `getTokenAccountsByOwner` to minimize network requests
- **Exponential backoff**: Handles rate limits gracefully with retry logic
- **Interactive & scriptable**: Prompts for input or accepts command-line arguments
- **Multiple output formats**: Human-readable tables and JSON output
- **Debug mode**: Shows raw offsets and data for troubleshooting

## Usage

### Interactive Mode
```bash
cargo run
# Enter wallet address when prompted
# Select network (default: devnet)
```

### Command Line Mode
```bash
# Basic usage
cargo run -- --address <PUBKEY> --cluster devnet

# JSON output
cargo run -- --address <PUBKEY> --json

# Debug mode with raw data
cargo run -- --address <PUBKEY> --debug

# Custom RPC endpoint
cargo run -- --address <PUBKEY> --rpc-url https://api.mainnet-beta.solana.com
```

## Architecture

The project follows a clean separation of concerns:

- **`src/main.rs`**: CLI interface and orchestration
- **`src/rpc.rs`**: JSON-RPC client with backoff and batching
- **`src/parse.rs`**: Zero-copy SPL Token account parser
- **`src/view.rs`**: Display models and formatting
- **`src/errors.rs`**: Comprehensive error taxonomy

## Zero-Copy Design

The parser returns borrowed views that reference the original byte data:

```rust
pub struct TokenAccountView<'a> {
    pub mint: &'a [u8; 32],
    pub owner: &'a [u8; 32], 
    pub amount: u64,
    // ... other fields
}
```

Allocations only happen at the final display step when converting to owned strings for output.

## Error Handling

Clear error taxonomy distinguishes between:
- Network errors
- RPC errors with codes
- Rate limiting with retry-after
- Decode errors with field details
- Schema mismatches
- Not found conditions

## Testing

```bash
cargo test
```

Includes unit tests for the parser and integration tests for the full pipeline.

## Dependencies

Minimal dependency footprint:
- `clap` for CLI parsing
- `tokio` for async runtime
- `reqwest` for HTTP client
- `serde` for JSON serialization
- `base64` and `bs58` for encoding
- `proptest` for property-based testing

## Performance

- Parses accounts in <1ms each
- Handles 20+ token accounts in <1s total
- Minimal memory footprint with zero-copy design
- Batch RPC calls to minimize network overhead

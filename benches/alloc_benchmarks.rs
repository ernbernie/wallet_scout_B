use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use wallet_scout_b::parse::parse_spl_token_account;

#[cfg(feature = "alloc-prof")]
use dhat::{Dhat, DhatAlloc};

#[cfg(feature = "alloc-prof")]
#[global_allocator]
static ALLOCATOR: DhatAlloc = DhatAlloc;

/// Generate test account data of different sizes
fn generate_account_data(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    
    // Fill with realistic SPL token account data
    if size >= 72 {
        // Basic fields
        data[64..72].copy_from_slice(&12345u64.to_le_bytes());
    }
    if size >= 105 {
        data[72] = 1; // delegate_option
        data[105] = 1; // state
    }
    if size >= 123 {
        data[106] = 0; // is_native_option
        data[115..123].copy_from_slice(&67890u64.to_le_bytes()); // delegated_amount
    }
    if size >= 156 {
        data[123] = 1; // close_authority_option
    }
    
    data
}

/// Benchmark parse-only allocation profile
fn benchmark_parse_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_allocations");
    
    for size in [72, 105, 123, 156].iter() {
        let data = generate_account_data(*size);
        
        group.bench_with_input(
            BenchmarkId::new("parse_account", size),
            size,
            |b, _| {
                b.iter(|| {
                    let _view = parse_spl_token_account(black_box(&data));
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark with allocation tracking
fn benchmark_with_allocation_tracking(c: &mut Criterion) {
    #[cfg(feature = "alloc-prof")]
    let _dhat = Dhat::start_heap_profiling();
    
    let mut group = c.benchmark_group("allocation_tracking");
    
    for size in [72, 105, 123, 156].iter() {
        let data = generate_account_data(*size);
        
        group.bench_with_input(
            BenchmarkId::new("parse_with_tracking", size),
            size,
            |b, _| {
                b.iter(|| {
                    let _view = parse_spl_token_account(black_box(&data));
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark parsing multiple accounts
fn benchmark_parse_multiple_accounts(c: &mut Criterion) {
    #[cfg(feature = "alloc-prof")]
    let _dhat = Dhat::start_heap_profiling();
    
    let mut group = c.benchmark_group("parse_multiple");
    
    for count in [10, 100, 500, 1000].iter() {
        let accounts: Vec<Vec<u8>> = (0..*count)
            .map(|i| generate_account_data(72 + (i % 4) * 20)) // Vary sizes
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("parse_multiple", count),
            count,
            |b, _| {
                b.iter(|| {
                    for account_data in &accounts {
                        let _view = parse_spl_token_account(black_box(account_data));
                    }
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_parse_allocations,
    benchmark_with_allocation_tracking,
    benchmark_parse_multiple_accounts
);

criterion_main!(benches);

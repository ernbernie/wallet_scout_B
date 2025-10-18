use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use wallet_scout_b::analysis::WalletAnalyzer;
use wallet_scout_b::view::TokenRow;

/// Generate test data for benchmarking
fn generate_test_data(account_count: usize) -> Vec<TokenRow> {
    let mut tokens = Vec::new();
    let mint = "6kbwsSY4hL6WVadLRLnWV2irkMN2AvFZVAS8McKJmAtJ";

    for i in 0..account_count {
        tokens.push(TokenRow {
            account: format!("Account{}", i),
            mint: mint.to_string(),
            owner: "Owner123".to_string(),
            amount_raw: if i % 10 == 0 {
                0
            } else {
                (i as u64 + 1) * 1_000_000
            },
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }

    tokens
}

/// Benchmark analysis performance with different wallet sizes
fn benchmark_analysis_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("analysis_performance");

    let analyzer = WalletAnalyzer::new();

    for account_count in [10, 100, 500, 1000, 2000].iter() {
        let tokens = generate_test_data(*account_count);

        group.bench_with_input(
            BenchmarkId::new("analyze", account_count),
            account_count,
            |b, _| {
                b.iter(|| {
                    let _insights = analyzer.analyze(black_box(&tokens));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark pattern detection performance
fn benchmark_pattern_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_detection");

    let analyzer = WalletAnalyzer::new();

    for account_count in [50, 100, 500, 1000].iter() {
        let tokens = generate_test_data(*account_count);

        group.bench_with_input(
            BenchmarkId::new("detect_patterns", account_count),
            account_count,
            |b, _| {
                b.iter(|| {
                    let _patterns = analyzer.detect_patterns(black_box(&tokens));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark statistics calculation performance
fn benchmark_statistics_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics_calculation");

    let analyzer = WalletAnalyzer::new();

    for account_count in [100, 500, 1000, 2000, 5000].iter() {
        let tokens = generate_test_data(*account_count);

        group.bench_with_input(
            BenchmarkId::new("calculate_statistics", account_count),
            account_count,
            |b, _| {
                b.iter(|| {
                    let _stats = analyzer.calculate_statistics(black_box(&tokens));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark summary generation performance
fn benchmark_summary_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("summary_generation");

    let analyzer = WalletAnalyzer::new();

    for account_count in [100, 500, 1000].iter() {
        let tokens = generate_test_data(*account_count);
        let insights = analyzer.analyze(&tokens);

        group.bench_with_input(
            BenchmarkId::new("generate_summary", account_count),
            account_count,
            |b, _| {
                b.iter(|| {
                    let _summary = analyzer.generate_summary(
                        black_box(&insights.wallet_type),
                        black_box(&insights.patterns),
                        black_box(&insights.statistics),
                    );
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory allocation patterns
fn benchmark_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    let analyzer = WalletAnalyzer::new();

    for account_count in [100, 500, 1000, 2000].iter() {
        let tokens = generate_test_data(*account_count);

        group.bench_with_input(
            BenchmarkId::new("full_analysis", account_count),
            account_count,
            |b, _| {
                b.iter(|| {
                    let _insights = analyzer.analyze(black_box(&tokens));
                    // Force allocation by cloning the result
                    let _cloned = _insights.clone();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark different wallet types
fn benchmark_wallet_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("wallet_types");

    let analyzer = WalletAnalyzer::new();

    // Distribution wallet (many accounts, same mint)
    let distribution_tokens = generate_test_data(1000);
    group.bench_function("distribution_wallet", |b| {
        b.iter(|| {
            let _insights = analyzer.analyze(black_box(&distribution_tokens));
        });
    });

    // Exchange wallet (many different mints)
    let mut exchange_tokens = Vec::new();
    for i in 0..1000 {
        exchange_tokens.push(TokenRow {
            account: format!("ExchangeAccount{}", i),
            mint: format!("Mint{}", i % 100), // 100 different mints
            owner: "ExchangeOwner".to_string(),
            amount_raw: 1_000_000,
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }

    group.bench_function("exchange_wallet", |b| {
        b.iter(|| {
            let _insights = analyzer.analyze(black_box(&exchange_tokens));
        });
    });

    // Airdrop wallet (many small amounts)
    let mut airdrop_tokens = Vec::new();
    for i in 0..1000 {
        airdrop_tokens.push(TokenRow {
            account: format!("AirdropAccount{}", i),
            mint: "AirdropMint".to_string(),
            owner: "AirdropOwner".to_string(),
            amount_raw: if i % 5 == 0 { 1 } else { 100_000 },
            state: 0,
            delegated_amount: 0,
            delegate: None,
            close_authority: None,
        });
    }

    group.bench_function("airdrop_wallet", |b| {
        b.iter(|| {
            let _insights = analyzer.analyze(black_box(&airdrop_tokens));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_analysis_performance,
    benchmark_pattern_detection,
    benchmark_statistics_calculation,
    benchmark_summary_generation,
    benchmark_memory_allocation,
    benchmark_wallet_types
);

criterion_main!(benches);

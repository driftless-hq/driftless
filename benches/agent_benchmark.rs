use criterion::{black_box, criterion_group, criterion_main, Criterion};
use driftless::agent::{Agent, AgentConfig};

/// Benchmark agent initialization
fn bench_agent_initialization(c: &mut Criterion) {
    c.bench_function("agent_initialization", |b| {
        b.iter(|| {
            let config = AgentConfig::default();
            let _agent = Agent::new(black_box(config));
        });
    });
}

/// Benchmark resource usage monitoring
fn bench_resource_monitoring(c: &mut Criterion) {
    let mut agent = Agent::new(AgentConfig::default());

    c.bench_function("resource_monitoring", |b| {
        b.iter(|| {
            agent.update_resource_usage_sync();
            black_box(agent.memory_usage());
            black_box(agent.cpu_usage());
        });
    });
}

/// Benchmark circuit breaker operations
fn bench_circuit_breaker(c: &mut Criterion) {
    let mut agent = Agent::new(AgentConfig::default());

    c.bench_function("circuit_breaker_attempt", |b| {
        b.iter(|| {
            black_box(agent.can_attempt_apply());
        });
    });

    c.bench_function("circuit_breaker_record_success", |b| {
        b.iter(|| {
            agent.record_apply_success();
        });
    });

    c.bench_function("circuit_breaker_record_failure", |b| {
        b.iter(|| {
            agent.record_apply_failure();
        });
    });
}

/// Benchmark metrics collection
fn bench_metrics_collection(c: &mut Criterion) {
    let agent = Agent::new(AgentConfig::default());

    c.bench_function("apply_metrics", |b| {
        b.iter(|| {
            black_box(agent.apply_metrics());
        });
    });

    c.bench_function("facts_metrics", |b| {
        b.iter(|| {
            black_box(agent.facts_metrics());
        });
    });

    c.bench_function("logs_metrics", |b| {
        b.iter(|| {
            black_box(agent.logs_metrics());
        });
    });

    c.bench_function("circuit_breaker_status", |b| {
        b.iter(|| {
            black_box(agent.circuit_breaker_status());
        });
    });
}

/// Benchmark configuration loading (simulated)
fn bench_config_loading(c: &mut Criterion) {
    c.bench_function("load_agent_config", |b| {
        b.iter(|| {
            // This would normally load from disk, but we'll simulate
            let _config = black_box(AgentConfig::default());
        });
    });
}

criterion_group!(
    benches,
    bench_agent_initialization,
    bench_resource_monitoring,
    bench_circuit_breaker,
    bench_metrics_collection,
    bench_config_loading
);
criterion_main!(benches);

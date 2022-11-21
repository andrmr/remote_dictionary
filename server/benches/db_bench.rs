#[macro_use]
extern crate lazy_static;

use criterion::{Criterion, criterion_group, criterion_main};
use server::Db;
use tokio::runtime::Runtime;

lazy_static! {
    pub static ref DB: Db<String, String> = Db::create("./test.db", "test".into())
        .unwrap();

    pub static ref RUNTIME: Runtime = tokio::runtime::Runtime::new()
        .unwrap();
}

// TODO: mixed workload benches

fn db_get_benchmark(c: &mut Criterion) {
    let key = "dummy".to_owned();

    c.bench_function("Db<String, String> _ 1000 _ GET", move |b| {
        b.to_async(&*RUNTIME).iter(|| async { DB.get(&key).await });
    });
}

fn db_set_benchmark(c: &mut Criterion) {
    let key = "dummy".to_owned();
    let val = "dummy_val".to_owned();

    c.bench_function("Db<String, String> _ 1000 _ GET", move |b| {
        b.to_async(&*RUNTIME).iter(|| async { DB.set(&key, &val).await });
    });
}

criterion_group!(db_bench, db_get_benchmark, db_set_benchmark);
criterion_main!(db_bench);

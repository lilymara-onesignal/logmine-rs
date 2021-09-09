use criterion::{black_box, criterion_group, criterion_main, Criterion};
use logmine_rs::pattern::Pattern;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("merge patterns", |b| {
        b.iter(|| {
            let pattern1 = Pattern::default()
                .push_text("a")
                .push_text("b")
                .push_text("c")
                .push_text("d")
                .push_text("e");

            let pattern2 = Pattern::default()
                .push_text("a")
                .push_text("b")
                // .push_text("c")
                .push_text("d")
                .push_text("e");

            black_box(pattern1.merge(pattern2));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

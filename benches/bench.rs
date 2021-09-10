use std::{fs::File, io::Read};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use logmine_rs::{clusterer::Clusterer, pattern::Pattern};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("merge patterns", |b| {
        b.iter(|| {
            let mut pattern1 = Pattern::default();
            pattern1
                .push_text("a")
                .push_text("b")
                .push_text("c")
                .push_text("d")
                .push_text("e");

            let mut pattern2 = Pattern::default();
            pattern2
                .push_text("a")
                .push_text("b")
                // .push_text("c")
                .push_text("d")
                .push_text("e");

            black_box(pattern1.merge(pattern2));
        })
    });

    c.bench_function("clusterer", |b| {
        let mut f = File::open("test_files/c.txt").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();

        b.iter(|| {
            let mut clusterer = Clusterer::new();

            for line in s.lines() {
                clusterer.process_line(line);
            }

            black_box(clusterer.take_result().collect::<Vec<_>>());
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

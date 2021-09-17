use std::{fs::File, io::BufReader};

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use indicatif::{ProgressBar, ProgressDrawTarget};
use logmine_rs::{clusterer::ClustererOptions, pattern::Pattern};
use rayon::ThreadPoolBuilder;
use regex::Regex;

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

    let f = File::open("test_files/c.txt").unwrap();
    let size = f.metadata().unwrap().len();

    let mut group = c.benchmark_group("clusterer");
    group
        .throughput(Throughput::Bytes(size))
        .bench_function("single-core", |b| {
            b.iter_batched(
                || Regex::new("\\s+").unwrap(),
                |split_regex| {
                    let f = BufReader::new(File::open("test_files/c.txt").unwrap());
                    let progress = ProgressBar::new(0);
                    progress.set_draw_target(ProgressDrawTarget::hidden());

                    black_box(logmine_rs::main_single_core(
                        ClustererOptions {
                            ..Default::default()
                        },
                        f,
                        progress,
                        split_regex,
                    ));
                },
                criterion::BatchSize::SmallInput,
            )
        });

    group.bench_function("multi-core", |b| {
        b.iter_batched(
            || {
                (
                    ThreadPoolBuilder::new().num_threads(4).build().unwrap(),
                    BufReader::new(File::open("test_files/c.txt").unwrap()),
                    {
                        let progress = ProgressBar::new(0);
                        progress.set_draw_target(ProgressDrawTarget::hidden());
                        progress
                    },
                    Regex::new("\\s+").unwrap(),
                )
            },
            |(pool, f, progress, split_regex)| {
                black_box(logmine_rs::parallel_clusterer::run(
                    Default::default(),
                    2,
                    f,
                    progress,
                    split_regex,
                    pool,
                ));
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

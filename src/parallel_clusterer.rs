use indicatif::ProgressBar;
use parking_lot::Mutex;
use std::{io::BufRead, sync::Arc};

use crossbeam_channel::Sender;
use rayon::ThreadPoolBuilder;

use crate::{
    clusterer::{Cluster, Clusterer, ClustererOptions},
    scoring,
};

/// Number of times each IO thread will attempt to steal the lock on the file
/// between CPU-bound work
const LOCK_STEAL_ATTEMPTS: usize = 4;

pub fn run(
    options: ClustererOptions,
    read_chunk_size: usize,
    file: impl 'static + Sync + Send + BufRead,
    jobs: usize,
    progress: ProgressBar,
) -> Vec<Cluster<'static>> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(jobs)
        .thread_name(|i| format!("logmine-wrk-{}", i))
        .build()
        .unwrap();
    let (tx, rx) = crossbeam_channel::bounded(pool.current_num_threads());

    let file = Arc::new(Mutex::new(file));

    for _ in 0..pool.current_num_threads() {
        let tx = tx.clone();
        let file = file.clone();
        let progress = progress.clone();

        pool.spawn(move || {
            run_single_thread(tx, options, read_chunk_size, file, progress);
        });
    }

    drop(tx);

    let mut total: Vec<Cluster<'static>> = Vec::new();

    for thread_results in rx {
        merge(&mut total, thread_results, options);
    }

    total
}

fn fill(lines: &mut Vec<String>, reader: &mut impl BufRead) {
    for _ in 0..lines.capacity() {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap() == 0 {
            break;
        }
        lines.push(line);
    }
}

fn run_single_thread(
    tx: Sender<Vec<Cluster<'static>>>,
    options: ClustererOptions,
    read_chunk_size: usize,
    file: Arc<Mutex<impl BufRead>>,
    progress: ProgressBar,
) {
    let mut clusterer = Clusterer::default().with_options(options);

    let mut lines = Vec::with_capacity(read_chunk_size);

    'outer: loop {
        let mut lock = file.lock();

        fill(&mut lines, &mut *lock);
        drop(lock);
        if lines.is_empty() {
            break;
        }

        for _ in 0..LOCK_STEAL_ATTEMPTS {
            let range_max = (lines.capacity() / LOCK_STEAL_ATTEMPTS).min(lines.len());

            let mut size = 0;
            for line in lines.drain(..range_max) {
                clusterer.process_line(&line);
                size += line.len();
            }
            progress.inc(size as u64);

            if let Some(mut lock) = file.try_lock() {
                fill(&mut lines, &mut *lock);
                if lines.is_empty() {
                    break 'outer;
                }
            }
        }
    }

    tx.send(clusterer.take_result().collect()).unwrap();
}

fn merge(
    total: &mut Vec<Cluster<'static>>,
    thread_results: Vec<Cluster<'static>>,
    options: ClustererOptions,
) {
    for mut cluster_a in thread_results {
        for cluster_b in total.iter_mut() {
            let score = scoring::distance(
                &cluster_a.representative,
                &cluster_b.representative,
                options.max_dist,
            );

            if score <= options.max_dist {
                cluster_b.count += 1;

                let pattern_b = std::mem::take(&mut cluster_b.pattern);

                cluster_b.pattern = cluster_a.pattern.merge(pattern_b);
                return;
            }
        }

        total.push(cluster_a);
    }
}

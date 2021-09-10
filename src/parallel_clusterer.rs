use std::io::BufRead;

use crossbeam_channel::Sender;
use rayon::ThreadPoolBuilder;

use crate::{
    clusterer::{Cluster, Clusterer},
    scoring,
};

pub fn run(clusterer: Clusterer, read_chunk_size: usize) -> Vec<Cluster<'static>> {
    let pool = ThreadPoolBuilder::new()
        .thread_name(|i| format!("logmine-wrk-{}", i))
        .build()
        .unwrap();
    let (tx, rx) = crossbeam_channel::bounded(pool.current_num_threads());

    for _ in 0..pool.current_num_threads() {
        let clusterer = clusterer.clone();
        let tx = tx.clone();

        pool.spawn(move || {
            run_single_thread(tx, clusterer, read_chunk_size);
        });
    }

    drop(tx);

    let mut total: Vec<Cluster<'static>> = Vec::new();

    for thread_results in rx {
        merge(&mut total, thread_results, &clusterer);
    }

    total
}

fn run_single_thread(
    tx: Sender<Vec<Cluster<'static>>>,
    mut clusterer: Clusterer,
    read_chunk_size: usize,
) {
    let stdin = std::io::stdin();
    let mut lines = Vec::with_capacity(read_chunk_size);

    loop {
        {
            let stdin_lock = stdin.lock();
            let mut lines_iter = stdin_lock.lines();

            for _ in 0..read_chunk_size {
                match lines_iter.next() {
                    Some(line) => lines.push(line.unwrap()),
                    None => break,
                }
            }
            if lines.is_empty() {
                break;
            }
        }

        for line in lines.drain(..) {
            clusterer.process_line(&line);
        }
    }

    tx.send(clusterer.take_result().collect()).unwrap();
}

fn merge(
    total: &mut Vec<Cluster<'static>>,
    thread_results: Vec<Cluster<'static>>,
    clusterer: &Clusterer,
) {
    for mut cluster_a in thread_results {
        for cluster_b in total.iter_mut() {
            let score = scoring::distance(
                &cluster_a.representative,
                &cluster_b.representative,
                clusterer.max_dist,
            );

            if score <= clusterer.max_dist {
                cluster_b.count += 1;

                let pattern_b = std::mem::take(&mut cluster_b.pattern);

                cluster_b.pattern = cluster_a.pattern.merge(pattern_b);
                return;
            }
        }

        total.push(cluster_a);
    }
}

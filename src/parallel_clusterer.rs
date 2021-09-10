use crossbeam_channel::Receiver;
use rayon::ThreadPoolBuilder;

use crate::{
    clusterer::{Cluster, Clusterer},
    scoring,
};

pub fn run(clusterer: Clusterer, lines_rx: Receiver<String>) -> Vec<Cluster<'static>> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_cpus::get_physical())
        .build()
        .unwrap();
    let (tx, rx) = crossbeam_channel::bounded(pool.current_num_threads());

    for _ in 0..pool.current_num_threads() {
        let mut clusterer = clusterer.clone();
        let tx = tx.clone();
        let lines_rx = lines_rx.clone();

        pool.spawn(move || {
            for line in lines_rx {
                clusterer.process_line(&line);
            }

            tx.send(clusterer.take_result().collect()).unwrap();
        });
    }

    drop(lines_rx);
    drop(tx);

    let mut total: Vec<Cluster<'static>> = Vec::new();

    for thread_results in rx {
        merge(&mut total, thread_results, &clusterer);
    }

    total
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

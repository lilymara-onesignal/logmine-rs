use rayon::prelude::*;
use std::sync::mpsc::Sender;

use crate::{
    clusterer::{Cluster, Clusterer},
    scoring,
};

#[derive(Clone)]
struct ParallelClusterer {
    clusterer: Option<Clusterer>,
    tx: Sender<Vec<Cluster<'static>>>,
}

impl Drop for ParallelClusterer {
    fn drop(&mut self) {
        let clusterer = self.clusterer.take().unwrap();
        self.tx.send(clusterer.result()).unwrap();
    }
}

pub fn run(
    clusterer: Clusterer,
    lines: impl Send + Iterator<Item = String>,
) -> Vec<Cluster<'static>> {
    let (tx, rx) = std::sync::mpsc::channel();

    lines.par_bridge().for_each_with(
        ParallelClusterer {
            clusterer: Some(clusterer.clone()),
            tx,
        },
        |clusterer, line| {
            if let Some(c) = &mut clusterer.clusterer {
                c.process_line(&line);
            }
        },
    );

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
    for cluster_a in thread_results {
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

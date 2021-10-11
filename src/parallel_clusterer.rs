use indicatif::ProgressBar;
use parking_lot::Mutex;
use regex::Regex;
use std::{io::BufRead, sync::Arc};

use crossbeam_channel::Sender;
use rayon::ThreadPool;

use crate::{
    clusterer::{Cluster, Clusterer, ClustererOptions},
    pool::StringPool,
    scoring,
};

/// Number of times each IO thread will attempt to steal the lock on the file
/// between CPU-bound work
const LOCK_STEAL_ATTEMPTS: usize = 4;

pub fn run(
    options: ClustererOptions,
    read_chunk_size: usize,
    file: impl Sync + Send + BufRead,
    progress: ProgressBar,
    split_regex: Regex,
    pool: ThreadPool,
) -> Vec<Cluster<'static>> {
    let (tx, rx) = crossbeam_channel::bounded(pool.current_num_threads());

    let file = Arc::new(Mutex::new(file));

    pool.scope(|scope| {
        for _ in 0..pool.current_num_threads() {
            let tx = tx.clone();
            let file = file.clone();
            let progress = progress.clone();
            let split_regex = split_regex.clone();

            scope.spawn(move |_| {
                run_single_thread(tx, options, read_chunk_size, file, progress, split_regex);
            });
        }

        drop(tx);
    });

    let mut total: Vec<Cluster<'static>> = Vec::new();

    for thread_results in rx {
        merge(&mut total, thread_results, options);
    }

    total
}

fn fill(lines: &mut StringPool, reader: &mut impl BufRead) {
    while let Some(mut line) = lines.take_dead() {
        if reader.read_line(&mut *line).unwrap() == 0 {
            line.stay_dead();
            break;
        }
    }
}

fn run_single_thread(
    tx: Sender<Vec<Cluster<'static>>>,
    options: ClustererOptions,
    read_chunk_size: usize,
    file: Arc<Mutex<impl BufRead>>,
    progress: ProgressBar,
    split_regex: Regex,
) {
    let mut clusterer = Clusterer::new(options, split_regex);

    let mut lines = StringPool::with_capacity(read_chunk_size);

    'outer: loop {
        let mut lock = file.lock();

        fill(&mut lines, &mut *lock);
        drop(lock);
        if lines.is_empty() {
            break;
        }

        for _ in 0..LOCK_STEAL_ATTEMPTS {
            let range_max = if lines.capacity() < LOCK_STEAL_ATTEMPTS {
                lines.len()
            } else {
                (lines.capacity() / LOCK_STEAL_ATTEMPTS).min(lines.len())
            };

            let mut size = 0;
            for _ in 0..range_max {
                let line = lines.take_live().unwrap();
                clusterer.process_line(line.as_ref());
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

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use indicatif::{ProgressBar, ProgressDrawTarget};
    use rayon::ThreadPoolBuilder;
    use regex::Regex;

    use super::run;

    #[test]
    fn test_file_c_completes() {
        let f = BufReader::new(File::open("test_files/c.txt").unwrap());
        let progress = ProgressBar::new(0);
        progress.set_draw_target(ProgressDrawTarget::hidden());

        run(
            Default::default(),
            2,
            f,
            progress,
            Regex::new("\\s+").unwrap(),
            ThreadPoolBuilder::new().num_threads(1).build().unwrap(),
        );
    }
}

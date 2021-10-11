use std::io::BufRead;

use clusterer::{Cluster, Clusterer, ClustererOptions};
use indicatif::ProgressBar;
use regex::Regex;

#[macro_use]
#[cfg(test)]
mod macros;

pub mod clusterer;
pub mod parallel_clusterer;
pub mod pattern;
mod pool;
pub mod scoring;

/// special-cased runner for when user passes --jobs=1. This avoids the
/// threading & communication overhead of the parallel mode (~10%). With a non-1
/// value for --jobs, this overhead is dwarfed by the performance gains from
/// parallelism.
pub fn main_single_core(
    options: ClustererOptions,
    mut file: impl BufRead,
    progress: ProgressBar,
    split_regex: Regex,
) -> Vec<Cluster<'static>> {
    let mut clusterer = Clusterer::new(options, split_regex);

    let mut line = String::new();

    'outer: loop {
        let mut size = 0;
        for _ in 0..100 {
            line.clear();
            if file.read_line(&mut line).unwrap() == 0 {
                break 'outer;
            }

            clusterer.process_line(&line);
            size += line.len();
        }
        progress.inc(size as u64);
    }

    clusterer.take_result().collect()
}

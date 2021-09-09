#[macro_use]
#[cfg(test)]
mod macros;

mod clusterer;
mod parallel_clusterer;
mod pattern;
mod processor;
mod scoring;

use std::io::BufRead;

use clusterer::{Cluster, Clusterer};
use structopt::StructOpt;

#[derive(structopt::StructOpt)]
struct Options {
    #[structopt(long)]
    parallel: bool,

    #[structopt(long, default_value = "0.6")]
    max_distance: f64,

    #[structopt(long, default_value = "2")]
    min_members: u32,
}

fn main() {
    let opts = Options::from_args();

    let clusterer = Clusterer::new()
        .with_max_dist(opts.max_distance)
        .with_min_members(opts.min_members);

    let clusters = if opts.parallel {
        main_parallel(clusterer)
    } else {
        main_single_core(clusterer)
    };

    for c in clusters {
        println!("{}", c);
    }
}

fn main_single_core(mut clusterer: Clusterer) -> Vec<Cluster<'static>> {
    let stdin = std::io::stdin();
    let mut line = String::new();

    loop {
        line.clear();
        if stdin.read_line(&mut line).unwrap() == 0 {
            break;
        }

        clusterer.process_line(&mut line);
    }

    clusterer.result()
}

fn main_parallel(clusterer: Clusterer) -> Vec<Cluster<'static>> {
    let (tx, rx) = std::sync::mpsc::sync_channel(10_000);

    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let stdin_lock = stdin.lock();

        for line in stdin_lock.lines() {
            tx.send(line.unwrap()).unwrap();
        }
    });

    parallel_clusterer::run(clusterer, rx.into_iter())
}

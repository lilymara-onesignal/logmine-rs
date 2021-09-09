#[macro_use]
#[cfg(test)]
mod macros;

mod clusterer;
mod parallel_clusterer;
mod patterns;
mod processor;
mod scoring;

use std::io::BufRead;

use clusterer::Clusterer;
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

    if opts.parallel {
        main_parallel(clusterer);
    } else {
        main_single_core(clusterer);
    }
}

fn main_single_core(mut clusterer: Clusterer) {
    let stdin = std::io::stdin();
    let mut line = String::new();

    loop {
        line.clear();
        if stdin.read_line(&mut line).unwrap() == 0 {
            break;
        }

        clusterer.process_line(&mut line);
    }

    for c in clusterer.result() {
        println!("{}", c);
    }
}

fn main_parallel(clusterer: Clusterer) {
    let (tx, rx) = std::sync::mpsc::sync_channel(10_000);

    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let stdin_lock = stdin.lock();

        for line in stdin_lock.lines() {
            tx.send(line.unwrap()).unwrap();
        }
    });

    for c in parallel_clusterer::run(clusterer, rx.into_iter()) {
        println!("{}", c);
    }
}

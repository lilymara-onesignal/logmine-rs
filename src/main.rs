use std::io::BufRead;

use logmine_rs::clusterer::{Cluster, Clusterer};
use structopt::StructOpt;

#[derive(structopt::StructOpt)]
/// Use the logmine algorithm to find patterns in log files
struct Options {
    /// Run on all available system cores. By default logmine runs in a
    /// single-threaded mode.
    #[structopt(long)]
    parallel: bool,

    /// Controls the granularity of the clustering algorithm. Lower values of
    /// max_distance will increase the granularity of clustering.
    #[structopt(long, default_value = "0.6")]
    max_distance: f64,

    /// Minimum size of clusters to print in the output report. IE if
    /// min_members is 2, and there is only one log entry matching a particular
    /// pattern, that pattern will not be printed in the output.
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

        clusterer.process_line(&line);
    }

    clusterer.take_result().collect()
}

fn main_parallel(clusterer: Clusterer) -> Vec<Cluster<'static>> {
    let (tx, rx) = std::sync::mpsc::sync_channel(100_000);

    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let stdin_lock = stdin.lock();

        for line in stdin_lock.lines() {
            tx.send(line.unwrap()).unwrap();
        }
    });

    logmine_rs::parallel_clusterer::run(clusterer, rx.into_iter())
}

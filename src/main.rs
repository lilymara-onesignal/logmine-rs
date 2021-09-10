use logmine_rs::clusterer::{Cluster, Clusterer};
use structopt::StructOpt;

#[derive(structopt::StructOpt)]
/// Find patterns in log files. Does not take in a file name to read, only reads
/// from stdin
struct Options {
    /// Pin logmine to a single core rather than trying to use all available CPU
    /// cores
    #[structopt(long)]
    single_core: bool,

    /// Number of lines read at a time by each thread when running in parallel mode
    #[structopt(long, short = "c", default_value = "10000")]
    parallel_read_chunk_size: usize,

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

    let clusters = if opts.single_core {
        main_single_core(clusterer)
    } else {
        logmine_rs::parallel_clusterer::run(clusterer, opts.parallel_read_chunk_size)
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

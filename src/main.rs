use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use indicatif::{ProgressBar, ProgressStyle};
use logmine_rs::clusterer::{Cluster, Clusterer, ClustererOptions};
use structopt::StructOpt;

#[derive(structopt::StructOpt)]
/// Find patterns in log files
struct Options {
    /// Number of parallel threads to run
    #[structopt(long, short)]
    jobs: Option<usize>,

    /// Number of lines read at a time by each thread when running in parallel
    /// mode. Has zero effect when --jobs=1.
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

    /// Path to the file to read. Will read from stdin if not specified.
    file: Option<PathBuf>,
}

fn main() {
    let opts = Options::from_args();

    let clusterer_options = ClustererOptions::default()
        .with_max_dist(opts.max_distance)
        .with_min_members(opts.min_members);

    let (file_path, is_stdin) = match opts.file {
        Some(file) => (file, false),
        None => ("/dev/stdin".into(), true),
    };

    let file = File::open(file_path).unwrap();
    let filesize_bytes = file.metadata().unwrap().len();

    let progress_bar = if is_stdin {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner} {elapsed_precise} {bytes} {bytes_per_sec}"),
        );
        bar
    } else {
        let bar = ProgressBar::new(filesize_bytes);
        bar.set_style(ProgressStyle::default_bar().template(
            "{percent}% {bytes} {bar:40.cyan/blue} {total_bytes} {elapsed_precise} (eta: {eta_precise}) {bytes_per_sec}",
        ));
        bar
    };

    let file = BufReader::new(file);

    let jobs = opts.jobs.unwrap_or_else(|| num_cpus::get_physical());

    let clusters = if jobs == 1 {
        main_single_core(clusterer_options, file, progress_bar.clone())
    } else {
        logmine_rs::parallel_clusterer::run(
            clusterer_options,
            opts.parallel_read_chunk_size,
            file,
            jobs,
            progress_bar.clone(),
        )
    };

    progress_bar.finish_at_current_pos();

    for c in clusters {
        println!("{}", c);
    }
}

/// special-cased runner for when user passes --jobs=1. This avoids the
/// threading & communication overhead of the parallel mode (~10%). With a non-1
/// value for --jobs, this overhead is dwarfed by the performance gains from
/// parallelism.
fn main_single_core(
    options: ClustererOptions,
    mut file: impl BufRead,
    progress: ProgressBar,
) -> Vec<Cluster<'static>> {
    let mut clusterer = Clusterer::default().with_options(options);

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

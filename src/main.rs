use std::{fs::File, io::BufReader, path::PathBuf};

use indicatif::{ProgressBar, ProgressStyle};
use logmine_rs::clusterer::ClustererOptions;
use rayon::ThreadPoolBuilder;
use regex::Regex;
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

    /// Regex pattern to use to split segments of each line.
    #[structopt(long, default_value = "\\s+")]
    split_pattern: String,

    /// Path to the file to read. Will read from stdin if not specified.
    file: Option<PathBuf>,
}

fn main() {
    let opts = Options::from_args();

    let split_regex = Regex::new(&opts.split_pattern).unwrap();

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

    let jobs = opts.jobs.unwrap_or_else(num_cpus::get_physical);

    let mut clusters = if jobs == 1 {
        logmine_rs::main_single_core(clusterer_options, file, progress_bar.clone(), split_regex)
    } else {
        logmine_rs::parallel_clusterer::run(
            clusterer_options,
            opts.parallel_read_chunk_size,
            file,
            progress_bar.clone(),
            split_regex,
            ThreadPoolBuilder::new()
                .num_threads(jobs)
                .thread_name(|i| format!("logmine-wrk-{}", i))
                .build()
                .unwrap(),
        )
    };

    progress_bar.finish_at_current_pos();

    clusters.sort_by(|c1, c2| c2.count.cmp(&c1.count));

    for c in clusters {
        println!("{}", c);
    }
}

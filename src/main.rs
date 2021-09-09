use std::io::BufRead;

#[macro_use]
#[cfg(test)]
mod macros;

mod clusterer;
mod pattern_generator;
mod processor;
mod scorer;

use clusterer::Clusterer;

fn main() {
    let mut clusterer = Clusterer::new().with_max_dist(0.6).with_min_members(2);

    let stdin = std::io::stdin();
    let lock = stdin.lock();

    for line in lock.lines() {
        let line = line.unwrap();

        clusterer.process_line(&line);
    }

    for c in clusterer.result() {
        println!("{}", c);
    }
}

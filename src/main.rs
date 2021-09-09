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
    let mut line = String::new();

    loop {
        line.clear();
        let read = stdin.read_line(&mut line).unwrap();
        if read == 0 {
            break;
        }

        clusterer.process_line(&line);
    }

    for c in clusterer.result() {
        println!("{}", c);
    }
}

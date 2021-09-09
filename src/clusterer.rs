use std::fmt;

use crate::{
    pattern_generator::{PatternElement, PatternGenerator},
    scorer::Scorer,
};

pub struct Clusterer {
    clusters: Vec<Cluster>,
    max_dist: f64,
    scorer: Scorer,
    pattern_generator: PatternGenerator,
    min_members: i32,
}

#[derive(Debug, PartialEq)]
pub struct Cluster {
    representative: Vec<String>,
    count: i32,
    pattern: Vec<PatternElement>,
}

impl fmt::Display for Cluster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.count)?;

        for element in &self.pattern {
            match element {
                PatternElement::Text(t) => write!(f, "{} ", t)?,
                PatternElement::Placeholder => write!(f, "----- ")?,
            }
        }

        Ok(())
    }
}

impl Clusterer {
    pub fn new() -> Self {
        Self {
            clusters: Vec::new(),
            max_dist: 0.01,
            scorer: Scorer::new(),
            pattern_generator: PatternGenerator::new(),
            min_members: 1,
        }
    }

    pub fn with_max_dist(mut self, max_dist: f64) -> Self {
        self.max_dist = max_dist;
        self
    }

    pub fn with_min_members(mut self, min_members: i32) -> Self {
        self.min_members = min_members;
        self
    }

    pub fn process_line(&mut self, line: &str) {
        let tokens: Vec<String> = line.split(" ").map(|m| m.to_owned()).collect();
        let pattern: Vec<PatternElement> = tokens
            .iter()
            .map(|t| PatternElement::Text(t.clone()))
            .collect();

        let mut found = false;

        for cluster in &mut self.clusters {
            let score = self
                .scorer
                .distance(&cluster.representative, &tokens, self.max_dist);

            if score <= self.max_dist {
                found = true;
                cluster.count += 1;
                cluster.pattern = self
                    .pattern_generator
                    .create_pattern(&cluster.pattern, &pattern);
            }
        }

        if !found {
            self.clusters.push(Cluster {
                representative: tokens,
                count: 1,
                pattern,
            });
        }
    }

    pub fn result(self) -> Vec<Cluster> {
        if self.min_members > 1 {
            let min_members = self.min_members;

            self.clusters
                .into_iter()
                .filter(|c| c.count >= min_members)
                .collect()
        } else {
            self.clusters
        }
    }

    fn find(mut self, input_lines: &[&str]) -> Vec<Cluster> {
        for line in input_lines {
            self.process_line(line);
        }
        self.result()
    }
}

#[cfg(test)]
mod test {
    use crate::pattern_generator::PatternElement;

    use super::{Cluster, Clusterer};

    #[test]
    fn test() {
        let clusters =
            Clusterer::new()
                .with_max_dist(0.5)
                .find(&["hello 1 y 3", "hello 1 x 3", "abc m n q"]);

        assert_eq!(
            clusters,
            vec![
                Cluster {
                    representative: vec_into!["hello", "1", "y", "3"],
                    count: 2,
                    pattern: vec_into!["hello", "1", PatternElement::Placeholder, "3"]
                },
                Cluster {
                    representative: vec_into!["abc", "m", "n", "q"],
                    count: 1,
                    pattern: vec_into!["abc", "m", "n", "q"]
                },
            ]
        );
    }

    #[test]
    fn test_min_members() {
        let clusters = Clusterer::new()
            .with_max_dist(0.5)
            .with_min_members(2)
            .find(&["hello 1 y 3", "hello 1 x 3", "abc m n q"]);

        assert_eq!(
            clusters,
            vec![Cluster {
                representative: vec_into!["hello", "1", "y", "3"],
                count: 2,
                pattern: vec_into!["hello", "1", PatternElement::Placeholder, "3"]
            }]
        );
    }

    #[test]
    fn test_small_max_dist() {
        let clusters =
            Clusterer::new()
                .with_max_dist(0.01)
                .find(&["hello 1 y 3", "hello 1 x 3", "abc m n q"]);

        assert_eq!(
            clusters,
            vec![
                Cluster {
                    representative: vec_into!["hello", "1", "y", "3"],
                    count: 1,
                    pattern: vec_into!["hello", "1", "y", "3"]
                },
                Cluster {
                    representative: vec_into!["hello", "1", "x", "3"],
                    count: 1,
                    pattern: vec_into!["hello", "1", "x", "3"]
                },
                Cluster {
                    representative: vec_into!["abc", "m", "n", "q"],
                    count: 1,
                    pattern: vec_into!["abc", "m", "n", "q"]
                },
            ]
        );
    }
}

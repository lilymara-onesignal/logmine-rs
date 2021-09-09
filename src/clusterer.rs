use std::{borrow::Cow, fmt};

use crate::{
    patterns::{self, PatternElement},
    scoring,
};

#[derive(Clone)]
pub struct Clusterer {
    clusters: Vec<Cluster<'static>>,
    pub max_dist: f64,
    min_members: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cluster<'a> {
    pub representative: Vec<PatternElement<'a>>,
    pub count: u32,
    pub pattern: Vec<PatternElement<'a>>,
}

impl<'a> fmt::Display for Cluster<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.count)?;

        for element in &self.pattern {
            match element {
                PatternElement::Text(t) => write!(f, "{} ", t)?,
                PatternElement::Placeholder => write!(f, "--- ")?,
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
            min_members: 1,
        }
    }

    pub fn with_max_dist(mut self, max_dist: f64) -> Self {
        self.max_dist = max_dist;
        self
    }

    pub fn with_min_members(mut self, min_members: u32) -> Self {
        self.min_members = min_members;
        self
    }

    pub fn process_line(&mut self, line: &str) {
        let pattern: Vec<PatternElement> = line
            .split(" ")
            .map(|t| PatternElement::Text(Cow::Borrowed(t)))
            .collect();

        for cluster in &mut self.clusters {
            let score = scoring::distance(&cluster.representative, &pattern, self.max_dist);

            if score <= self.max_dist {
                cluster.count += 1;
                let old_pattern = std::mem::take(&mut cluster.pattern);

                cluster.pattern = patterns::create(old_pattern, pattern);

                return;
            }
        }

        let pattern: Vec<PatternElement<'static>> = pattern
            .into_iter()
            .map(|element| match element {
                PatternElement::Placeholder => PatternElement::Placeholder,
                PatternElement::Text(t) => PatternElement::Text(Cow::Owned(t.into_owned())),
            })
            .collect();

        self.clusters.push(Cluster {
            representative: pattern.clone(),
            count: 1,
            pattern,
        });
    }

    pub fn result(self) -> Vec<Cluster<'static>> {
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
}

#[cfg(test)]
mod test {
    use crate::patterns::PatternElement;

    use super::{Cluster, Clusterer};

    impl Clusterer {
        fn find(mut self, input_lines: &[&str]) -> Vec<Cluster<'static>> {
            for line in input_lines {
                self.process_line(line);
            }
            self.result()
        }
    }

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

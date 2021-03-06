use std::{borrow::Cow, fmt};

use regex::Regex;

use crate::{
    pattern::{Pattern, PatternElement},
    scoring,
};

#[derive(Clone, Copy)]
pub struct ClustererOptions {
    pub max_dist: f64,
    pub min_members: u32,
}

pub struct Clusterer {
    clusters: Vec<Cluster<'static>>,
    options: ClustererOptions,
    pattern_backing_storage: Pattern<'static>,
    split_regex: Regex,
}

#[derive(Debug, PartialEq)]
pub struct Cluster<'a> {
    pub representative: Pattern<'a>,
    pub count: u32,
    pub pattern: Pattern<'a>,
}

impl<'a> fmt::Display for Cluster<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.count)?;

        for element in self.pattern.iter() {
            match element {
                PatternElement::Text(t) => write!(f, "{} ", t)?,
                PatternElement::Placeholder => write!(f, "--- ")?,
            }
        }

        Ok(())
    }
}

impl ClustererOptions {
    pub fn with_max_dist(mut self, max_dist: f64) -> Self {
        self.max_dist = max_dist;
        self
    }

    pub fn with_min_members(mut self, min_members: u32) -> Self {
        self.min_members = min_members;
        self
    }
}

impl Default for ClustererOptions {
    fn default() -> Self {
        Self {
            max_dist: 0.01,
            min_members: 1,
        }
    }
}

impl Clusterer {
    pub fn new(options: ClustererOptions, split_regex: Regex) -> Self {
        Self {
            options,
            split_regex,
            clusters: Default::default(),
            pattern_backing_storage: Default::default(),
        }
    }

    pub fn process_line(&mut self, line: &str) {
        let mut pattern = std::mem::take(&mut self.pattern_backing_storage).clear_and_reinterpret();
        for t in self.split_regex.split(line) {
            pattern.push_text(t);
        }

        for cluster in &mut self.clusters {
            let score = scoring::distance(&cluster.representative, &pattern, self.options.max_dist);

            if score <= self.options.max_dist {
                cluster.count += 1;
                let mut old_pattern = std::mem::take(&mut cluster.pattern);

                cluster.pattern = old_pattern.merge(pattern);

                self.pattern_backing_storage = old_pattern;

                return;
            }
        }

        let mut old_pattern = pattern;

        let pattern = Pattern::new(
            old_pattern
                .drain()
                .map(|element| match element {
                    PatternElement::Placeholder => PatternElement::Placeholder,
                    PatternElement::Text(t) => PatternElement::Text(Cow::Owned(t.into_owned())),
                })
                .collect(),
        );
        self.pattern_backing_storage = old_pattern.clear_and_reinterpret();

        self.clusters.push(Cluster {
            representative: pattern.clone(),
            count: 1,
            pattern,
        });
    }

    pub fn take_result(&mut self) -> impl Iterator<Item = Cluster<'static>> {
        let clusters = std::mem::take(&mut self.clusters);

        let min_members = self.options.min_members;

        clusters.into_iter().filter(move |c| c.count >= min_members)
    }
}

#[cfg(test)]
mod test {
    use regex::Regex;

    use crate::{
        clusterer::ClustererOptions,
        pattern::{Pattern, PatternElement},
    };

    use super::{Cluster, Clusterer};

    impl Clusterer {
        fn find(mut self, input_lines: &[&str]) -> Vec<Cluster<'static>> {
            for line in input_lines {
                self.process_line(line);
            }
            self.take_result().collect()
        }
    }

    #[test]
    fn test() {
        let clusters = Clusterer::new(
            ClustererOptions {
                max_dist: 0.5,
                ..Default::default()
            },
            Regex::new("\\s+").unwrap(),
        )
        .find(&["hello 1 y 3", "hello 1 x 3", "abc m n q"]);

        assert_eq!(
            clusters,
            vec![
                Cluster {
                    representative: Pattern::new(vec_into!["hello", "1", "y", "3"]),
                    count: 2,
                    pattern: Pattern::new(vec_into![
                        "hello",
                        "1",
                        PatternElement::Placeholder,
                        "3"
                    ])
                },
                Cluster {
                    representative: Pattern::new(vec_into!["abc", "m", "n", "q"]),
                    count: 1,
                    pattern: Pattern::new(vec_into!["abc", "m", "n", "q"])
                },
            ]
        );
    }

    #[test]
    fn test_min_members() {
        let clusters = Clusterer::new(
            ClustererOptions {
                max_dist: 0.5,
                min_members: 2,
            },
            Regex::new("\\s+").unwrap(),
        )
        .find(&["hello 1 y 3", "hello 1 x 3", "abc m n q"]);

        assert_eq!(
            clusters,
            vec![Cluster {
                representative: Pattern::new(vec_into!["hello", "1", "y", "3"]),
                count: 2,
                pattern: Pattern::new(vec_into!["hello", "1", PatternElement::Placeholder, "3"])
            }]
        );
    }

    #[test]
    fn test_small_max_dist() {
        let clusters = Clusterer::new(
            ClustererOptions {
                max_dist: 0.01,
                ..Default::default()
            },
            Regex::new("\\s+").unwrap(),
        )
        .find(&["hello 1 y 3", "hello 1 x 3", "abc m n q"]);

        assert_eq!(
            clusters,
            vec![
                Cluster {
                    representative: Pattern::new(vec_into!["hello", "1", "y", "3"]),
                    count: 1,
                    pattern: Pattern::new(vec_into!["hello", "1", "y", "3"])
                },
                Cluster {
                    representative: Pattern::new(vec_into!["hello", "1", "x", "3"]),
                    count: 1,
                    pattern: Pattern::new(vec_into!["hello", "1", "x", "3"])
                },
                Cluster {
                    representative: Pattern::new(vec_into!["abc", "m", "n", "q"]),
                    count: 1,
                    pattern: Pattern::new(vec_into!["abc", "m", "n", "q"])
                },
            ]
        );
    }
}

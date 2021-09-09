pub struct Scorer {}

impl Scorer {
    pub fn new() -> Self {
        Self {}
    }

    pub(crate) fn distance(&self, fields1: &[String], fields2: &[String], max_dist: f64) -> f64 {
        let max_len = fields1.len().max(fields2.len()) as f64;

        let mut total = 0.0;
        for (f1, f2) in fields1.iter().zip(fields2.iter()) {
            total += self.score(f1, f2) / max_len;

            if (1.0 - total) < max_dist {
                return 1.0 - total;
            }
        }

        1.0 - total
    }

    fn score(&self, f1: &str, f2: &str) -> f64 {
        if f1 == f2 {
            1.0
        } else {
            0.0
        }
    }
}

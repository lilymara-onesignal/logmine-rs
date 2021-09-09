use crate::pattern::{Pattern, PatternElement};

pub(crate) fn distance(fields1: &Pattern, fields2: &Pattern, max_dist: f64) -> f64 {
    let max_len = fields1.len().max(fields2.len()) as f64;

    let mut total = 0.0;
    for (f1, f2) in fields1.iter().zip(fields2.iter()) {
        total += score(f1, f2) / max_len;

        if (1.0 - total) < max_dist {
            return 1.0 - total;
        }
    }

    1.0 - total
}

fn score(f1: &PatternElement, f2: &PatternElement) -> f64 {
    if f1 == f2 {
        1.0
    } else {
        0.0
    }
}

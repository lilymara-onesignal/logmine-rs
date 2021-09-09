use std::borrow::Cow;

use seal::pair::{AlignmentSet, InMemoryAlignmentMatrix, SmithWaterman, Step};

#[derive(Debug, PartialEq, Clone)]
pub enum PatternElement<'a> {
    Text(Cow<'a, str>),
    Placeholder,
}

// p1 and p2 are owned vecs here instead of an immutable borrows so that we
// can re-use heap space and cut down on allocations/clones
pub(crate) fn create(
    p1: Vec<PatternElement<'static>>,
    p2: Vec<PatternElement<'_>>,
) -> Vec<PatternElement<'static>> {
    if p1.is_empty() && p2.is_empty() {
        return Vec::new();
    }

    let strategy = SmithWaterman::new(10, -1, 0, 0);

    let aligner = AlignmentSet::<InMemoryAlignmentMatrix>::new(
        p1.len(),
        p2.len(),
        strategy,
        |p1_idx, p2_idx| p1[p1_idx] == p2[p2_idx],
    )
    .unwrap();

    let mut p2 = p2;
    p2.clear();

    // Safety ----- p2 lacks the 'static lifetime in the function input
    // params because it holds non-owned `PatternElement` values. When p2 is
    // returned from this function, all of the values within it come from
    // p1, which does have the 'static lifetime. That's why this is safe. At
    // this point in the fn, p2 has just been cleared, so none of the values
    // within it are still live.
    let mut out_pattern = unsafe {
        fn assert_static<T: 'static>(_x: &T) {}
        assert_static(&p1);

        std::mem::transmute::<Vec<PatternElement<'_>>, Vec<PatternElement<'static>>>(p2)
    };

    let mut in_pattern = p1;

    let mut just_inserted_placeholder = false;
    for s in aligner.global_alignment().steps() {
        match s {
            Step::Align { x, .. } => {
                let element = std::mem::replace(&mut in_pattern[x], PatternElement::Placeholder);

                out_pattern.push(element);
                just_inserted_placeholder = false;
            }
            Step::Delete { .. } | Step::Insert { .. } => {
                if !just_inserted_placeholder {
                    out_pattern.push(PatternElement::Placeholder);
                    just_inserted_placeholder = true;
                }
            }
        }
    }

    out_pattern
}

impl<'a> From<&'a str> for PatternElement<'a> {
    fn from(s: &'a str) -> Self {
        Self::Text(Cow::Borrowed(s))
    }
}

#[cfg(test)]
mod tests {
    use super::{create, PatternElement};

    #[test]
    fn test() {
        assert_eq!(
            create(vec_into!["a", "b"], vec_into!["a", "c"]),
            vec_into!["a", PatternElement::Placeholder],
        );
    }

    #[test]
    fn test_with_single_gap() {
        assert_eq!(
            create(vec_into!["a", "c", "b"], vec_into!["a", "b"]),
            vec_into!["a", PatternElement::Placeholder, "b"],
        );
    }

    #[test]
    fn test_with_multiple_gaps() {
        assert_eq!(
            create(vec_into!["a", "c", "b"], vec_into!["a", "b", "d"]),
            vec_into![
                "a",
                PatternElement::Placeholder,
                "b",
                PatternElement::Placeholder,
            ],
        );
    }

    #[test]
    fn test_with_gaps_in_the_middle() {
        assert_eq!(
            create(
                vec_into!["a", "b", "d", "e", "f"],
                vec_into!["a", "b", "c", "e", "f"]
            ),
            vec_into!["a", "b", PatternElement::Placeholder, "e", "f",],
        );
    }

    #[test]
    #[ignore]
    fn test_with_multiple_gaps_next_to_each_other() {
        assert_eq!(
            create(vec_into!["a", "b", "c", "d"], vec_into!["a", "d"]),
            vec_into![
                "a",
                PatternElement::Placeholder,
                PatternElement::Placeholder,
                "d",
            ],
        );
    }
}

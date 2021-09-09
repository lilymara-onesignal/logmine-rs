use std::borrow::Cow;

use seal::pair::{AlignmentSet, InMemoryAlignmentMatrix, SmithWaterman, Step};

#[derive(Debug, PartialEq, Clone)]
pub enum PatternElement<'a> {
    Text(Cow<'a, str>),
    Placeholder,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Pattern<'a> {
    items: Vec<PatternElement<'a>>,
}

impl<'a> Pattern<'a> {
    pub fn new(items: Vec<PatternElement<'a>>) -> Self {
        Self { items }
    }

    pub fn iter(&self) -> impl Iterator<Item = &PatternElement<'a>> {
        self.items.iter()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    fn reuse_for_static(mut self) -> Pattern<'static> {
        let mut items = std::mem::take(&mut self.items);
        items.clear();

        // Safety ----- it is acceptable to re-use vector heap space here since we ensure to clear the vector of any non-'static items before running the transmute.
        let static_items = unsafe {
            std::mem::transmute::<Vec<PatternElement<'_>>, Vec<PatternElement<'static>>>(items)
        };

        Pattern {
            items: static_items,
        }
    }

    pub fn into_iter(self) -> impl Iterator<Item = PatternElement<'a>> {
        self.items.into_iter()
    }
}

// p1 and p2 are owned vecs here instead of an immutable borrows so that we
// can re-use heap space and cut down on allocations/clones
pub(crate) fn create(p1: Pattern<'static>, p2: Pattern<'_>) -> Pattern<'static> {
    if p1.items.is_empty() && p2.items.is_empty() {
        return Pattern::default();
    }

    let strategy = SmithWaterman::new(10, -1, 0, 0);

    let aligner = AlignmentSet::<InMemoryAlignmentMatrix>::new(
        p1.items.len(),
        p2.items.len(),
        strategy,
        |p1_idx, p2_idx| p1.items[p1_idx] == p2.items[p2_idx],
    )
    .unwrap();

    let mut in_pattern = p1;
    let mut out_pattern = p2.reuse_for_static();

    let mut just_inserted_placeholder = false;
    for s in aligner.global_alignment().steps() {
        match s {
            Step::Align { x, .. } => {
                let element =
                    std::mem::replace(&mut in_pattern.items[x], PatternElement::Placeholder);

                out_pattern.items.push(element);
                just_inserted_placeholder = false;
            }
            Step::Delete { .. } | Step::Insert { .. } => {
                if !just_inserted_placeholder {
                    out_pattern.items.push(PatternElement::Placeholder);
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
    use super::{create, Pattern, PatternElement};

    #[test]
    fn test() {
        assert_eq!(
            create(
                Pattern::new(vec_into!["a", "b"]),
                Pattern::new(vec_into!["a", "c"])
            ),
            Pattern::new(vec_into!["a", PatternElement::Placeholder]),
        );
    }

    #[test]
    fn test_with_single_gap() {
        assert_eq!(
            create(
                Pattern::new(vec_into!["a", "c", "b"]),
                Pattern::new(vec_into!["a", "b"])
            ),
            Pattern::new(vec_into!["a", PatternElement::Placeholder, "b"]),
        );
    }

    #[test]
    fn test_with_multiple_gaps() {
        assert_eq!(
            create(
                Pattern::new(vec_into!["a", "c", "b"]),
                Pattern::new(vec_into!["a", "b", "d"])
            ),
            Pattern::new(vec_into![
                "a",
                PatternElement::Placeholder,
                "b",
                PatternElement::Placeholder,
            ]),
        );
    }

    #[test]
    fn test_with_gaps_in_the_middle() {
        assert_eq!(
            create(
                Pattern::new(vec_into!["a", "b", "d", "e", "f"]),
                Pattern::new(vec_into!["a", "b", "c", "e", "f"]),
            ),
            Pattern::new(vec_into!["a", "b", PatternElement::Placeholder, "e", "f",]),
        );
    }

    #[test]
    #[ignore]
    fn test_with_multiple_gaps_next_to_each_other() {
        assert_eq!(
            create(
                Pattern::new(vec_into!["a", "b", "c", "d"]),
                Pattern::new(vec_into!["a", "d"])
            ),
            Pattern::new(vec_into![
                "a",
                PatternElement::Placeholder,
                PatternElement::Placeholder,
                "d",
            ]),
        );
    }
}

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

    /// Reinterpret the vector heap space used by this pattern (which may not
    /// necessarily store 'static items) as a vector which can only store
    /// 'static items. This will clear out all the items from this Pattern and
    /// give you a new empty pattern whose vector already has some scratch space
    /// to work with on it (assuming this pattern isn't empty).
    fn reuse_for_static(mut self) -> Pattern<'static> {
        let mut items = std::mem::take(&mut self.items);
        items.clear();

        // Safety ----- it is acceptable to re-use vector heap space here since
        // we ensure to clear the vector of any non-'static items before running
        // the transmute.
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

impl Pattern<'static> {
    pub fn merge(self, other: Pattern<'_>) -> Pattern<'static> {
        if self.items.is_empty() && other.items.is_empty() {
            return Pattern::default();
        }

        let strategy = SmithWaterman::new(10, -1, 0, 0);

        let aligner = AlignmentSet::<InMemoryAlignmentMatrix>::new(
            self.items.len(),
            other.items.len(),
            strategy,
            |p1_idx, p2_idx| self.items[p1_idx] == other.items[p2_idx],
        )
        .unwrap();

        let mut in_pattern = self;
        let mut out_pattern = other.reuse_for_static();

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
}

impl<'a> From<&'a str> for PatternElement<'a> {
    fn from(s: &'a str) -> Self {
        Self::Text(Cow::Borrowed(s))
    }
}

#[cfg(test)]
mod tests {
    use super::{Pattern, PatternElement};

    #[test]
    fn test() {
        assert_eq!(
            Pattern::new(vec_into!["a", "b"]).merge(Pattern::new(vec_into!["a", "c"])),
            Pattern::new(vec_into!["a", PatternElement::Placeholder]),
        );
    }

    #[test]
    fn test_with_single_gap() {
        assert_eq!(
            Pattern::new(vec_into!["a", "c", "b"]).merge(Pattern::new(vec_into!["a", "b"])),
            Pattern::new(vec_into!["a", PatternElement::Placeholder, "b"]),
        );
    }

    #[test]
    fn test_with_multiple_gaps() {
        assert_eq!(
            Pattern::new(vec_into!["a", "c", "b"]).merge(Pattern::new(vec_into!["a", "b", "d"])),
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
            Pattern::new(vec_into!["a", "b", "d", "e", "f"])
                .merge(Pattern::new(vec_into!["a", "b", "c", "e", "f"])),
            Pattern::new(vec_into!["a", "b", PatternElement::Placeholder, "e", "f",]),
        );
    }

    #[test]
    #[ignore]
    fn test_with_multiple_gaps_next_to_each_other() {
        assert_eq!(
            Pattern::new(vec_into!["a", "b", "c", "d"]).merge(Pattern::new(vec_into!["a", "d"])),
            Pattern::new(vec_into![
                "a",
                PatternElement::Placeholder,
                PatternElement::Placeholder,
                "d",
            ]),
        );
    }
}

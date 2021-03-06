use std::borrow::Cow;

use seal::pair::{AlignmentSet, InMemoryAlignmentMatrix, SmithWaterman, Step};

#[derive(Debug, PartialEq, Clone)]
pub enum PatternElement<'a> {
    Text(Cow<'a, str>),
    Placeholder,
}

#[cfg(feature = "small-vec")]
type Storage<'a> = smallvec::SmallVec<[PatternElement<'a>; 5]>;
#[cfg(not(feature = "small-vec"))]
type Storage<'a> = Vec<PatternElement<'a>>;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Pattern<'a> {
    items: Storage<'a>,
}

impl<'a> Pattern<'a> {
    pub fn new(items: Storage<'a>) -> Self {
        Self { items }
    }

    pub fn iter(&self) -> impl Iterator<Item = &PatternElement<'a>> {
        self.items.iter()
    }

    pub fn drain<'b>(&'b mut self) -> impl 'b + Iterator<Item = PatternElement<'a>> {
        self.items.drain(..)
    }

    pub fn push_text(&mut self, item: impl Into<Cow<'a, str>>) -> &mut Self {
        self.items.push(PatternElement::Text(item.into()));
        self
    }

    pub fn push_placeholder(&mut self) -> &mut Self {
        self.items.push(PatternElement::Placeholder);
        self
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Allow the heap space occupied by this pattern to be re-interpreted to
    /// store items of a different lifetime.
    pub fn clear_and_reinterpret<'b>(mut self) -> Pattern<'b> {
        let mut items = std::mem::take(&mut self.items);
        items.clear();

        // Safety ----- it is acceptable to re-use vector heap space here since
        // we ensure to clear the vector of any non-'static items before running
        // the transmute.
        let static_items = unsafe { std::mem::transmute::<Storage<'a>, Storage<'b>>(items) };

        Pattern {
            items: static_items,
        }
    }
}

impl<'a> IntoIterator for Pattern<'a> {
    type Item = PatternElement<'a>;
    type IntoIter = <Vec<PatternElement<'a>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl Pattern<'static> {
    pub fn merge(&mut self, other: Pattern<'_>) -> Pattern<'static> {
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

        let in_pattern = self;
        let mut out_pattern = other.clear_and_reinterpret::<'static>();

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

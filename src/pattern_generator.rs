use seal::pair::{AlignmentSet, InMemoryAlignmentMatrix, SmithWaterman, Step};

pub struct PatternGenerator {}

#[derive(Debug, PartialEq, Clone)]
pub enum PatternElement {
    Text(String),
    Placeholder,
}

impl PatternGenerator {
    pub fn new() -> Self {
        Self {}
    }

    // p1 and p2 are owned vecs here instead of an immutable borrows so that we
    // can re-use heap space and cut down on allocations/clones
    pub(crate) fn create_pattern(
        &self,
        p1: Vec<PatternElement>,
        p2: Vec<PatternElement>,
    ) -> Vec<PatternElement> {
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

        let mut in_pattern = p1;

        let mut out_pattern = p2;
        out_pattern.clear();

        let mut just_inserted_placeholder = false;
        for s in aligner.global_alignment().steps() {
            match s {
                Step::Align { x, .. } => {
                    let element =
                        std::mem::replace(&mut in_pattern[x], PatternElement::Placeholder);

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
}

impl From<&str> for PatternElement {
    fn from(s: &str) -> Self {
        Self::Text(s.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::{PatternElement, PatternGenerator};

    #[test]
    fn test() {
        let generator = PatternGenerator::new();

        assert_eq!(
            generator.create_pattern(vec_into!["a", "b"], vec_into!["a", "c"]),
            vec_into!["a", PatternElement::Placeholder],
        );
    }

    #[test]
    fn test_with_single_gap() {
        let generator = PatternGenerator::new();

        assert_eq!(
            generator.create_pattern(vec_into!["a", "c", "b"], vec_into!["a", "b"]),
            vec_into!["a", PatternElement::Placeholder, "b"],
        );
    }

    #[test]
    fn test_with_multiple_gaps() {
        let generator = PatternGenerator::new();

        assert_eq!(
            generator.create_pattern(vec_into!["a", "c", "b"], vec_into!["a", "b", "d"]),
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
        let generator = PatternGenerator::new();

        assert_eq!(
            generator.create_pattern(
                vec_into!["a", "b", "d", "e", "f"],
                vec_into!["a", "b", "c", "e", "f"]
            ),
            vec_into!["a", "b", PatternElement::Placeholder, "e", "f",],
        );
    }

    #[test]
    #[ignore]
    fn test_with_multiple_gaps_next_to_each_other() {
        let generator = PatternGenerator::new();

        assert_eq!(
            generator.create_pattern(vec_into!["a", "b", "c", "d"], vec_into!["a", "d"]),
            vec_into![
                "a",
                PatternElement::Placeholder,
                PatternElement::Placeholder,
                "d",
            ],
        );
    }
}
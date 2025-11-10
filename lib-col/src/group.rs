pub const GROUP_COUNT: usize = 32;

/// A group can be thought of an encoding of a set.
/// It encodes what set a collider belongs to.
/// It is built using bitflags on u32, most methods are `const fn`, so
/// most operations should be quite fast.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Deserialize, serde::Serialize)]
#[repr(transparent)]
pub struct Group(pub u32);

impl Group {
    /// Empty group.
    pub const fn empty() -> Group {
        Group(0)
    }

    /// Returns `true` if the group is empty.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Constructs a group from its index.
    /// Essentially, a singleton.
    pub const fn from_id(x: u32) -> Group {
        Group(1u32.unbounded_shl(x))
    }

    /// Add the groups together. Essentially, a set union.
    pub const fn union(self, other: Group) -> Group {
        Group(self.0 | other.0)
    }

    /// Get the most common group of two groups.
    /// Essentially, a set intersection.
    pub const fn intersection(self, other: Group) -> Group {
        Group(self.0 & other.0)
    }

    /// Check if the group contains `idx`.
    /// Essentially, a set membership check.
    pub const fn contains(self, idx: u32) -> bool {
        self.includes(Group::from_id(idx))
    }

    /// Check if `self` is included in `target`.
    /// Essentially, a subset check.
    pub const fn includes(self, target: Group) -> bool {
        self.0 & target.0 == target.0
    }
}

#[cfg(test)]
mod tests {
    const GROUP_SAMPLE_COUNT: usize = 1_000_000;

    use super::{GROUP_COUNT, Group};

    use rand::random_range;

    #[test]
    fn identities() {
        for _ in 0..GROUP_SAMPLE_COUNT {
            let x = Group(random_range(0..std::u32::MAX));
            assert!(x.includes(x));
            assert_eq!(x.union(x), x);
            assert_eq!(x.intersection(x), x);
        }
    }

    #[test]
    fn empty_is_empty() {
        assert!(Group::empty().is_empty());
    }

    #[test]
    fn is_empty_included() {
        assert!(Group::empty().is_empty());
        for _ in 0..GROUP_SAMPLE_COUNT {
            let x = random_range(0..std::u32::MAX);
            if x == 0 {
                continue;
            }
            assert!(
                Group(x).includes(Group::empty()),
                "{x:b} must include empty"
            );
        }
    }

    #[test]
    fn from_id_contains() {
        for idx in 0..GROUP_COUNT {
            let idx = idx as u32;
            let g = Group::from_id(idx);
            assert!(g.contains(idx), "{:b} must contain {idx}", g.0);
        }
    }

    #[test]
    fn from_id_disjoint() {
        for idx in 0..GROUP_COUNT {
            let g = Group::from_id(idx as u32);
            for other_idx in 0..GROUP_COUNT {
                if other_idx == idx {
                    continue;
                }
                let other_g = Group::from_id(other_idx as u32);
                assert_eq!(g.intersection(other_g), Group::empty());
            }
        }
    }

    #[test]
    fn union_includes() {
        for _ in 0..GROUP_SAMPLE_COUNT {
            let x = Group(random_range(0..std::u32::MAX));
            let y = Group(random_range(0..std::u32::MAX));
            let union = x.union(y);
            assert!(union.includes(x), "{:b} must include {:b}", union.0, x.0);
            assert!(union.includes(y), "{:b} must include {:b}", union.0, y.0);
        }
    }

    #[test]
    fn intersection_includes() {
        for _ in 0..GROUP_SAMPLE_COUNT {
            let x = Group(random_range(0..std::u32::MAX));
            let y = Group(random_range(0..std::u32::MAX));
            let intersection = x.intersection(y);
            assert!(
                x.includes(intersection),
                "{:b} must include {:b}",
                x.0,
                intersection.0
            );
            assert!(
                y.includes(intersection),
                "{:b} must include {:b}",
                y.0,
                intersection.0
            );
        }
    }

    #[test]
    fn reassemble() {
        for _ in 0..GROUP_SAMPLE_COUNT {
            let sample = Group(random_range(0..std::u32::MAX));
            let mut built = Group::empty();
            for idx in 0..GROUP_COUNT {
                let idx = idx as u32;
                let bit = Group::from_id(idx);
                if !sample.includes(bit) {
                    continue;
                }
                built = built.union(bit);
            }
            assert_eq!(sample, built);
        }
    }
}

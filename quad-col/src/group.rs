pub const GROUP_COUNT: usize = 32;

/// A group can be thought of an encoding of a set.
/// It encodes what set a collider belongs to.
/// It is built using bitflags on u32, most methods are `const fn`, so
/// most operations should be quite fast.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

use macroquad::prelude::*;
use rapier2d::prelude::InteractionGroups;
use shipyard::{Component, EntityId};

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PhysicsGroup {
    pub level: bool,
    pub npcs: bool,
    pub player: bool,
    pub projectiles: bool,
    pub maincell: bool,
    pub items: bool,
}

impl PhysicsGroup {
    pub const fn empty() -> PhysicsGroup {
        PhysicsGroup {
            level: false,
            npcs: false,
            player: false,
            projectiles: false,
            maincell: false,
            items: false,
        }
    }
}

impl PhysicsGroup {
    fn into_group(self) -> rapier2d::prelude::Group {
        use rapier2d::prelude::*;

        pub const LEVEL: Group = Group::GROUP_1;
        pub const NPCS: Group = Group::GROUP_2;
        pub const PLAYER: Group = Group::GROUP_3;
        pub const PROJECTILES: Group = Group::GROUP_4;
        pub const MAINCELL: Group = Group::GROUP_5;
        pub const ITEMS: Group = Group::GROUP_6;

        let mut filter = Group::NONE;
        if self.level {
            filter |= LEVEL;
        }
        if self.npcs {
            filter |= NPCS;
        }
        if self.player {
            filter |= PLAYER;
        }
        if self.projectiles {
            filter |= PROJECTILES;
        }
        if self.maincell {
            filter |= MAINCELL
        }
        if self.items {
            filter |= ITEMS
        }

        filter
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PhysicsFilter(pub PhysicsGroup, pub PhysicsGroup);

impl PhysicsFilter {
    pub(crate) fn into_interaction_groups(self) -> InteractionGroups {
        InteractionGroups {
            memberships: self.0.into_group(),
            filter: self.1.into_group(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ColliderTy {
    Box { width: f32, height: f32 },
    Circle { radius: f32 },
}

#[derive(Clone, Debug, Component)]
pub struct BeamTag {
    pub width: f32,
    pub length: f32,
    pub cast_filter: PhysicsFilter,
    pub overlap_filter: PhysicsFilter,
    pub overlaps: Vec<EntityId>,
}

impl BeamTag {
    pub fn new(overlap_filter: PhysicsFilter, cast_filter: PhysicsFilter, width: f32) -> Self {
        Self {
            width,
            length: 0.0f32,
            cast_filter,
            overlap_filter,
            overlaps: Vec::with_capacity(32),
        }
    }
}

#[derive(Clone, Debug, Component)]
pub struct OneSensorTag {
    pub shape: ColliderTy,
    pub groups: PhysicsFilter,
    pub col: Option<EntityId>,
}

impl OneSensorTag {
    pub fn new(shape: ColliderTy, groups: PhysicsFilter) -> Self {
        Self {
            shape,
            groups,
            col: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Component)]
pub struct KinematicControl {
    pub dr: Vec2,
    pub slide: bool,
}

impl KinematicControl {
    pub fn new() -> Self {
        Self {
            dr: Vec2::ZERO,
            slide: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Component)]
pub struct VelocityProxy(pub Vec2);

#[derive(Clone, Copy, Debug, Component)]
pub struct ForceApplier {
    pub force: Vec2,
}

#[derive(Clone, Copy, Debug, Component)]
pub struct ImpulseApplier {
    pub impulse: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BodyKind {
    Static,
    Kinematic,
}

#[derive(Clone, Copy, Debug, Component)]
#[track(Deletion, Removal, Insertion)]
pub struct BodyTag {
    pub enabled: bool,
    pub groups: PhysicsFilter,
    pub(crate) shape: ColliderTy,
    pub(crate) mass: f32,
    pub(crate) kind: BodyKind,
}

impl BodyTag {
    pub fn new(
        groups: PhysicsFilter,
        shape: ColliderTy,
        mass: f32,
        enabled: bool,
        kind: BodyKind,
    ) -> Self {
        Self {
            enabled,
            groups,
            mass,
            shape,
            kind,
        }
    }

    pub fn shape(&self) -> &ColliderTy {
        &self.shape
    }

    pub fn kind(&self) -> BodyKind {
        self.kind
    }
}

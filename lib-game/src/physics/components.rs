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

    pub(crate) fn into_interaction_groups(self) -> InteractionGroups {
        InteractionGroups { 
            memberships: self.into_group(), 
            filter: self.into_group(), 
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ColliderTy {
    Box { width: f32, height: f32 },
    Circle { radius: f32 },
}

#[derive(Clone, Debug, Component)]
pub struct OneSensorTag {
    pub shape: ColliderTy,
    pub groups: PhysicsGroup,
    pub col: Option<EntityId>,
}

impl OneSensorTag {
    pub fn new(shape: ColliderTy, groups: PhysicsGroup) -> Self {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BodyKind {
    Static,
    Kinematic,
}

#[derive(Clone, Copy, Debug, Component)]
#[track(Deletion, Removal, Insertion)]
pub struct BodyTag {
    pub enabled: bool,
    pub groups: PhysicsGroup,
    pub(crate) shape: ColliderTy,
    pub(crate) mass: f32,
    pub(crate) kind: BodyKind,
}

impl BodyTag {
    pub fn new(
        groups: PhysicsGroup,
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

use std::{marker::PhantomData, path::PathBuf};

use anyhow::Context;
use hashbrown::HashMap;
use hecs::{Component, DynamicBundleClone, EntityBuilderClone};
use serde::{Deserialize, de::DeserializeOwned};

#[derive(Deserialize)]
#[serde(transparent)]
pub struct PrePrefab<'a>(#[serde(borrow)] pub HashMap<&'a str, &'a serde_json::value::RawValue>);

pub struct PrefabFactory<T> {
    registry: HashMap<String, ComponentEntry<T>>,
    _phantom: PhantomData<fn(&mut T)>,
}

impl<T> PrefabFactory<T> {
    pub fn new() -> Self {
        PrefabFactory { registry: HashMap::new(), _phantom: PhantomData }
    }

    pub fn register_bundle<B: DeserializeOwned + Clone + DynamicBundleClone>(&mut self, key: &str) {
        self.register::<B>(
            key,
            |_, builder, x| {
                builder.add_bundle(x);
                Ok(())
            },
            |_| Ok(Vec::new()),
        );
    }

    pub fn register_component<C: DeserializeOwned + Clone + Component>(&mut self, key: &str) {
        self.register::<C>(
            key,
            |_, builder, x| {
                builder.add(x);
                Ok(())
            },
            |_| Ok(Vec::new()),
        );
    }

    pub fn register_component_with_constructor_ctx<Seed: DeserializeOwned, C: Clone + Component>(
        &mut self,
        key: &str,
        constructor: impl Fn(Seed, &mut T) -> anyhow::Result<C> + 'static,
        deps: impl Fn(&serde_json::value::RawValue) -> anyhow::Result<Vec<PathBuf>> + 'static,
    ) {
        self.register::<Seed>(
            key,
            move |ctx, builder, x| {
                builder.add(constructor(x, ctx)?);
                Ok(())
            },
            deps,
        );
    }

    pub fn register_component_with_constructor<Seed: DeserializeOwned, C: Clone + Component>(
        &mut self,
        key: &str,
        constructor: impl Fn(Seed) -> C + 'static,
    ) {
        self.register::<Seed>(
            key,
            move |_, builder: &mut EntityBuilderClone, x: Seed| {
                builder.add(constructor(x));
                Ok(())
            },
            |_| Ok(Vec::new()),
        );
    }

    pub fn register<V: DeserializeOwned>(
        &mut self,
        key: &str,
        insert: impl Fn(&mut T, &mut EntityBuilderClone, V) -> anyhow::Result<()> + 'static,
        deps: impl Fn(&serde_json::value::RawValue) -> anyhow::Result<Vec<PathBuf>> + 'static,
    ) {
        if self.registry.contains_key(key) {
            panic!("duplicate key {key:?}");
        }

        let entry = ComponentEntry {
            deps: Box::new(deps),
            builder: Box::new(move |ctx, builder, val| {
                let val = serde_json::from_str::<V>(val.get())?;
                insert(ctx, builder, val)?;
                Ok(())
            }),
        };

        self.registry.insert(key.to_string(), entry);
    }

    pub fn list_deps(&self, pref: &PrePrefab) -> anyhow::Result<Vec<PathBuf>> {
        let mut result = Vec::new();
        for (name, value) in pref.0.iter() {
            let Some(entry) = self.registry.get(*name) else {
                anyhow::bail!("unknown component: {name:?}");
            };
            let deps = (entry.deps)(value).with_context(|| format!("deps of {name:?}"))?;
            result.extend(deps);
        }

        Ok(result)
    }

    pub fn build(
        &self,
        ctx: &mut T,
        start: &mut EntityBuilderClone,
        pref: &PrePrefab,
    ) -> anyhow::Result<()> {
        for (name, value) in pref.0.iter() {
            let Some(entry) = self.registry.get(*name) else {
                anyhow::bail!("unknown component: {name:?}");
            };
            tracing::info!(entry = name, "build prefab component");
            (entry.builder)(ctx, start, value).with_context(|| format!("build {name:?}"))?;
        }
        Ok(())
    }
}

impl<T> Default for PrefabFactory<T> {
    fn default() -> Self {
        Self::new()
    }
}

struct ComponentEntry<T> {
    deps: ComponentDependencies,
    builder: ComponentBuilder<T>,
}

type ComponentDependencies =
    Box<dyn Fn(&serde_json::value::RawValue) -> anyhow::Result<Vec<PathBuf>>>;

type ComponentBuilder<T> = Box<
    dyn Fn(&mut T, &mut EntityBuilderClone, &serde_json::value::RawValue) -> anyhow::Result<()>,
>;

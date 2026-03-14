use std::marker::PhantomData;

use anyhow::Context;
use hashbrown::HashMap;
use hecs::{Component, DynamicBundleClone, EntityBuilderClone};
use serde::{Deserialize, de::DeserializeOwned};

#[derive(Deserialize)]
#[serde(transparent)]
pub struct PrePrefab<'a>(#[serde(borrow)] pub HashMap<&'a str, &'a serde_json::value::RawValue>);

pub struct PrefabFactory<T> {
    registry: HashMap<String, ComponentBuilder<T>>,
    _phantom: PhantomData<fn(&mut T)>,
}

impl<T> PrefabFactory<T> {
    pub fn new() -> Self {
        PrefabFactory {
            registry: HashMap::new(),
            _phantom: PhantomData,
        }
    }

    pub fn register_bundle<B: DeserializeOwned + Clone + DynamicBundleClone>(&mut self, key: &str) {
        self.register::<B>(key, |_, builder, x| {
            builder.add_bundle(x);
            Ok(())
        });
    }

    pub fn register_component<C: DeserializeOwned + Clone + Component>(&mut self, key: &str) {
        self.register::<C>(key, |_, builder, x| {
            builder.add(x);
            Ok(())
        });
    }

    pub fn register_component_with_constructor_ctx<Seed: DeserializeOwned, C: Clone + Component>(
        &mut self,
        key: &str,
        constructor: impl Fn(Seed, &mut T) -> anyhow::Result<C> + 'static,
    ) {
        self.register::<Seed>(key, move |ctx, builder, x| {
            builder.add(constructor(x, ctx)?);
            Ok(())
        });
    }

    pub fn register_component_with_constructor<Seed: DeserializeOwned, C: Clone + Component>(
        &mut self,
        key: &str,
        constructor: impl Fn(Seed) -> C + 'static,
    ) {
        self.register::<Seed>(key, move |_, builder, x| {
            builder.add(constructor(x));
            Ok(())
        });
    }

    pub fn register<V: DeserializeOwned>(
        &mut self,
        key: &str,
        insert: impl Fn(&mut T, &mut EntityBuilderClone, V) -> anyhow::Result<()> + 'static,
    ) {
        if self.registry.contains_key(key) {
            panic!("duplicate key {key:?}");
        }

        self.registry.insert(
            key.to_string(),
            Box::new(move |ctx, builder, val| {
                let val = serde_json::from_str::<V>(val.get())?;
                insert(ctx, builder, val)?;
                Ok(())
            }),
        );
    }

    pub fn build(
        &self,
        ctx: &mut T,
        start: &mut EntityBuilderClone,
        pref: &PrePrefab,
    ) -> anyhow::Result<()> {
        for (name, value) in pref.0.iter() {
            let Some(builder) = self.registry.get(*name) else {
                anyhow::bail!("unknown component: {name:?}");
            };
            builder(ctx, start, value).with_context(|| format!("build {name:?}"))?;
        }
        Ok(())
    }
}

type ComponentBuilder<T> = Box<
    dyn Fn(&mut T, &mut EntityBuilderClone, &serde_json::value::RawValue) -> anyhow::Result<()>,
>;

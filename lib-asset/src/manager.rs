use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use hashbrown::HashMap;
use hecs::{BuiltEntityClone, EntityBuilderClone};
use mimiq::{FileReady, FsServerHandle};

use crate::{AssetRoot, FsResolver, PrefabFactory};

const TARGET_NAME: &str = "asset_manager";

pub struct AssetManager<T> {
    pub fs_resolver: FsResolver,
    prefab_factory: Rc<PrefabFactory<T>>,
    fs_server: FsServerHandle,
    outgoing_requests: HashMap<PathBuf, Callback<T>>,
}

impl<T: 'static> AssetManager<T> {
    pub fn new(fs_server: FsServerHandle, prefab_factory: PrefabFactory<T>) -> Self {
        AssetManager {
            fs_server,
            prefab_factory: Rc::new(prefab_factory),
            fs_resolver: FsResolver::new(),
            outgoing_requests: HashMap::new(),
        }
    }

    pub fn load_prefab(
        &mut self,
        src: impl AsRef<Path>,
        callback: impl FnOnce(&mut T, &FsResolver, BuiltEntityClone, PathBuf) + 'static,
    ) {
        let factory = self.prefab_factory.clone();
        let path = src.as_ref().to_path_buf();
        
        tracing::info!(target: TARGET_NAME, file_path=?path, "Submitting prefab task");
        self.fs_server.load_file(path.clone());
        self.outgoing_requests.insert(
            path.clone(),
            Box::new(move |ctx, res, data| {
                let _span = tracing::info_span!(
                    target: TARGET_NAME,
                    "load_json",
                    ?path,
                )
                .entered();
                let pre_prefab = match serde_json::from_slice(&data) {
                    Ok(manifest) => manifest,
                    Err(err) => {
                        tracing::error!(target: TARGET_NAME, %err, "failed to deserialize prefab");
                        return;
                    }
                };
                let mut builder = EntityBuilderClone::new();
                if let Err(err) = factory.build(ctx, &mut builder, &pre_prefab) {
                    tracing::error!(target: TARGET_NAME, %err, "failed to build prefab");
                };
                callback(ctx, res, builder.build(), path);
            }),
        );

    }

    pub fn load_image(
        &mut self,
        src: impl AsRef<Path>,
        callback: impl FnOnce(&mut T, &FsResolver, image::DynamicImage, PathBuf) + 'static,
    ) {
        let path = src.as_ref().to_path_buf();
        
        tracing::info!(target: TARGET_NAME, file_path=?path, "Submitting an image task");
        self.fs_server.load_file(path.clone());
        self.outgoing_requests.insert(
            path.clone(),
            Box::new(move |ctx, res, data| {
                let _span = tracing::info_span!(
                    target: TARGET_NAME,
                    "load_image",
                    ?path,
                )
                .entered();
                match image::load_from_memory(&data) {
                    Ok(img) => callback(ctx, res, img, path),
                    Err(err) => {
                        tracing::error!(target: TARGET_NAME, %err, "failed to decode image")
                    }
                }
            }),
        );
    }

    pub fn on_file_ready(&mut self, ctx: &mut T, event: FileReady) {
        match event.bytes_result {
            Ok(data) => {
                let callback = self.outgoing_requests.remove(&event.path).unwrap();
                callback(ctx, &self.fs_resolver, data);
            }
            Err(err) => {
                tracing::error!(target: TARGET_NAME, path=?event.path, %err, "Failed to load the file")
            }
        }
    }
}

type Callback<T> = Box<dyn FnOnce(&mut T, &FsResolver, Vec<u8>)>;

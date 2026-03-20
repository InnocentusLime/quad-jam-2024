use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use hecs::{BuiltEntityClone, EntityBuilderClone};
use mimiq::{FileReady, FsServerHandle};
use slab::Slab;

use crate::{AssetRoot, FsResolver, PrefabFactory};

const TARGET_NAME: &str = "asset_manager";

pub struct AssetManager<T> {
    pub fs_resolver: FsResolver,
    prefab_factory: Rc<PrefabFactory<T>>,
    fs_server: FsServerHandle,
    outgoing_requests: Slab<Callback<T>>,
}

impl<T: 'static> AssetManager<T> {
    pub fn new(fs_server: FsServerHandle, prefab_factory: PrefabFactory<T>) -> Self {
        AssetManager {
            fs_server,
            prefab_factory: Rc::new(prefab_factory),
            fs_resolver: FsResolver::new(),
            outgoing_requests: Slab::new(),
        }
    }

    pub fn load_prefab(
        &mut self,
        src: impl AsRef<Path>,
        callback: impl FnOnce(&mut T, &FsResolver, BuiltEntityClone, PathBuf) + 'static,
    ) {
        let factory = self.prefab_factory.clone();
        let src = src.as_ref().to_path_buf();
        let path = self.fs_resolver.get_path(AssetRoot::Assets, &src);
        let request_id = self
            .outgoing_requests
            .insert(Box::new(move |ctx, res, data| {
                let _span = tracing::info_span!(
                    target: TARGET_NAME,
                    "load_json",
                    ?src,
                )
                .entered();
                let pre_prefab = match serde_json::from_slice(&data) {
                    Ok(manifest) => manifest,
                    Err(err) => {
                        tracing::error!(
                            target: TARGET_NAME,
                            %err,
                            "failed to deserialize prefab",
                        );
                        return;
                    }
                };
                let mut builder = EntityBuilderClone::new();
                if let Err(err) = factory.build(ctx, &mut builder, &pre_prefab) {
                    tracing::error!(
                        target: TARGET_NAME,
                        %err,
                        "failed to build prefab",
                    );
                };
                callback(ctx, res, builder.build(), src);
            }));

        tracing::info!(
            target: TARGET_NAME,
            file_path=?path,
            request_id,
            "Submitting prefab task"
        );
        self.fs_server.submit_task(&path, request_id as u64);
    }

    pub fn load_image(
        &mut self,
        src: impl AsRef<Path>,
        callback: impl FnOnce(&mut T, &FsResolver, image::DynamicImage, PathBuf) + 'static,
    ) {
        let src = src.as_ref().to_path_buf();
        let path = self.fs_resolver.get_path(AssetRoot::Assets, &src);
        let request_id = self
            .outgoing_requests
            .insert(Box::new(move |ctx, res, data| {
                let _span = tracing::info_span!(
                    target: TARGET_NAME,
                    "load_image",
                    ?src,
                )
                .entered();
                match image::load_from_memory(&data) {
                    Ok(img) => callback(ctx, res, img, src),
                    Err(err) => tracing::error!(
                        target: TARGET_NAME,
                        %err,
                        "failed to decode image",
                    ),
                }
            }));

        tracing::info!(
            target: TARGET_NAME,
            file_path=?path,
            request_id,
            "Submitting an image task"
        );
        self.fs_server.submit_task(&path, request_id as u64);
    }

    pub fn on_file_ready(&mut self, ctx: &mut T, event: FileReady) {
        let request_id = event.user_id as usize;
        match event.bytes_result {
            Ok(data) => {
                let callback = self.outgoing_requests.remove(request_id);
                callback(ctx, &self.fs_resolver, data);
            }
            Err(err) => tracing::error!(
                target: TARGET_NAME,
                request_id,
                %err,
                "Failed to load the file",
            ),
        }
    }
}

type Callback<T> = Box<dyn FnOnce(&mut T, &FsResolver, Vec<u8>)>;

use std::path::{Path, PathBuf};

use mimiq::{FileReady, FsServerHandle};
use slab::Slab;

use crate::{AssetRoot, FsResolver};

const TARGET_NAME: &str = "asset_manager";

pub struct AssetManager<T> {
    pub fs_resolver: FsResolver,
    fs_server: FsServerHandle,
    outgoing_requests: Slab<Callback<T>>,
}

impl<T> AssetManager<T> {
    pub fn new(fs_server: FsServerHandle) -> Self {
        AssetManager {
            fs_server,
            fs_resolver: FsResolver::new(),
            outgoing_requests: Slab::new(),
        }
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
        self.fs_server
            .submit_task(&path.to_string_lossy(), request_id as u64);
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

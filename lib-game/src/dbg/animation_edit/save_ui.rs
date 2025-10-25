use std::fs::File;
use std::path::{Path, PathBuf};

use egui::Modal;
use egui::Ui;

use hashbrown::HashMap;
use lib_anim::{Animation, AnimationId, AnimationPackId};
use lib_asset::FsResolver;
use log::{error, info, warn};
use rfd::FileDialog;
use strum::VariantArray;

pub fn save_anim_pack_modal(
    ui: &mut Ui,
    current_pack_id: &mut AnimationPackId,
    anims: &HashMap<AnimationId, Animation>,
) -> bool {
    let mut keep_open = true;
    Modal::new(egui::Id::new("Save Pack")).show(ui.ctx(), |ui| {
        ui.set_width(250.0);
        ui.heading("Save animation pack");

        super::enum_select(ui, "animation_pack_id", "animation pack", current_pack_id);
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                let dst = FileDialog::new()
                    .set_title("Save animation pack")
                    .add_filter("", &["json"])
                    .save_file();
                if let Some(dst) = dst {
                    save_anim_pack(dst, *current_pack_id, anims);
                    keep_open = false;
                }
            }
            if ui.button("Cancel").clicked() {
                keep_open = false;
            }
        });
    });
    keep_open
}

fn save_anim_pack(
    dst: impl AsRef<Path>,
    pack_id: AnimationPackId,
    anims: &HashMap<AnimationId, Animation>,
) {
    let dst = dst.as_ref();
    let mut file = match File::create(dst) {
        Ok(file) => file,
        Err(e) => {
            error!("Could not open {dst:?}: {e}");
            return;
        }
    };

    let mut output = HashMap::new();
    for anim_id in AnimationId::VARIANTS {
        let anim_id_name: &'static str = anim_id.into();
        let pack_id_name: &'static str = pack_id.into();
        if !anim_id_name.starts_with(pack_id_name) {
            info!("Skipping {anim_id:?}");
            continue;
        }

        let Some(anim) = anims.get(anim_id) else {
            warn!("Skipping {anim_id:?}: not loaded");
            continue;
        };
        info!("Adding {anim_id_name}");
        output.insert(*anim_id, anim.clone());
    }

    match serde_json::to_writer_pretty(&mut file, &output) {
        Ok(_) => info!("Wrote data to {dst:?}"),
        Err(e) => error!("Failed to write to {dst:?}: {e}"),
    }
}

pub fn animation_load_ui(
    ui: &mut Ui,
    resolver: &FsResolver,
    current_anim_id: AnimationId,
    current_anim: &mut Animation,
) {
    if ui.button("Save").clicked() {
        let fname: &'static str = current_anim_id.into();
        let dst = FileDialog::new()
            .set_title("Save animation")
            .set_file_name(format!("{fname}.json"))
            .add_filter("", &["json"])
            .save_file();
        if let Some(dst) = dst {
            save_anim(dst, current_anim);
        }
    }
    if ui.button("Load").clicked() {
        let fname: &'static str = current_anim_id.into();
        let src = FileDialog::new()
            .set_title("Load animation")
            .set_file_name(format!("{fname}.json"))
            .add_filter("", &["json"])
            .pick_file();
        if let Some(loaded_anim) = src.and_then(load_anim) {
            *current_anim = loaded_anim;
        }
    }
    if ui.button("Load Aseprite").clicked() {
        let src = FileDialog::new()
            .set_title("Load aseprite animation")
            .add_filter("", &["json"])
            .pick_file();
        let loaded_anim = src.and_then(|src| load_aseprite_anim(src, resolver, current_anim_id));
        if let Some(loaded_anim) = loaded_anim {
            *current_anim = loaded_anim;
        }
    }
}

fn save_anim(dst: PathBuf, anim: &Animation) {
    let mut file = match File::create(&dst) {
        Ok(file) => file,
        Err(e) => {
            error!("Could not open {dst:?}: {e}");
            return;
        }
    };

    match serde_json::to_writer_pretty(&mut file, anim) {
        Ok(_) => info!("Wrote data to {dst:?}"),
        Err(e) => error!("Failed to write to {dst:?}: {e}"),
    }
}

fn load_anim(src: PathBuf) -> Option<Animation> {
    let mut file = match File::open(&src) {
        Ok(file) => file,
        Err(e) => {
            error!("Could not open {src:?}: {e}");
            return None;
        }
    };

    match serde_json::from_reader(&mut file) {
        Ok(x) => {
            info!("Loaded data to {src:?}");
            x
        }
        Err(e) => {
            error!("Failed to load from {src:?}: {e}");
            None
        }
    }
}

fn load_aseprite_anim(
    src: PathBuf,
    resolver: &FsResolver,
    target: AnimationId,
) -> Option<Animation> {
    let mut anims = match lib_anim::aseprite_load::load_animations(resolver, &src) {
        Ok(x) => x,
        Err(e) => {
            error!("Failed to load {src:?}: {e}");
            return None;
        }
    };
    let Some(anim) = anims.remove(&target) else {
        error!("No animation for {target:?}");
        return None;
    };

    Some(anim)
}

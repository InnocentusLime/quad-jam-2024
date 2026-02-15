# === Migration from v3 to v4 ===
# 1. DrawSprite clip actions no longer reference the atlas by
#    some internal ID. Instead, filenames are used.
# 2. The field used to reference the atlas is now called "atlas_file"

import json

files = [
    "bnuuy.json",
    "shooter.json",
    "stabber.json",
]

id2file = {
    "BunnyAtlas": "bnuuy.png",
    "WorldAtlas": "world.png",
}

DRAW_SPRITE = "draw_sprite" 

for out in files:
    path = "animations/" + "_" + out
    out = "animations/" + out
    pack = None

    with open(path) as f:
        pack = json.load(f)
    for anim in pack.values():
        for clip in anim["action_tracks"][DRAW_SPRITE]["clips"]:
            texture_id = clip["action"]["texture_id"]
            del clip["action"]["texture_id"]
            clip["action"]["atlas_file"] = id2file[texture_id]

    with open(out, "w") as f:
        json.dump(pack, f, ensure_ascii=True, indent=2)
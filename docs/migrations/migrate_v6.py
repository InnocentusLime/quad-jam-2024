# === Migration from v5 to v6 ===
# 1. All animation atlases now live in assets/atlas

import json

files = [
    "bnuuy.json",
    "shooter.json",
    "stabber.json",
]

DRAW_SPRITE = "draw_sprite" 

for out in files:
    path = "assets/anim/" + "_" + out
    out = "assets/anim/" + out
    pack = None

    with open(path) as f:
        pack = json.load(f)
    repl = {}
    for animkey,animval in pack.items():
        newanimkey = animkey.replace("Bunny", "Bnuuy")
        repl[animkey] = newanimkey,animval
    for anim in pack.values():
        for clip in anim["action_tracks"][DRAW_SPRITE]["clips"]:
            f = clip["action"]["atlas_file"]
            clip["action"]["atlas_file"] = "atlas/" + f

    with open(out, "w") as f:
        json.dump(pack, f, ensure_ascii=True, indent=2)
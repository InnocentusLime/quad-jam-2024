# === Migration from v2 to v3 ===
# 1. Action fields are no longer inlined into clips. Instead, they are
#    stored in a field called "actions"
# 2. All separate clip action tracks are stored in a field called "action_tracks"

import json

files = [
    "bnuuy.json",
    "shooter.json",
    "stabber.json",
]

CLIP_KEYS = {
    "track_id",
    "start",
    "len",
}

def clipkind(clip):
    for key in clip:
        if key not in CLIP_KEYS:
            return key
    raise "Clip has not kind"
        

for out in files:
    path = "animations/" + "_" + out
    out = "animations/" + out
    pack = None

    with open(path) as f:
        pack = json.load(f)
    for packkey,packval in pack.items():
        controls = {}
        for animkey,animval in packval.items():
            if animkey == "is_looping": continue
            for clip in animval["clips"]:
                action = None
                for clipkey,clipval in clip.items():
                    if clipkey in CLIP_KEYS: 
                        clip[clipkey] = clipval
                        continue
                    if action is None: action = {}
                    action[clipkey] = clipval
                if action is not None:
                    for key in action: del clip[key] 
                clip["action"] = action
            controls[animkey] = animval
        for controlkey in controls: del packval[controlkey]
        packval["action_tracks"] = controls

    with open(out, "w") as f:
        json.dump(pack, f, ensure_ascii=True, indent=2)
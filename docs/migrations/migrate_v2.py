# === Migration from v1 to v2 ===
# 1. Tracks no longer have an ID. Their ID is now their index in the array
# 2. Clips no longer have an ID. Their ID is now their index in the array
# 3. draw_sprite clips no longer use the "rect" field. The rect is now stored
#    as two separate vector fields: "rect_pos" and "rect_size"
# 4. attack_box clips no longer use the "team" field. The team of the attack is
#    inferred from the parent object team
# 5. Different clip action kinds are stored separately

import json
import copy

files = [
    "bnuuy.json",
    "shooter.json",
    "stabber.json",
]

ATTACK_BOX = "attack_box"
DRAW_SPRITE = "draw_sprite"
MOVE = "move"
LOCK_INPUT = "lock_input"
SPAWN = "spawn"
INVULNERABILITY = "invulnerability"
ACTION_KEYS = {
    ATTACK_BOX: ATTACK_BOX,
    DRAW_SPRITE: DRAW_SPRITE,
    MOVE: MOVE,
    LOCK_INPUT: LOCK_INPUT,
    SPAWN: SPAWN,
    INVULNERABILITY: INVULNERABILITY,
}

CLIP_KEYS = [
    "track_id",
    "start",
    "len",
]

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
    for anim in pack.values():
        tid2newtid = {}
        oldtracks = {}
        for track in anim["tracks"]:
            oldtracks[track["id"]] = track
        trackvst = set()

        for action in ACTION_KEYS.values():
            anim[action] = {
                "clips": [],
                "tracks": [],
            }
        
        for clip in anim["clips"]:
            del clip["id"]
            kind = clipkind(clip)
            kind_key = ACTION_KEYS[kind]
            
            if clip["track_id"] not in tid2newtid:
                tid2newtid[clip["track_id"]] = len(anim[kind_key]["tracks"])
                anim[kind_key]["tracks"].append({
                    "name": oldtracks[clip["track_id"]]["name"]
                })
            clip["track_id"] = tid2newtid[clip["track_id"]]
            
            if DRAW_SPRITE in clip:
                rect = clip[DRAW_SPRITE]["rect"]
                clip[DRAW_SPRITE]["rect_pos"] = [
                    clip[DRAW_SPRITE]["rect"]["x"],
                    clip[DRAW_SPRITE]["rect"]["y"],
                ]
                clip[DRAW_SPRITE]["rect_size"] = [
                    clip[DRAW_SPRITE]["rect"]["w"],
                    clip[DRAW_SPRITE]["rect"]["h"],
                ]
            if ATTACK_BOX in clip:
                del clip[ATTACK_BOX]["team"]
            
            action = clip[kind]
            del clip[kind]
            if action is not None:
                for field, val in action.items():
                    clip[field] = val            
            anim[kind_key]["clips"].append(clip)

        del anim["tracks"]
        del anim["clips"]

    with open(out, "w") as f:
        json.dump(pack, f, ensure_ascii=True, indent=2)
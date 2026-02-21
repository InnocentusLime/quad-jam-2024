# === Migration from v4 to v5 ===
# 1. All bnuuy animation IDs now use "Bnuuy" as a prefix instead
#    of "Bunny"

import json

files = [
    "bnuuy.json",
]

for out in files:
    path = "assets/" + "_" + out
    out = "assets/" + out
    pack = None

    with open(path) as f:
        pack = json.load(f)
    repl = {}
    for animkey,animval in pack.items():
        newanimkey = animkey.replace("Bunny", "Bnuuy")
        repl[animkey] = newanimkey,animval
    for animkey,(newanimkey,animval) in repl.items():
        del pack[animkey]
        pack[newanimkey] = animval

    with open(out, "w") as f:
        json.dump(pack, f, ensure_ascii=True, indent=2)
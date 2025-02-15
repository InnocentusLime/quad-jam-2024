# Full Design Doc

## Introduction

Multihit is an arcade top-down game with a “Baba is you” art-style. It is inspired by the following level from Little Big Planet 2

https://www.youtube.com/watch?v=bQK-qMR4ktc

But the player will wield something like a ball-o-hurt from Terraria.

https://www.youtube.com/watch?v=JnKf0zQNXxs

This weapon will not kill enemies unless they manage to hit several enemies at once
If there is only 1 enemy in the room — it dies

## Enemies

* **Bunny**: Moves more or less aimlessly hopping by large tile counts
* **Brute**: Simply walks towards the player. But when they get hit — they get a strong knockback
* **Bull**: Simply walks towards the player. When player is in sight — dashes at them
* **Worms**: Jump between walls. Otherwise move inside them

## Boss

Pretty much what Little Big Planet had, but with a spin (todo: figure out)

# Core

1. The ball-o-hurt
2. Brute
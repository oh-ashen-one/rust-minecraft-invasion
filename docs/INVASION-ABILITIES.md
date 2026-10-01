# Rust invasion abilities and quickscope class

Opt in with `IW4L_RUST_INVASION=1`, then select Rust and **Intervention Quickscope**.
The class is inserted first without removing existing saved classes. It uses
Intervention, USP .45, frag, flash, Sleight of Hand Pro, Stopping Power and Steady Aim.
Sleight of Hand Pro halves eligible weapon reload and ADS transition times.
The regular Sleight of Hand perk keeps its existing reload-only behavior.

Creepers stop, hiss, swell and flash through a 1.5-second fuse. Leaving their
range or breaking sight cancels the fuse. Blasts deal distance-falloff damage
and knockback to players and nearby mobs, with sound and smoke. Charged Creepers
have a larger blast. Native Rust geometry blocks blast damage and remains intact.

Skeleton-family archers retreat from close players, aim a bow and fire physical
arrows. Spiders and Cave Spiders can climb vertical native collision. Injured or
obstructed Endermen teleport only to a supported, clear destination, with a
cooldown, sound and particles. Witches alternate poison and harming splash
potions. Poison lasts six seconds, is nonlethal, and clears on player death.
Blazes charge and shoot three fireballs per burst; Ghasts charge larger fireballs.
Ender Dragons retain their flight path and use visible explosive fireballs.

The 67-entry roster, two of each, varied sizes and three-second respawns remain.
These are custom arena adaptations, not the complete vanilla Minecraft AI.
Other creature families retain their existing arena behavior.

Implementation: `sim/src/invasion/behavior.rs` handles collision-backed abilities;
`render_anim/src/rust_invasion/abilities.rs` adds geometry to the existing entity
pass. Impacts use the authoritative damage path. The Pro class retains perk ID
17 in its loadout while supplying the authored base perk name to class scripts;
its ADS bit is applied to both movement and combat timing from immutable facts.

Validation: 17 focused behavior, collision, respawn and perk checks passed.
The owned MW2 data admitted the class and measured Intervention ADS at 400 ms
normally / 200 ms with Pro; all 134 mobs found supported Rust spawns. Temporary
new tests/probes were removed before push under the repository test policy.
No renderer/play session was launched; visual/FPS/gameplay acceptance is pending.

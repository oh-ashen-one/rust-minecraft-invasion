# Invasion killstreaks and survival balance

All fifteen standard MW2 (2009) multiplayer rewards are enabled together.
No killstreak selection is required. Player weapon/grenade kills advance the
ladder; support kills and mobs killing each other do not create reward loops.
Earned rewards remain queued across death; the consecutive kill counter resets.
D-pad Right / keyboard 4 deploys the oldest reward, or exits a remote view.

| Kills | Rewards |
| --- | --- |
| 3 | UAV |
| 4 | Care Package, Counter-UAV |
| 5 | Sentry Gun, Predator Missile |
| 6 | Precision Airstrike |
| 7 | Harrier Strike, Attack Helicopter |
| 8 | Emergency Airdrop |
| 9 | Pave Low, Stealth Bomber |
| 11 | Chopper Gunner, AC-130 |
| 15 | EMP |
| 25 | Tactical Nuke |

These are survival adaptations targeting the Minecraft mob simulation, with
custom support geometry and effects. UAV reveals mobs on a radar. Counter-UAV
disrupts ranged accuracy/range; EMP suppresses ranged attacks for twenty seconds.
Supply crates descend, grant a support reward and heal when approached. Emergency
Airdrop drops four crates. Sentries validate placement and use line of sight.
Airstrikes, bombers and autonomous helicopters damage mobs against native cover.

Predator, Chopper Gunner and AC-130 use remote cameras with the existing mouse /
right-stick aim. R2 / left click fires or boosts the Predator. L2 / right click
cycles AC-130 cannons (25 / 40 / 105 mm). The player body is held still and
protected from mob damage during remote control; exit/expiry/death restores it.
The nuke has a ten-second countdown, clears living mobs, heals the player and
restarts the ladder. The arena continues, with normal individual mob respawns.

Invasion players have a minimum 300 HP, mobs have 75% of their previous health,
and mob damage has a 0.25-second grace interval. The full 134 target population,
original MW2 terrain/weapons, native Mac controller path and Pro class remain.

Validation: 16 simulation checks passed (six existing, ten temporary additions).
A CPU-only probe exercised every reward for 250 ticks against the actual Rust
collision, including remote-camera clearance. A fresh resource preparation
completed without a renderer. New temporary tests/probes are removed before
publication under repository policy. Live visual/input/balance acceptance is
still pending owner play; CPU checks are not a rendered FPS measurement.

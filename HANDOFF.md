# Public survival release handoff

Current release: experimental **v0.8.0**, branch `release/0.8.0`.
Public repository: `oh-ashen-one/rust-minecraft-invasion`.

The full runtime and portable launcher source are in this repository. It is a
clean export of the invasion development lane, retaining upstream license and
credit files. Private development history, machine configuration, game assets,
profiles and build caches are excluded.

Current mode: original MW2 Rust, 67 mob/variant entries with two of each,
three-second respawns, characteristic mob abilities, Intervention Quickscope
with Sleight of Hand Pro, 300 HP, 75% mob health and incoming-hit grace. All
fifteen standard MW2 rewards are adapted to the Minecraft arena and earned
without selecting a loadout. D-pad Right / keyboard 4 uses the oldest reward;
its decoded action-slot index is 3. Remote support protects and holds the body,
then restores the normal view on exit, expiry or death. A nuke wipes the mobs
and restarts the ladder while the arena continues.

Read README.md, VALIDATION.md and docs/INVASION-KILLSTREAKS.md. The tests/probes
used during development were temporary under the existing test policy. New
live visual, controller, balance and FPS acceptance remains with players. Do
not present CPU-only checks or successful compilation as gameplay acceptance.

Build with `cargo build --locked --profile play -p launcher -j 4` and package
with `python3 invasion-launcher/package_macos.py`. Packaging executes only
bundle/help checks. Never automatically enter a match or operate an existing
player session. The launcher retains shared renderer-slot locks, the two-engine
cap, graceful owned-child shutdown and no automatic crash relaunch.

macOS runtime files belong in the versioned Application Support directory;
the MW2 installation is read-only. Ship no original game archives or Mojang
assets. The app prepares Minecraft resources from official servers on demand.
Keep runtime credentials, profiles and logs out of Git. Future code changes
belong on a task branch; this release does not authorize merging other branches.

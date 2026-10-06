# Native killstreak integration — v0.9.0 candidate

Branch: `codex/native-mw2-killstreaks`, based on the menu-fix branch at c84a909.
The owner accepted that base, then requested original MW2 rewards and aircraft,
a bottom-left earned stack, manual activation and mob-kill credit. Keep the
accepted v0.8.1 app intact. Build/install this candidate separately and let the
owner playtest; do not send input into a match or restart their game.

The custom support implementation has been removed. Mob deaths feed the full
fifteen-reward ladder into the original reward script. Original persistent
reward records drive the stock item/action slot and a bottom-left HUD using
owned stock icon materials, grouping repeated rewards with counts. The stock
stack is newest-first. No award invokes deployment.

The source adapter applies only to opt-in Rust invasion and modifies the owned
scripts in memory. It changes enemy acquisition in three helicopter routines
and the Harrier routine, adds mob blast damage to airstrike/nuke paths, and loads
an authored HUD module. Native sentry acquisition and vehicle/turret bullet rays
recognize mob targets. Native explosions preserve weapon radius/falloff and
cover checks. UAV uses the original compass. Native nuke ends the match; EMP
and Counter-UAV no longer apply invented creature debuffs.

Full build and original-script-graph compilation passed, and all fifteen costs,
weapon refs and icon refs matched the local table. See VALIDATION.md for bounds.
The code candidate at 52a6f32 was packaged, ad-hoc signed, bundle/CLI checked
and installed separately as `Rust Minecraft Invasion Native Killstreaks 0.9.app`.
A portable ZIP was saved to Downloads (SHA256
`b1f8eafac8b7ff98644abbff8a7d50e39d1ab8d803b054c3d7c11e2090eb56ca`).
The accepted v0.8.1 binary still matches its recorded build hash. No candidate
renderer or match was launched.
Live visuals, activation sequences, remote control and sound await the owner.
Probes/extracts stay out of publication. Do not merge this branch or replace the
accepted app based only on these offline checks.

Runtime data remains in `0.8.0` Application Support, including the owner's
controller/class/settings and machine-local RendererSlotDirectory. The candidate
shares the existing single-instance guard. The public v0.8 release stays unchanged.

---

# Menu recovery patch — v0.8.1

Branch: `codex/fix-invasion-menu-responses`, based on public v0.8.0 `97bed1e`.
Owner reported getting trapped at Auto-Assign on Rust and being unable to end
the match on October 6. The log recorded Back as the first team-menu response,
then repeated team and End Round choices without a join/spawn.

Confirmed cause: the menu-answer bridge cleared server menu state on receipt
of any answer. A rejected/cancelled Back answer therefore blocked subsequent
team choices. Client-opened End Round popups were also gated on a server-open
menu record that did not exist. Real client responses now reach GSC after the
player begins, while automatic join/class answers retain their readiness gate.
Scripts own menu closure. Explicit choices supersede queued automatic choices
for the same menu, and Back does not mark the player joined.

Validation: two targeted tests reproduced the old failure, then six menu
regression checks passed. Four isolated launcher fixtures passed for persistent
lock-root selection, exclusive-lock refusal, PAUSED, conflicting configuration
and missing-root refusal (the root-selection case is the exclusive-lock test).
Only temporary fixtures were used, and they were removed under repository
policy. No game window, match or input was used for this repair's tests.

The launcher reads optional `RendererSlotDirectory` from the machine-local
`settings.plist`, preserves it when the MW2 folder changes, refuses conflicting
or missing configured roots, recognizes additional game renderers and blocks
when GTA is active. Never change shared PAUSED/slot files or other processes.

App version is 0.8.1; runtime data stays in the existing `0.8.0` Application
Support folder so settings, controls, classes and downloaded assets survive.
Install only with this app/game closed and retain a recoverable old-app backup.
Do not automatically relaunch: the owner will choose when to test the repair.
The public v0.8.0 release/tag remains unchanged; this patch is on its task branch.

---

## Previous release handoff

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

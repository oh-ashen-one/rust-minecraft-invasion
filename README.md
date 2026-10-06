# Rust Minecraft Invasion

**v0.9.0 — Native MW2 Killstreaks.** Completed and accepted for publication by the project owner. Includes the team-menu recovery fix and preserves existing profiles.

A local survival crossover: **Modern Warfare 2’s Rust map and weapons against a crowd of Minecraft mobs**, including Ender Dragons, Withers and Wardens.

**[Download the Apple silicon Mac build](https://github.com/oh-ashen-one/rust-minecraft-invasion/releases/tag/v0.9.0)** · [Release downloads](https://github.com/oh-ashen-one/rust-minecraft-invasion/releases) · [Source build](#build-from-source)

This is an experimental single-player mode built on [2010 Rust Rewrite Mashup](https://github.com/chasmlol/2010-rust-rewrite-mashup), which builds on [IW4L](https://github.com/vladtrc/iw4L). It is a standalone native runtime that reads assets from a game installation you own. Call of Duty, Minecraft and other original game assets are **not bundled**.

## Give it to your agent

Share this repository and ask your agent to follow **[PLAY_WITH_AGENT.md](PLAY_WITH_AGENT.md)**.
That guide includes a copy-and-paste setup prompt, supported-machine checks,
release checksum verification, game-folder selection and preparation steps.
The downloadable Mac app works without an agent or a development toolchain.

## Play on an Apple silicon Mac

Requirements: **macOS 14 or later**, Apple silicon, and the **English Windows PC files for Modern Warfare 2 (2009)**, including the base game and multiplayer data. The 2022 game and Campaign Remastered are different products and cannot supply this map data. A folder installed through Steam/CrossOver is suitable. No Skyrim files are needed.

1. Download the `macOS-arm64.zip` asset from [Releases](https://github.com/oh-ashen-one/rust-minecraft-invasion/releases) and extract it.
2. Move **Rust Minecraft Invasion.app** to Applications, then open it. The app opens a launcher; it does not start a match.
3. Click **Choose MW2 Folder** and select the folder containing `main/` and `zone/english/mp_rust.ff`.
4. Click **Prepare Files**. The game downloads the pinned Minecraft resources directly from Mojang and checks their hashes. This can take several minutes. No game window opens during preparation.
5. Click **Open Game Menu → Create Game → Rust Invasion → Rust**, start the match, choose **Intervention Quickscope**, and select **Equip**.

The Mac build is ad-hoc signed, not Apple-notarized. If macOS blocks the downloaded app, follow [Apple’s instructions for opening a trusted app](https://support.apple.com/en-us/102445). The app and these instructions do not disable Gatekeeper or remove quarantine attributes.

**Check Setup** verifies the executable and prepared files without starting a game. Settings, profiles, resources and logs live in `~/Library/Application Support/Rust Minecraft Invasion/0.8.0/`. The selected MW2 installation is read-only. Close the game before moving the app or changing audio output devices.

## Controls

| Action | Keyboard / mouse | Default DualSense |
| --- | --- | --- |
| Move / look | WASD / mouse | Left / right stick |
| Fire / aim | Left / right mouse | R2 / L2 |
| Jump / reload | Space / R | Cross / Square |
| Switch weapon | Existing MW2 weapon-cycle binding | Triangle |
| Deploy next earned streak | **4** | **D-pad Right** |
| Remote gunship fire / Predator boost | Left mouse | R2 |

Controllers use Apple’s GameController framework on macOS; Steam Input is not required. Focus the game window to play. Controller rumble is not implemented by the native Mac bridge. Options → Controls / Controller retains the normal binding and sensitivity menus.

## The invasion

- **67 mob/variant entries, two of each: 134 target enemies**, with varied sizes and individual **three-second respawns**. [Complete roster](docs/INVASION-ROSTER.json).
- Creepers hiss, swell, flash and explode; retreating cancels their fuse. Explosions damage and push players/mobs without destroying Rust’s structures.
- Skeletons use bows/arrows, spiders climb, Endermen teleport to checked positions, witches throw splash potions, and flying attackers use fireballs.
- **300 player HP**, **25% less mob health** than the previous invasion build, and a brief **0.25-second interval between mob hits** to limit crowd volleys.
- **Intervention Quickscope + Sleight of Hand Pro**: twice-as-fast eligible reloads and scope transitions. The Intervention’s data gives 400 ms aim-in normally / 200 ms with Pro. Regular Sleight of Hand stays reload-only.
- The existing invasion’s unlimited ammunition and bonus Intervention / SPAS-12 / UMP45 remain available.

## Every standard MW2 killstreak, enabled together

No killstreak loadout selection is required. Minecraft mob kills unlock **all rewards at each threshold**, including rewards that share a cost. Earning a reward never deploys it. The original MW2 scripts retain the reward stack across death, display the earned notification, equip the appropriate killstreak item, and handle manual use through **D-pad Right / 4**. The consecutive mob-kill counter resets on death.

The bottom-left stack uses the original reward icons from your MW2 installation. Repeated rewards show a count; the stock action-slot icon shows the next usable item. Rewards follow MW2's newest-first stack order.

| Consecutive kills | Rewards |
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

This release uses the original MW2 killstreak scripts, aircraft, weapons, effects and sounds from the player's own installation. Minecraft mobs are exposed to helicopter/Harrier/sentry targeting; native bullets and explosions damage their hitboxes. UAV pings appear on the original minimap. The custom block aircraft, radar panel, remote cameras and supply-crate behavior have been removed.

The original scripts control deployment restrictions, remote weapons, care-package collection, EMP and Counter-UAV behavior. The nuke follows the original countdown and match-ending sequence. This community runtime remains experimental; see the verification limits below.

## Build from source

The complete runtime, mob implementation, controller bridge, launcher and packaging source are in this repository. There are no private source submodules.

Install Rust (the repository pins the toolchain) and platform build dependencies from [BUILD.md](docs/BUILD.md). On macOS, install Xcode Command Line Tools. Then:

```sh
git clone --branch v0.9.0 https://github.com/oh-ashen-one/rust-minecraft-invasion.git
cd rust-minecraft-invasion
cargo build --locked --profile play -p launcher -j 4
python3 invasion-launcher/package_macos.py
```

The packaging script creates `dist/Rust Minecraft Invasion.app`, a ZIP and `SHA256SUMS.txt`. It verifies code signatures and executes the binary’s help path; it does not launch a renderer or match. The packaged app does not need Python, Rust or this checkout on the player’s computer.

For a source run on macOS/Linux, choose a writable runtime directory and use absolute executable/data paths:

```sh
/path/to/iw4l prepare-invasion
IW4L_GAMES='/path/to/owned/MW2' IW4L_RUST_INVASION=1 IW4L_SKATE=off IW4L_UPDATE=0 /path/to/iw4l menu
```

Upstream Windows/Linux build paths remain available in [BUILD.md](docs/BUILD.md) and [WINDOWS.md](docs/WINDOWS.md). This release includes a tested-to-load Apple silicon binary; this invasion version has not been runtime-tested on Windows, Intel Mac or Linux.

## Verification and limitations

See [VALIDATION.md](VALIDATION.md) for the release checks and their limits. This is an experimental public build. Compilation, focused simulation checks, actual Rust collision probes and fresh resource preparation passed. The owner accepted this build for publication. That is distinct from automated verification: no exhaustive per-reward, cross-platform or FPS certification is claimed. No unattended match was started to claim a playtest.

The broad inherited `make publish-check` is not clean; it reports pre-existing address-shaped literals and test placement across upstream components. The invasion update added none of those findings. Temporary development probes were removed before publication under the repository test policy.

The launcher holds shared renderer locks, allows at most two renderer-bearing engines, and never closes another application or automatically restarts a crashed game. If launch fails, inspect `logs/latest-launch.log` under the runtime folder above. For setup failures, inspect `logs/prepare.log`. Please omit personal paths or account details when posting logs.

## Credits and license

Full attribution: **[CREDITS.md](CREDITS.md)**.

- **vladtrc and IW4L contributors** — standalone MW2 runtime.
- **chasmlol** — 2010 Rust Rewrite Mashup, MinecraftOSS integration and the upstream crossover work.
- **oh-ashen-one** — Rust invasion roster/abilities, survival reward ladder, native Mac input/launch packaging and this public fork.

Code is under **Apache-2.0**, with bundled third-party notices retained; see [LICENSE](LICENSE), [NOTICE](NOTICE), and the [upstream README](docs/UPSTREAM-README.md). Font licenses are shipped with the app. All game trademarks and original content belong to their owners. This project is unofficial and unaffiliated with Activision, Mojang or Microsoft.

For a workstation using a non-default shared renderer coordinator, set `RendererSlotDirectory` in the runtime's local `settings.plist` to that coordinator's existing absolute directory. The launcher refuses conflicting environment overrides or a missing configured protocol. Keep machine-specific paths out of the repository and do not change shared pause/lock files.

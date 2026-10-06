# Set up Rust Minecraft Invasion with your coding agent

Give your agent this repository URL:
**https://github.com/oh-ashen-one/rust-minecraft-invasion**

Copy and paste:

> Help me install and prepare Rust Minecraft Invasion v0.9.0 from
> https://github.com/oh-ashen-one/rust-minecraft-invasion.
> Read README.md and PLAY_WITH_AGENT.md first. Check my operating system and
> processor, use the official release ZIP if my Mac is supported, verify its
> SHA-256 checksum, and help locate my own MW2 (2009) English PC installation.
> Prepare the required Minecraft resources and stop with the launcher ready.
> Do not modify my game files, change gameplay, start a match or play for me.
> Tell me exactly how to open the game and start Rust Invasion.

## Supported ready-to-play download

- Apple silicon Mac (M1 or newer), macOS 14 or later.
- A locally installed, owned **Windows PC copy of Modern Warfare 2 (2009)**,
  with English base-game and multiplayer data. The 2022 game and Campaign
  Remastered do not work as substitutes.
- Internet for the release and initial Minecraft resource preparation.
- Keyboard/mouse or a controller; the native Mac DualSense path does not need
  Steam Input.

Skyrim, Skate 3, Minecraft Java running in the background, Python and Rust are
not required to use the packaged Mac app. A coding agent is optional: the
[README's manual setup](README.md#play-on-an-apple-silicon-mac) is sufficient.
Windows, Linux and Intel Mac are source-build paths, not verified binary targets
for this release. Do not label them supported-to-play based only on a compile.

## Agent setup procedure

1. Verify the actual target computer and OS/architecture. If unsupported, explain
   the source-build limitation before installing unrelated software.
2. Read the [v0.9.0 release](https://github.com/oh-ashen-one/rust-minecraft-invasion/releases/tag/v0.9.0).
   Download `Rust-Minecraft-Invasion-0.9.0-macOS-arm64.zip` and `SHA256SUMS.txt`
   from that release into a new folder. Verify `shasum -a 256 -c SHA256SUMS.txt`.
3. Extract the ZIP and place `Rust Minecraft Invasion.app` in Applications.
   Preserve any existing installation and profiles; do not overwrite a running app.
   The app is ad-hoc signed, not Apple-notarized. If macOS blocks opening it,
   guide the person through Apple's normal Open Anyway flow. Do not disable
   Gatekeeper or remove quarantine attributes for them.
4. Locate the person's MW2 folder. Steam/CrossOver installations are suitable.
   Validate `main/` and `zone/english/mp_rust.ff`. Ask for the location if it
   cannot be identified; do not obtain game archives from third-party sites.
5. Open the launcher, select **Choose MW2 Folder**, and select that installation.
   The launcher reads game files without modifying them.
6. Use **Prepare Files**, then **Check Setup**. Preparation downloads pinned
   Minecraft resources from Mojang and checks hashes; it does not run a match.
7. Stop and hand over: **Open Game Menu → Create Game → Rust Invasion → Rust**.
   The player starts the match, selects **Intervention Quickscope**, and **Equip**.
   **D-pad Right / 4** selects an earned killstreak. Rewards never auto-deploy.

The launcher checks renderer availability and retains a single-instance guard.
Respect any busy/paused message. On workstations with an existing shared renderer
coordinator, configure the documented `RendererSlotDirectory` to that existing
root. Do not clear another process's locks or stop other applications.

## Troubleshooting and development

Runtime files are in
`~/Library/Application Support/Rust Minecraft Invasion/0.8.0/`.
The directory name stays at 0.8.0 to preserve profiles across app updates.
Launch log: `logs/latest-launch.log`. Preparation log: `logs/prepare.log`.
Do not post credentials, account details or unredacted personal paths in issues.
If the launcher reports a crash, inspect the log; do not loop automatic relaunches.

Prefer the published binary for playing. If deliberately building from source,
clone tag `v0.9.0`, read [BUILD.md](docs/BUILD.md), [AGENT.md](AGENT.md) and
[CONTEXT.md](CONTEXT.md), and follow the source-build section of README.md.
Never commit downloaded game data, profiles or extracted scripts. Preserve
[LICENSE](LICENSE), [NOTICE](NOTICE) and [CREDITS.md](CREDITS.md) in redistributions.

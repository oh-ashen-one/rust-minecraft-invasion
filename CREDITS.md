# Credits and provenance

Rust Minecraft Invasion is a community-developed, unofficial survival crossover.
The source and release preserve the upstream licenses and notices. Credits below
distinguish code used in the project from tools and references that were read.

## Project lineage

| Project / contributor | Contribution |
| --- | --- |
| [vladtrc and IW4L contributors](https://github.com/vladtrc/iw4L) | The standalone Rust MW2 runtime: asset readers, rendering, simulation, weapons, GSC execution, menus and supporting systems. |
| [chasmlol — 2010 Rust Rewrite Mashup](https://github.com/chasmlol/2010-rust-rewrite-mashup) | The direct upstream fork, Minecraft integration and inherited skating components. |
| [MinecraftOSS](third_party/minecraftoss/README.md) | Five in-tree engine crates carried by the upstream mashup, recorded there at commit `4013a68`: core, generator, world, player and entities. |
| [Hari / oh-ashen-one](https://github.com/oh-ashen-one) | This invasion fork's direction and owner playtesting; mob roster, respawning/abilities, balance, Intervention Pro class, native killstreak integration, Mac controller/launcher fixes and public packaging developed with AI coding assistance. |

The main project license is [Apache-2.0](LICENSE). [NOTICE](NOTICE) retains the
upstream copyright and provenance statements. Dependencies retain their own
licenses; the app includes collected Rust dependency notices.

## Bundled fonts

- **Oxanium**, The Oxanium Project Authors — [SIL Open Font License 1.1](crates/ui/assets/OFL-Oxanium.txt).
- **GNU FreeFont / FreeMono**, Free Software Foundation — [GPLv3 or later with the font exception](crates/console/assets/COPYING-FreeFont.txt).

Both font license texts ship inside the app.

## Research and development references

Upstream documents consultation of OpenAssetTools, its iw4x-x64/oat fork, IW4x,
KisakCOD and Ghidra. Their links and the scope of that consultation remain in
[NOTICE](NOTICE); they are not bundled with this release.

The native-killstreak work also reviewed
[Universal Modder](https://github.com/rehan-remade/universal-modder),
[AI Game Modding Guides](https://github.com/trevaintdead/ai-game-modding-guides),
[REA](https://github.com/morluto/rea),
[Ghidra MCP](https://github.com/bethington/ghidra-mcp),
[IDA MCP](https://github.com/HexRaysSA/ida-mcp),
[ILSpy](https://github.com/icsharpcode/ilspy),
[Cpp2IL](https://github.com/SamboyCoding/Cpp2IL),
[PortPS5](https://github.com/yuriolive/PortPS5) and
[AnyPS5](https://github.com/boykopovar/AnyPS5).
These were reference/tool evaluations, not code dependencies or endorsements.
No code from those nine projects was imported for this integration.

## Original games and content

**Call of Duty: Modern Warfare 2 (2009)**, its Rust map, weapons, models, scripts,
textures and sounds are original game content from Infinity Ward / Activision.
**Minecraft** and its original content belong to Mojang Studios / Microsoft.
Inherited Skate-related runtime code is credited to the upstream mashup; Skate 3
is an Electronic Arts / Black Box game and is not required for this invasion mode.

No original game archives, executable dumps, decompiled game source, Minecraft
JARs, textures or sounds are distributed in this repository or release ZIP.
Players supply their own MW2 installation; the resource preparer obtains the
pinned Minecraft resources directly from Mojang. The project is not affiliated
with or endorsed by those game publishers or by the reference-tool authors.

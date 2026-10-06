# Native MW2 rewards in Rust Invasion

All fifteen standard MW2 (2009) multiplayer rewards are enabled together.
Minecraft mob deaths caused by player weapons or native support fire advance
one consecutive-kill ladder. Creature-on-creature deaths do not advance it.
Rewards at the same threshold are all granted once per life. Death resets the
ladder; earned items remain in MW2's persistent reward stack.

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


Earning calls the original reward script without invoking its use function.
D-pad Right / keyboard 4 selects the stock killstreak item. Further interaction
is owned by the stock script (for example placement, a laptop or a marker).
The original stack selects the newest earned item first. Fifteen stock reward
icons stack at bottom left, with counts for duplicates; the normal action-slot
icon identifies the next item. The normal earned splash and announcer remain.

The runtime reads scripts, models, weapon definitions, materials and sounds
from the player's own MW2 installation. It does not distribute those assets.
Target adapters add Minecraft mobs only to the relevant helicopter/Harrier
acquisition routines and the native sentry selector. They do not add synthetic
multiplayer clients or modify the gametype's player list. Native weapon rays
respect world cover before hitting mob boxes. Blasts use the originating
weapon/script radius and damage falloff, with line-of-sight checks. UAV contacts
join the normal minimap sweep. The native nuke damages mobs and ends the match.
EMP and Counter-UAV retain their MW2 behavior; they no longer invent magic
creature debuffs. Care packages use the original delivery and use interaction.

The original Rust map, enemy population/respawns/abilities, health adjustments,
Intervention Pro class, controller input and v0.8.1 menu fix remain. The removed
custom support geometry and remote-control systems have no fallback path.

Live acceptance of all reward visuals, sounds, remote controls and delivery
sequences belongs to the player. Compilation and offline checks are documented
in VALIDATION.md and do not establish that every native host API behaves exactly
like the original executable.

## Reference tools reviewed

The owner's corrected references were reviewed on October 6, 2026.
[Universal Modder](https://github.com/rehan-remade/universal-modder)'s
reverse-engineering workflow and the
[AI Game Modding Guides](https://github.com/trevaintdead/ai-game-modding-guides)
IW4L case study support inspecting the original data and keeping that data out
of the public source. No source code from those projects was imported.

[REA](https://github.com/morluto/rea),
[Ghidra MCP](https://github.com/bethington/ghidra-mcp) and
[IDA MCP](https://github.com/HexRaysSA/ida-mcp) are analysis integrations. The
existing native source and MW2 script reader provided the required access here.
[ILSpy](https://github.com/icsharpcode/ilspy) targets .NET and
[Cpp2IL](https://github.com/SamboyCoding/Cpp2IL) targets Unity IL2CPP;
[PortPS5](https://github.com/yuriolive/PortPS5) and
[AnyPS5](https://github.com/boykopovar/AnyPS5) target PS5 executables. Those formats
are outside this MW2 PC-data/Rust-runtime integration. None was installed or
attached to a running game for this change.

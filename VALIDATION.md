# v0.8.0 validation

Verified on an Apple silicon Mac Studio. The checks below do not launch a renderer or inject gameplay input.

- Optimized native Rust workspace build and native AppKit launcher compilation.
- Seventeen focused checks: six existing invasion checks, ten temporary reward/balance tests, and a real input-command decoding test for D-pad Right / key 4: complete threshold ladder, same-threshold pairs, manual queue semantics, death reset with retained rewards, all fifteen deployments, sentry placement rejection, remote aim/cannon switching/exit, Predator impact/exit, supply collection/healing, ten-second nuke and ladder reset, EMP/cover behavior, 300 HP / 75% mob health / incoming-hit grace.
- An actual MW2 Rust data probe exercised each of the fifteen rewards for 250 simulation ticks, checked finite creature positions and remote camera clearance against native collision. All fifteen passed. Standard reward costs were checked against the owned game's killstreak table; no retail tables or probe dumps are distributed.
- Fresh `prepare-invasion` invocation downloaded and prepared the resource set from Mojang without starting a game. Existing downloader hash verification remains enabled.
- Public source export was checked for original game archives/executables, credentials, private repository dependencies and author-machine paths. This repository is a clean snapshot and does not expose private development history.
- Packaging checks the app signature and the actual signed game CLI-loader path, which exits before renderer/network/match initialization.

Temporary new tests/probes were removed before publishing, as required by the source repository's explicit test policy. The existing test suite was preserved. The broad source `publish-check` remains nonzero on 601 inherited findings (101 address-shaped/decompiler strings and 500 test-placement findings); comparison against the preceding source snapshot found zero new findings. The checker was not disabled or weakened.

Pending: new live-game visual review, controller feel, killstreak balancing, rendered FPS, multiplayer behavior and non-Apple-silicon runtime tests. Support geometry/effects and behavior are custom arena adaptations of the fifteen standard MW2 rewards. The public build is an experimental prerelease.

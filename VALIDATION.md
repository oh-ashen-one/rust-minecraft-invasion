# v0.9.0 native killstreak candidate

The owner confirmed the v0.8.1 base works and looks good. That acceptance
predates this change. No live game input or rendered match was used here.

- Full Apple-silicon `play` build of the launcher/runtime passed.
- The v0.9.0 app passed ad-hoc code-signature, bundle and CLI loader checks;
  its installed binary hash matches build metadata for code commit 52a6f32.
  A separate candidate app and portable ZIP were saved. The accepted v0.8.1
  binary remains unchanged. No renderer was launched.
- The runtime's real asset loader read the owned MW2 Rust/common/patch data.
  The complete game-script graph plus the invasion HUD/target adaptations
  compiled against the native host catalog.
- All fifteen ladder entries matched the stock table's costs and had a stock
  item weapon and reward icon. The ladder awarded each reward once through 25
  kills, added no duplicates through 50, retained pending rewards on life reset,
  and earned a new UAV after three kills in the next life.
- Native action-slot input now reaches the original scripts. The cube support
  renderer, custom remote cameras and fake reward activation path are removed.
- Temporary inspection examples and original-data extracts are not published.
- `publish-check` still reports 601 inherited findings: 101 address/decompiler
  shapes and 500 test-placement findings. Every reported source line exists in
  the base commit; this change adds none. The checker was not weakened.

These are build and offline checks. Original helicopter flight/appearance,
care-package delivery/collection, remote weapons, earned notifications,
bottom-left icon spacing, sound and all reward activation sequences still need
player validation in the candidate. The standalone runtime's native host APIs
remain experimental; source/script compilation does not prove full retail parity.

---

# v0.8.1 menu regression checks

The owner encounter log showed `team_marinesopfor back` followed by repeated
`autoassign`, `axis`, `allies` and `popup_endgame endround` responses, with no
successful join/spawn. Two temporary tests reproduced the bridge failures.
After the fix, six checks passed: Back then Auto-Assign, End Round while a
spectator, client responses waiting for player begin, automatic answers waiting
for the correct menu, End Round bypassing a blocked automatic join, and explicit
Back cancelling an automatic team choice without marking joined.

Four isolated, non-GUI launcher checks verified persistent shared-directory
selection through an exclusive-lock refusal, PAUSED handling, conflicting roots,
and missing-root refusal. They used temporary fixture files and did not acquire
any live renderer slot. Native launcher syntax and the full native build passed.
New temporary tests are removed before push under the repository test policy.
No live-game replay or subjective acceptance is claimed. The game is left closed.

---

# v0.8.0 validation

Verified on an Apple silicon Mac Studio. The checks below do not launch a renderer or inject gameplay input.

- Optimized native Rust workspace build and native AppKit launcher compilation.
- Eighteen focused checks: six existing invasion checks, ten temporary reward/balance tests, a real input-command decoding test for D-pad Right / key 4, and a headless startup check for the radar/reticle HUD without renderer or system-query conflicts: complete threshold ladder, same-threshold pairs, manual queue semantics, death reset with retained rewards, all fifteen deployments, sentry placement rejection, remote aim/cannon switching/exit, Predator impact/exit, supply collection/healing, ten-second nuke and ladder reset, EMP/cover behavior, 300 HP / 75% mob health / incoming-hit grace.
- An actual MW2 Rust data probe exercised each of the fifteen rewards for 250 simulation ticks, checked finite creature positions and remote camera clearance against native collision. All fifteen passed. Standard reward costs were checked against the owned game's killstreak table; no retail tables or probe dumps are distributed.
- Fresh `prepare-invasion` invocation downloaded and prepared the resource set from Mojang without starting a game. Existing downloader hash verification remains enabled.
- Public source export was checked for original game archives/executables, credentials, private repository dependencies and author-machine paths. This repository is a clean snapshot and does not expose private development history.
- Packaging checks the app signature and the actual signed game CLI-loader path, which exits before renderer/network/match initialization.

Temporary new tests/probes were removed before publishing, as required by the source repository's explicit test policy. The existing test suite was preserved. The broad source `publish-check` remains nonzero on 601 inherited findings (101 address-shaped/decompiler strings and 500 test-placement findings); comparison against the preceding source snapshot found zero new findings. The checker was not disabled or weakened.

Pending: new live-game visual review, controller feel, killstreak balancing, rendered FPS, multiplayer behavior and non-Apple-silicon runtime tests. Support geometry/effects and behavior are custom arena adaptations of the fifteen standard MW2 rewards. The public build is an experimental prerelease.

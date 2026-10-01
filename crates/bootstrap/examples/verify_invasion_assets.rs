//! Offline data conversion/validation only. No Bevy App, window or renderer.
fn main() {
    let games = asset_transport::games_root_from_env().expect("MW2 data path");
    asset_game::ui_games_root(&games).expect("MW2 multiplayer menu data");
    let found = assets::find_zone_file(&games, "iw4:mp_rust").expect("original Rust fastfile");
    assert_eq!(asset_transport::peek_zone_version(&found.path), Some(0x114));
    let common = asset_transport::find_runtime_common_mp(&games, &found.path).map(|zone| zone.path);
    let loaded = match bevy::tasks::futures_lite::future::block_on(assets::load_prepared_match(
        Ok(found.path),
        common,
        asset_transport::LoadProgress::default(),
    )) {
        assets::MatchLoadOutcome::Ready(prepared) => prepared,
        assets::MatchLoadOutcome::Canceled => panic!("offline map conversion canceled"),
    };
    let clip = loaded.clip.expect("native arena collision");
    let spawns = &loaded.prepared_map.spawns;
    assert!(!spawns.is_empty(), "Rust has no authored spawns");
    assert!(
        !clip.brushes.is_empty(),
        "Rust has no native collision brushes"
    );
    assert!(
        loaded.world.draw.is_some(),
        "Rust has no native world draw product"
    );
    let mut safe = 0;
    for spawn in spawns {
        let p = spawn.origin;
        let hit = clip.sweep_box(
            [p[0], p[1], p[2] + 105.0],
            [p[0], p[1], p[2] - 395.0],
            [-12.0, -12.0, 0.0],
            [12.0, 12.0, 70.0],
            0x0281_0011,
        );
        if !hit.startsolid && !hit.allsolid && hit.fraction < 1.0 && hit.normal[2] > 0.5 {
            safe += 1;
        }
    }
    assert!(
        safe >= 4,
        "too few ground-supported enemy spawn points: {safe}"
    );
    let collision = |a: bevy::math::Vec3, b: bevy::math::Vec3, r: f32, h: f32| {
        let hit = clip.sweep_box(
            a.to_array(),
            b.to_array(),
            [-r, -r, 0.0],
            [r, r, h],
            0x0281_0011,
        );
        sim::invasion::Sweep {
            fraction: hit.fraction,
            normal: bevy::math::Vec3::from_array(hit.normal),
            solid: hit.startsolid || hit.allsolid,
        }
    };
    let mut arena = sim::invasion::Arena::new(spawns.iter().map(|s| s.origin).collect());
    let player = bevy::math::Vec3::from_array(spawns[0].origin);
    arena.tick(0.05, player, &collision);
    assert_eq!(
        arena.mobs.len(),
        arena.target_population(),
        "every enemy must find a safe native Rust spawn"
    );
    let start = std::time::Instant::now();
    for _ in 0..40 {
        arena.tick(0.05, player, &collision);
    }
    println!(
        "PASS: full roster safe spawn count={}, native 20Hz collision tick mean={:.2}ms (CPU-only, not game FPS)",
        arena.target_population(),
        start.elapsed().as_secs_f64() * 1000.0 / 40.0
    );
    println!(
        "PASS: original Rust world, native collision, {} authored spawns, {safe} supported enemy positions, {} weapon definitions",
        spawns.len(),
        loaded.weapons.len()
    );
}

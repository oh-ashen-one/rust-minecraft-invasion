//! An opt-in Minecraft enemy overlay on the original MW2 Rust arena.
//! Assets come from Mojang's resource pack; AI and boss behavior are custom.
use crate::minecraft_world::MinecraftWorldView;
use bevy::prelude::*;
use minecraft_terrain::{
    mesh::{Atlas, ChunkMesh},
    pack::{PackStack, ResourceId},
    scene::HandcraftedScene,
};
use sim::invasion::{Arena, Family, Kind, Mob, Sweep};
mod abilities;
use std::sync::{Arc, mpsc};

struct Art {
    packs: PackStack,
    atlas: Arc<Atlas>,
    sounds: crate::minecraft_sounds::Sounds,
    /// Bind-group slots remain valid when sky/crack passes are unused.
    blank: Arc<image::RgbaImage>,
    white_uv: [f32; 2],
}
#[derive(Default)]
struct Runtime {
    arena: Option<Arena>,
    loading: Option<mpsc::Receiver<Result<Art, String>>>,
    art: Option<Art>,
    wanted: bool,
    requested: bool,
    accumulator: f32,
    sound_clock: f32,
    error: Option<String>,
}
#[derive(Component)]
struct InvasionStatus;
#[derive(Component)]
struct InvasionRadar;
#[derive(Component)]
struct RadarDot(usize);
#[derive(Component)]
struct NukeFlash;
#[derive(Component)]
struct RemoteReticle;

fn enabled() -> bool {
    std::env::var("IW4L_RUST_INVASION").as_deref() == Ok("1")
}
pub(crate) fn register(app: &mut App) {
    if !enabled() {
        return;
    }
    app.insert_non_send(Runtime::default())
        .add_systems(Startup, status)
        .add_systems(
            Update,
            update
                .after(crate::minecraft_world::update)
                .before(crate::sync_camera_from_presented)
                .in_set(frame::ClientSet::Present),
        );
}
fn status(mut commands: Commands) {
    commands.spawn((
        RemoteReticle,
        Visibility::Hidden,
        Text::new("+"),
        TextFont {
            font_size: bevy::text::FontSize::Px(34.0),
            ..default()
        },
        TextColor(Color::srgb(0.4, 1.0, 0.45)),
        Node {
            position_type: PositionType::Absolute,
            left: percent(49.5),
            top: percent(48.0),
            ..default()
        },
        GlobalZIndex(105),
    ));
    commands.spawn((
        NukeFlash,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        BackgroundColor(Color::NONE),
        GlobalZIndex(90),
    ));
    commands
        .spawn((
            InvasionRadar,
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                right: px(20),
                top: px(65),
                width: px(170),
                height: px(170),
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.06, 0.03, 0.8)),
            GlobalZIndex(101),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("UAV"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            panel.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(83),
                    top: px(83),
                    width: px(5),
                    height: px(5),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
            ));
            for i in 0..sim::invasion::Kind::ALL.len() * 2 {
                panel.spawn((
                    RadarDot(i),
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(4),
                        height: px(4),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(1.0, 0.18, 0.1)),
                    Visibility::Hidden,
                ));
            }
        });
    commands.spawn((
        InvasionStatus,
        Text::new("RUST · MINECRAFT INVASION  |  Create Game → Rust"),
        TextFont {
            font_size: bevy::text::FontSize::Px(19.0),
            ..default()
        },
        TextColor(Color::srgb(0.94, 0.87, 0.67)),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(20),
            ..default()
        },
        GlobalZIndex(100),
    ));
}
fn load_art() -> Result<Art, String> {
    let root = assets::minecraft_map::root().ok_or_else(assets::minecraft_setup::status)?;
    let packs = PackStack::open(vec![root.join("resourcepacks/local/minecraft-26.3")])
        .map_err(|e| e.to_string())?;
    let dragon =
        ResourceId::parse("minecraft:entity/enderdragon/dragon").map_err(|e| e.to_string())?;
    if packs.texture(&dragon).map_err(|e| e.to_string())?.is_none() {
        return Err("Minecraft resource pack is missing the Ender Dragon skin".into());
    }
    let built = minecraft_terrain::mesh::build(&HandcraftedScene::default(), &packs)
        .map_err(|e| e.to_string())?;
    let sounds = crate::minecraft_sounds::Sounds::load(&packs);
    let white_uv = abilities::white_texel(&built.atlas);
    Ok(Art {
        white_uv,
        packs,
        atlas: built.atlas,
        sounds,
        blank: Arc::new(image::RgbaImage::from_pixel(
            1,
            1,
            image::Rgba([255, 255, 255, 255]),
        )),
    })
}
#[allow(clippy::too_many_arguments)]
fn update(
    time: Res<Time>,
    mut installed: MessageReader<frame::MatchInstalled>,
    mut torn: MessageReader<frame::MatchTornDown>,
    local: Res<net::LocalPresentClient>,
    presented: Res<net::PresentedSnapshot>,
    authority: Option<Res<net::AuthorityWorld>>,
    mut view: ResMut<MinecraftWorldView>,
    mut runtime: NonSendMut<Runtime>,
    mut texts: Query<&mut Text, With<InvasionStatus>>,
    mut sounds: ResMut<audio::McSoundQueue>,
    actions: Option<Res<net::ClientActionInput>>,
    mut radar: Query<
        &mut Visibility,
        (
            With<InvasionRadar>,
            Without<RadarDot>,
            Without<RemoteReticle>,
        ),
    >,
    mut dots: Query<
        (&RadarDot, &mut Node, &mut Visibility),
        (Without<InvasionRadar>, Without<RemoteReticle>),
    >,
    mut flash: Query<&mut BackgroundColor, With<NukeFlash>>,
    mut reticles: Query<
        &mut Visibility,
        (
            With<RemoteReticle>,
            Without<InvasionRadar>,
            Without<RadarDot>,
        ),
    >,
) {
    for mut visibility in &mut reticles {
        *visibility = Visibility::Hidden;
    }
    for mut visibility in &mut radar {
        *visibility = Visibility::Hidden;
    }
    for mut color in &mut flash {
        *color = BackgroundColor(Color::NONE);
    }
    for _ in torn.read() {
        sim::voxel::set_invasion_remote(None);
        runtime.wanted = false;
        runtime.arena = None;
        runtime.accumulator = 0.0;
        view.mobs_only = false;
        view.active = false;
        view.entity_meshes = Default::default();
        sim::voxel::deactivate();
    }
    for event in installed.read() {
        sim::voxel::set_invasion_remote(None);
        runtime.wanted = event.zone.rsplit(':').next() == Some("mp_rust");
        runtime.arena = None;
        runtime.accumulator = 0.0;
        runtime.error = None;
        if runtime.wanted {
            assets::minecraft_map::prepare();
        }
    }
    // This prepares resources without starting a match or spawning a player.
    if !runtime.requested && assets::minecraft_map::root().is_some() {
        let (send, recv) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = send.send(load_art());
        });
        runtime.loading = Some(recv);
        runtime.requested = true;
    }
    if let Some(receiver) = runtime.loading.as_ref() {
        if let Ok(result) = receiver.try_recv() {
            runtime.loading = None;
            match result {
                Ok(art) => runtime.art = Some(art),
                Err(error) => {
                    diag::warn!(World, "Rust invasion assets: {error}");
                    runtime.error = Some(error);
                }
            }
        }
    }
    let player = presented.player(local.0);
    let alive = player.is_some_and(|ps| ps.pm_type == 0);
    let mut label = if runtime.wanted {
        if let Some(error) = &runtime.error {
            format!("INVASION ASSETS: {error}")
        } else if runtime.art.is_none() {
            format!(
                "Preparing Minecraft enemies: {}",
                assets::minecraft_setup::status()
            )
        } else if !alive {
            "RUST · MINECRAFT INVASION  |  Choose a class to begin".into()
        } else {
            String::new()
        }
    } else {
        "RUST · MINECRAFT INVASION  |  Create Game → Rust".into()
    };
    if runtime.wanted && runtime.art.is_some() && alive {
        if let (Some(authority), Some(ps)) = (authority, player) {
            if runtime.arena.is_none() {
                runtime.arena = Some(Arena::new(authority.0.authored_spawn_origins()));
                sim::voxel::activate_mobs([0.0; 3]);
                view.generation += 1;
                diag::info!(
                    World,
                    "Rust invasion ready; IW4 terrain and collision retained"
                );
            }
            let collision = |a: Vec3, b: Vec3, r: f32, h: f32| {
                let mask = if r > 0.0 || h > 0.0 { 0x0281_0011 } else { 1 };
                let trace = authority.0.trace_static_world(
                    a.to_array(),
                    b.to_array(),
                    [-r, -r, 0.0],
                    [r, r, h],
                    mask,
                );
                Sweep {
                    fraction: trace.fraction,
                    normal: Vec3::from_array(trace.normal),
                    solid: trace.startsolid != 0 || trace.allsolid != 0,
                }
            };
            let Runtime {
                arena,
                art,
                accumulator,
                sound_clock,
                ..
            } = &mut *runtime;
            let arena = arena.as_mut().unwrap();
            let art = art.as_mut().unwrap();
            for event in sim::voxel::take_events() {
                match event {
                    sim::voxel::VoxelEvent::StreakAction { client } if client == local.0.0 => {
                        arena.use_streak(Vec3::from_array(ps.origin), ps.viewangles, &collision);
                    }
                    sim::voxel::VoxelEvent::MobShot { key, damage, .. } => {
                        let target = arena
                            .mobs
                            .iter()
                            .find(|m| m.id == key && m.health > 0.0)
                            .map(|m| (m.kind, m.position, m.health));
                        arena.damage(key, damage);
                        if let Some((kind, position, health)) = target {
                            let suffix = if damage >= health {
                                "death"
                            } else if kind == Kind::Creaking {
                                "attack"
                            } else {
                                "hurt"
                            };
                            let event = format!("entity.{}.{}", kind.sound_prefix(), suffix);
                            art.sounds
                                .play(&art.packs, &event, Some(position), 0.8, 1.0);
                        }
                    }
                    sim::voxel::VoxelEvent::Explosion { center } => arena.explode(
                        Vec3::from_array(sim::voxel::to_map([0.0; 3], center)),
                        &collision,
                    ),
                    _ => {}
                }
            }
            let feet = Vec3::from_array(ps.origin);
            arena.streaks.view_angles = ps.viewangles;
            arena.streaks.fire = actions.as_ref().is_some_and(|a| a.client.kb.attack.active);
            arena.streaks.alt = actions.as_ref().is_some_and(|a| a.client.kb.speed.active);
            if let Some(remote) = arena.streaks.remote() {
                sim::voxel::set_invasion_remote(Some(sim::voxel::InvasionRemote {
                    client: local.0.0,
                    origin: remote.position.to_array(),
                    angles: remote.aim,
                    fov: 65.0,
                }));
            }
            *accumulator += time.delta_secs().min(0.15);
            while *accumulator >= 1.0 / 20.0 {
                *accumulator -= 1.0 / 20.0;
                if authority.0.phase() == sim::MatchPhase::Playing {
                    for attack in arena.tick(1.0 / 20.0, feet, &collision) {
                        sim::voxel::push_player_impact(sim::voxel::PlayerImpact {
                            client: local.0.0,
                            amount: attack.damage,
                            from: Some(attack.from),
                            impulse: attack.impulse.to_array(),
                            nonlethal: attack.nonlethal,
                        });
                    }
                }
            }
            sim::voxel::set_invasion_remote(arena.streaks.remote().map(|r| {
                sim::voxel::InvasionRemote {
                    client: local.0.0,
                    origin: r.position.to_array(),
                    angles: r.aim,
                    fov: 65.0,
                }
            }));
            if arena.streaks.remote().is_some() {
                for mut visibility in &mut reticles {
                    *visibility = Visibility::Visible;
                }
            }
            if std::mem::take(&mut arena.streaks.heal) {
                sim::voxel::invasion_heal(local.0.0);
            }
            if arena.streaks.radar_until > arena.clock {
                for mut visibility in &mut radar {
                    *visibility = Visibility::Visible;
                }
                let yaw = ps.viewangles[1].to_radians();
                for (dot, mut node, mut visibility) in &mut dots {
                    *visibility = Visibility::Hidden;
                    if let Some(mob) = arena.mobs.get(dot.0).filter(|m| m.health > 0.0) {
                        let d = mob.position - feet;
                        let x = (d.x * yaw.sin() - d.y * yaw.cos()) / 1400.0;
                        let y = -(d.x * yaw.cos() + d.y * yaw.sin()) / 1400.0;
                        node.left = px(83.0 + x.clamp(-1.0, 1.0) * 78.0);
                        node.top = px(83.0 + y.clamp(-1.0, 1.0) * 78.0);
                        *visibility = Visibility::Visible;
                    }
                }
            }
            let nuke_flash = ((arena.streaks.nuke_flash_until - arena.clock) / 1.3).clamp(0.0, 0.8);
            for mut color in &mut flash {
                *color = BackgroundColor(Color::srgba(1.0, 0.98, 0.85, nuke_flash));
            }
            for cue in arena.take_sounds() {
                art.sounds
                    .play(&art.packs, cue.name, Some(cue.position), cue.volume, 1.0);
            }
            *sound_clock += time.delta_secs();
            if *sound_clock > 4.0 {
                *sound_clock = 0.0;
                let count = arena.mobs.len().max(1);
                let start = (arena.clock as usize) * 4 % count;
                for offset in 0..4 {
                    if let Some(mob) = arena
                        .mobs
                        .get((start + offset) % count)
                        .filter(|m| m.health > 0.0)
                    {
                        let suffix = if mob.kind == Kind::Dragon {
                            "growl"
                        } else {
                            "ambient"
                        };
                        let event = format!("entity.{}.{}", mob.kind.sound_prefix(), suffix);
                        if art.sounds.has_event(&event) {
                            art.sounds.play(
                                &art.packs,
                                &event,
                                Some(mob.position),
                                if mob.kind.boss() { 2.0 } else { 0.5 },
                                1.0,
                            );
                        }
                    }
                }
            }
            sounds.0.append(&mut art.sounds.queued);
            sim::voxel::set_mob_boxes(
                arena
                    .mobs
                    .iter()
                    .filter(|m| m.health > 0.0)
                    .flat_map(|m| {
                        m.hitboxes().into_iter().map(move |bb| {
                            let a = sim::voxel::to_block([0.0; 3], [bb[0], bb[1], bb[2]]);
                            let b = sim::voxel::to_block([0.0; 3], [bb[3], bb[4], bb[5]]);
                            (
                                m.id,
                                [
                                    a[0].min(b[0]),
                                    a[1].min(b[1]),
                                    a[2].min(b[2]),
                                    a[0].max(b[0]),
                                    a[1].max(b[1]),
                                    a[2].max(b[2]),
                                ],
                            )
                        })
                    })
                    .collect(),
            );
            let mut mesh = ChunkMesh::default();
            for mob in arena.mobs.iter().filter(|m| m.health > 0.0) {
                let mut pose = mob.clone();
                pose.position = mob
                    .previous
                    .lerp(mob.position, (*accumulator * 20.0).clamp(0.0, 1.0));
                pose.age += *accumulator;
                let first = mesh.vertices.len();
                let flash = pose.kind.explosive() && pose.fuse > 0.0;
                if flash {
                    pose.scale *= 1.0 + 0.18 * (pose.fuse / 1.5).powi(2);
                }
                model(&mut mesh, &pose, &art.atlas);
                if flash && ((pose.fuse * (8.0 + pose.fuse * 8.0)) as u32) % 2 == 0 {
                    for vertex in &mut mesh.vertices[first..] {
                        vertex.color[3] = 0.25;
                    }
                }
                abilities::held_weapon(&mut mesh, &pose, art.white_uv);
            }
            abilities::append(&mut mesh, arena, art.white_uv);
            abilities::supports(&mut mesh, arena, art.white_uv);
            view.active = true;
            view.mobs_only = true;
            view.origin = [0.0; 3];
            view.atlas = Some(art.atlas.clone());
            view.visible.clear();
            view.uploads.clear();
            view.clouds = None;
            view.celestial = Some(art.blank.clone());
            view.crack_texture = Some(art.blank.clone());
            view.light_volume = None;
            view.environment = [[0.0; 4]; 16];
            view.environment[5] = [0.75, 0.68, 0.55, 200000.0];
            view.environment[6] = [1.0, 0.94, 0.82, 0.8];
            view.environment[13] = [100000.0, 200000.0, 100000.0, 200000.0];
            view.environment[14] = [0.18, 0.15, 0.12, 0.0];
            view.environment[15] = [1.0, 0.85, 0.7, 0.1];
            view.entity_meshes = [
                (bytemuck::cast_slice(&mesh.vertices).to_vec(), mesh.indices),
                Default::default(),
                Default::default(),
                Default::default(),
            ];
            let boss = arena.boss().map_or(String::new(), |m| {
                format!(
                    "  |  ENDER DRAGON {:.0}/{:.0}",
                    m.health.max(0.0),
                    m.max_health
                )
            });
            let condition = if arena.poisoned() {
                "  |  POISONED"
            } else {
                ""
            };
            let queue = arena.streaks.queue.front().map_or("NONE", |r| r.name());
            let next = arena
                .streaks
                .next_cost()
                .map_or("ALL REWARDS EARNED".to_owned(), |cost| {
                    format!("NEXT REWARD: {cost} KILLS")
                });
            let banner = if arena.streaks.banner_until > arena.clock {
                arena.streaks.banner.as_str()
            } else {
                ""
            };
            let remote = arena
                .streaks
                .remote()
                .map(|r| {
                    use sim::invasion::streaks::Reward;
                    let controls = match r.reward {
                        Reward::Predator => "R2 / CLICK: BOOST".to_owned(),
                        Reward::Ac130 => format!(
                            "{}  |  R2 / CLICK: FIRE  |  L2 / RIGHT CLICK: SWITCH CANNON",
                            ["25 MM", "40 MM", "105 MM"][r.mode as usize]
                        ),
                        _ => "R2 / CLICK: FIRE".to_owned(),
                    };
                    format!(
                        "{}  {:.0}s  |  {}  |  D-pad Right / 4: EXIT",
                        r.reward.name(),
                        (r.duration - r.age).ceil(),
                        controls
                    )
                })
                .unwrap_or_default();
            let nuke = arena
                .streaks
                .active
                .iter()
                .find(|s| s.reward == sim::invasion::streaks::Reward::Nuke)
                .map(|s| format!("TACTICAL NUKE IN {:.0}", (10.0 - s.age).ceil()))
                .unwrap_or_default();
            let disruption = if arena.streaks.emp_until > arena.clock {
                "EMP: ENEMY PROJECTILES DISABLED"
            } else if arena.streaks.jam_until > arena.clock {
                "COUNTER-UAV: ENEMY AIM DISRUPTED"
            } else {
                ""
            };
            label = format!(
                "RUST INVASION  |  WAVE {}  |  {} ENEMIES  |  {} RESPAWNING  |  {} KILLS{}{}",
                arena.wave,
                arena.mobs.iter().filter(|m| m.health > 0.0).count(),
                arena.pending_respawns(),
                arena.kills,
                boss,
                condition
            );
            label.push_str(&format!("\nHEALTH {}/{}  |  STREAK {}  |  {}\nD-pad Right / 4: {}  |  {} REWARDS QUEUED\n{}\n{}\n{}\n{}",ps.health,ps.max_health,arena.streaks.count,next,queue,arena.streaks.queue.len(),banner,remote,nuke,disruption));
        }
    } else if runtime.wanted && !alive {
        sim::voxel::set_invasion_remote(None);
        // Pause AI on death/class selection and clear targets. Retain the
        // wave so respawning cannot duplicate bosses.
        sim::voxel::set_mob_boxes(Vec::new());
        if let Some(arena) = &mut runtime.arena {
            arena.clear_player_effects();
        }
    }
    for mut text in &mut texts {
        *text = Text::new(label.clone());
    }
}

type Part = ([f32; 3], [f32; 3], [f32; 2], [f32; 3]);
fn model(mesh: &mut ChunkMesh, mob: &Mob, atlas: &Atlas) {
    let id = ResourceId::parse(&format!("minecraft:entity/{}", mob.kind.skin())).unwrap();
    let region = atlas.entity_region(&id);
    let block = sim::voxel::to_block([0.0; 3], mob.position.to_array());
    let mut feet = glam::DVec3::from_array(block);
    if mob.kind.family() == Family::Slime {
        feet.y += (mob.age * 7.0).sin().max(0.0) as f64 * 0.35 * mob.scale as f64;
    }
    let rotation = glam::Quat::from_rotation_y(mob.yaw - std::f32::consts::FRAC_PI_2);
    let tint = if mob.hurt > 0.0 {
        [1.0, 0.35, 0.35]
    } else if mob.kind.explosive() && mob.fuse > 0.0 && ((mob.age * 8.0) as i32) % 2 == 0 {
        [1.0, 1.0, 1.0]
    } else if mob.kind == Kind::ChargedCreeper {
        [0.55, 0.8, 1.0]
    } else {
        [1.0; 3]
    };
    let size = mob.kind.texture_size();
    let mut emit = |part: Part, pose: glam::Quat| {
        let (from, to, uv, pivot) = part;
        let dims = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
        let fit = ((size[0] - uv[0]).max(1.0) / (2.0 * (dims[0] + dims[2]).max(1.0)))
            .min((size[1] - uv[1]).max(1.0) / (dims[1] + dims[2]).max(1.0))
            .min(1.0);
        let uv_dims = Some([dims[0] * fit, dims[1] * fit, dims[2] * fit]);
        minecraft_terrain::cow_render::cube_tinted_pose_mirror(
            mesh, feet, rotation, mob.scale, region, 15.0, 0.0, from, to, uv, pivot, pose, tint,
            size, uv_dims, false,
        );
    };
    let swing = (mob.walk * 0.6662).sin() * 0.65;
    let pitch = glam::Quat::from_rotation_x;
    match mob.kind.family() {
        Family::Humanoid | Family::TallHumanoid => {
            let ender = mob.kind == Kind::Enderman;
            let slender = mob.kind != Kind::Zombie;
            let width = if slender { 1.0 } else { 2.0 };
            let length = if ender { 28.0 } else { 12.0 };
            let lift = if ender { -14.0 } else { 0.0 };
            emit(
                ([-4., -8., -4.], [4., 0., 4.], [0., 0.], [0., lift, 0.]),
                glam::Quat::IDENTITY,
            );
            emit(
                ([-4., 0., -2.], [4., 12., 2.], [16., 16.], [0., lift, 0.]),
                glam::Quat::IDENTITY,
            );
            for (side, phase) in [(-1.0, 1.0), (1.0, -1.0)] {
                emit(
                    (
                        [-width, -2., -width],
                        [width, length - 2., width],
                        [40., 16.],
                        [side * 5., lift + 2., 0.],
                    ),
                    pitch(if mob.windup > 0.0 {
                        -1.5
                    } else if mob.kind == Kind::Zombie {
                        -1.3 + swing * 0.25
                    } else {
                        swing * phase
                    }),
                );
                emit(
                    (
                        [-width, 0., -width],
                        [width, length, width],
                        [0., 16.],
                        [side * 2., lift + 12., 0.],
                    ),
                    pitch(swing * phase),
                );
            }
        }
        Family::Creeper => {
            emit(
                ([-4., -8., -4.], [4., 0., 4.], [0., 0.], [0., 6., 0.]),
                glam::Quat::IDENTITY,
            );
            emit(
                ([-4., 0., -2.], [4., 12., 2.], [16., 16.], [0., 6., 0.]),
                glam::Quat::IDENTITY,
            );
            for (i, p) in [
                [-2., 18., -4.],
                [2., 18., -4.],
                [-2., 18., 4.],
                [2., 18., 4.],
            ]
            .into_iter()
            .enumerate()
            {
                emit(
                    ([-2., 0., -2.], [2., 6., 2.], [0., 16.], p),
                    pitch(if i % 2 == 0 { swing } else { -swing }),
                );
            }
        }
        Family::Spider => {
            emit(
                ([-4., -4., -8.], [4., 4., 0.], [32., 4.], [0., 16., -3.]),
                glam::Quat::IDENTITY,
            );
            emit(
                ([-3., -3., -3.], [3., 3., 3.], [0., 0.], [0., 16., 0.]),
                glam::Quat::IDENTITY,
            );
            emit(
                ([-5., -4., -6.], [5., 4., 6.], [0., 12.], [0., 16., 8.]),
                glam::Quat::IDENTITY,
            );
            for side in [-1.0, 1.0] {
                for i in 0..4 {
                    let yaw = (i as f32 - 1.5) * 0.3 + (mob.walk * 0.8 + i as f32).sin() * 0.2;
                    emit(
                        (
                            [0., -1., -1.],
                            [16., 1., 1.],
                            [18., 0.],
                            [side * 3., 16., (i as f32 - 1.5) * 3.],
                        ),
                        glam::Quat::from_euler(
                            glam::EulerRot::YXZ,
                            side * yaw,
                            0.0,
                            side * 0.55
                                + if side < 0.0 {
                                    std::f32::consts::PI
                                } else {
                                    0.0
                                },
                        ),
                    );
                }
            }
        }
        Family::Slime => {
            emit(
                ([-4., -8., -4.], [4., 0., 4.], [0., 0.], [0., 24., 0.]),
                glam::Quat::IDENTITY,
            );
            emit(
                ([-3., -7., -3.], [3., -1., 3.], [0., 16.], [0., 24., 0.]),
                glam::Quat::IDENTITY,
            );
        }
        Family::Dragon => {
            // Articulated neck, jaw, wings, legs and tail use the dragon
            // skin's canonical 256px regions. This is a custom arena boss,
            // not the vanilla End fight's phase machine.
            emit(
                ([-12., -8., -24.], [12., 8., 24.], [0., 0.], [0., -12., 0.]),
                glam::Quat::IDENTITY,
            );
            let flap = (mob.age * 4.0).sin();
            for side in [-1.0, 1.0] {
                let wing = glam::Quat::from_rotation_z(side * (0.25 + flap * 0.35));
                emit(
                    (
                        [0., -2., -8.],
                        [42., 2., 20.],
                        [112., 0.],
                        [side * 10., -12., 0.],
                    ),
                    if side < 0.0 {
                        glam::Quat::from_rotation_y(std::f32::consts::PI) * wing
                    } else {
                        wing
                    },
                );
                emit(
                    (
                        [0., -0.3, -8.],
                        [50., 0.3, 20.],
                        [112., 56.],
                        [side * 40., -12. + flap * 12., 0.],
                    ),
                    if side < 0.0 {
                        glam::Quat::from_rotation_y(std::f32::consts::PI) * wing
                    } else {
                        wing
                    },
                );
                for z in [-12., 14.] {
                    emit(
                        (
                            [-3., 0., -3.],
                            [3., 14., 3.],
                            [112., 104.],
                            [side * 12., -6., z],
                        ),
                        pitch(0.55 + flap * 0.15),
                    );
                }
            }
            for i in 0..4 {
                emit(
                    (
                        [-5., -5., -6.],
                        [5., 5., 6.],
                        [192., 104.],
                        [0., -14., -28. - i as f32 * 9.],
                    ),
                    pitch((mob.age * 2.0 + i as f32 * 0.4).sin() * 0.12),
                );
            }
            emit(
                ([-8., -8., -16.], [8., 8., 0.], [0., 0.], [0., -14., -56.]),
                glam::Quat::IDENTITY,
            );
            emit(
                ([-6., 0., -16.], [6., 4., 0.], [176., 65.], [0., -8., -56.]),
                pitch(0.15 + (mob.age * 3.0).sin() * 0.1),
            );
            for i in 0..7 {
                emit(
                    (
                        [-4., -4., -6.],
                        [4., 4., 6.],
                        [192., 104.],
                        [
                            (mob.age * 1.7 + i as f32 * 0.4).sin() * i as f32 * 1.4,
                            -12.,
                            28. + i as f32 * 10.,
                        ],
                    ),
                    glam::Quat::from_rotation_y((mob.age * 1.7 + i as f32 * 0.4).sin() * 0.18),
                );
            }
        }
        Family::Illager | Family::Witch => {
            emit(
                ([-4., -10., -4.], [4., 0., 4.], [0., 0.], [0., 0., 0.]),
                glam::Quat::IDENTITY,
            );
            emit(
                ([-1., -2., -7.], [1., 2., -4.], [24., 0.], [0., -2., 0.]),
                glam::Quat::IDENTITY,
            );
            emit(
                ([-4., 0., -2.], [4., 12., 2.], [16., 16.], [0., 0., 0.]),
                glam::Quat::IDENTITY,
            );
            for (side, phase) in [(-1., 1.), (1., -1.)] {
                emit(
                    (
                        [-2., 0., -2.],
                        [2., 12., 2.],
                        [0., 16.],
                        [side * 2., 12., 0.],
                    ),
                    pitch(swing * phase),
                );
                emit(
                    (
                        [-2., -2., -2.],
                        [2., 10., 2.],
                        [40., 16.],
                        [side * 5., 2., 0.],
                    ),
                    pitch(-1.0),
                );
            }
            if mob.kind.family() == Family::Witch {
                emit(
                    ([-5., -2., -5.], [5., 0., 5.], [0., 64.], [0., -10., 0.]),
                    glam::Quat::IDENTITY,
                );
                emit(
                    ([-3., -7., -3.], [3., 0., 3.], [0., 70.], [0., -12., 0.]),
                    pitch(0.12),
                );
            }
        }
        Family::Warden | Family::Creaking => {
            let narrow = mob.kind.family() == Family::Creaking;
            let w = if narrow { 3.0 } else { 8.0 };
            emit(
                ([-w, -10., -w], [w, 0., w], [0., 0.], [0., -12., 0.]),
                glam::Quat::IDENTITY,
            );
            emit(
                ([-w, -8., -4.], [w, 12., 4.], [0., 24.], [0., 0., 0.]),
                glam::Quat::IDENTITY,
            );
            for (side, phase) in [(-1., 1.), (1., -1.)] {
                emit(
                    (
                        [-3., -2., -3.],
                        [3., 21., 3.],
                        [40., 0.],
                        [side * (w + 3.), -6., 0.],
                    ),
                    pitch(swing * phase * 0.5),
                );
                emit(
                    (
                        [-3., 0., -3.],
                        [3., 12., 3.],
                        [0., 48.],
                        [side * 4., 12., 0.],
                    ),
                    pitch(swing * phase),
                );
                emit(
                    (
                        [-1., -8., -1.],
                        [1., 0., 1.],
                        [0., 0.],
                        [side * (w + 2.), -17., 0.],
                    ),
                    pitch(0.3),
                );
            }
        }
        Family::Wither => {
            emit(
                ([-3., -14., -3.], [3., 10., 3.], [0., 16.], [0., 2., 0.]),
                pitch(0.2),
            );
            emit(
                ([-12., -3., -3.], [12., 3., 3.], [0., 16.], [0., -4., 0.]),
                glam::Quat::IDENTITY,
            );
            for x in [-11., 0., 11.] {
                emit(
                    ([-4., -8., -4.], [4., 0., 4.], [0., 0.], [x, -8., 0.]),
                    glam::Quat::from_rotation_y((mob.age * 1.3 + x).sin() * 0.2),
                );
            }
            for y in [3., 8., 13.] {
                emit(
                    ([-8., -1., -1.], [8., 1., 1.], [0., 16.], [0., y, 0.]),
                    pitch(0.2),
                );
            }
        }
        Family::Blaze | Family::Breeze => {
            emit(
                ([-4., -8., -4.], [4., 0., 4.], [0., 0.], [0., 4., 0.]),
                glam::Quat::IDENTITY,
            );
            for i in 0..12 {
                let a = mob.age * 1.5 + i as f32 * std::f32::consts::TAU / 4.;
                let y = 7. + (i / 4) as f32 * 6.;
                emit(
                    (
                        [-1., -4., -1.],
                        [1., 4., 1.],
                        [0., 16.],
                        [a.cos() * 8., y, a.sin() * 8.],
                    ),
                    pitch((a * 0.5).sin() * 0.3),
                );
            }
        }
        Family::Ghast => {
            emit(
                ([-8., -8., -8.], [8., 8., 8.], [0., 0.], [0., -10., 0.]),
                glam::Quat::IDENTITY,
            );
            for i in 0..9 {
                emit(
                    (
                        [-1., 0., -1.],
                        [1., 10. + (i % 3) as f32 * 2., 1.],
                        [0., 0.],
                        [(i % 3) as f32 * 6. - 6., -2., (i / 3) as f32 * 6. - 6.],
                    ),
                    pitch((mob.age * 2. + i as f32).sin() * 0.3),
                );
            }
        }
        Family::Phantom | Family::Vex | Family::Bee | Family::Dolphin => {
            let bee = mob.kind.family() == Family::Bee;
            emit(
                ([-4., -3., -8.], [4., 3., 8.], [0., 0.], [0., 16., 0.]),
                glam::Quat::IDENTITY,
            );
            emit(
                ([-3., -3., -5.], [3., 3., 0.], [0., 0.], [0., 15., -8.]),
                glam::Quat::IDENTITY,
            );
            for side in [-1., 1.] {
                emit(
                    (
                        [0., -0.25, -4.],
                        [if bee { 6. } else { 18. }, 0.25, 4.],
                        [0., 0.],
                        [side * 4., 13., 0.],
                    ),
                    glam::Quat::from_rotation_z(
                        side * (mob.age * if bee { 35. } else { 6. }).sin() * 0.7
                            + if side < 0. { std::f32::consts::PI } else { 0. },
                    ),
                );
            }
        }
        Family::Quadruped | Family::Hoglin | Family::Mount | Family::Ravager => {
            let tall = mob.kind.family() == Family::Mount;
            let leg = if tall { 16. } else { 8. };
            let torso_y = 24. - leg - 6.;
            emit(
                ([-5., -6., -9.], [5., 6., 9.], [0., 0.], [0., torso_y, 0.]),
                glam::Quat::IDENTITY,
            );
            emit(
                (
                    [-4., -5., -8.],
                    [4., 5., 0.],
                    [0., 0.],
                    [0., torso_y - if tall { 10. } else { 2. }, -10.],
                ),
                glam::Quat::IDENTITY,
            );
            for (i, p) in [
                [-4., 24. - leg, -6.],
                [4., 24. - leg, -6.],
                [-4., 24. - leg, 6.],
                [4., 24. - leg, 6.],
            ]
            .into_iter()
            .enumerate()
            {
                emit(
                    ([-2., 0., -2.], [2., leg, 2.], [0., 16.], p),
                    pitch(if i % 2 == 0 { swing } else { -swing }),
                );
            }
            if matches!(
                mob.kind,
                Kind::Hoglin | Kind::Zoglin | Kind::Ravager | Kind::Goat
            ) {
                for side in [-1., 1.] {
                    emit(
                        (
                            [-1., -7., -1.],
                            [1., 0., 1.],
                            [0., 0.],
                            [side * 4., torso_y - 5., -13.],
                        ),
                        pitch(0.3),
                    );
                }
            }
        }
        Family::Guardian | Family::Nautilus | Family::Shulker => {
            emit(
                ([-5., -5., -5.], [5., 5., 5.], [0., 0.], [0., 19., 0.]),
                glam::Quat::IDENTITY,
            );
            if mob.kind.family() == Family::Shulker {
                emit(
                    (
                        [-6., -4., -6.],
                        [6., 0., 6.],
                        [0., 0.],
                        [0., 13. + (mob.age * 2.).sin(), 0.],
                    ),
                    glam::Quat::IDENTITY,
                );
            } else {
                for i in 0..8 {
                    let a = i as f32 * std::f32::consts::TAU / 8.;
                    emit(
                        (
                            [-0.7, -6., -0.7],
                            [0.7, 0., 0.7],
                            [0., 0.],
                            [a.cos() * 5., 15., a.sin() * 5.],
                        ),
                        glam::Quat::from_rotation_z(a),
                    );
                }
            }
        }
        Family::Silverfish | Family::Rabbit => {
            for i in 0..5 {
                emit(
                    (
                        [-2., -2., -2.],
                        [2., 2., 2.],
                        [0., 0.],
                        [0., 22., i as f32 * 3. - 6.],
                    ),
                    glam::Quat::from_rotation_y((mob.walk + i as f32).sin() * 0.15),
                );
            }
            if mob.kind.family() == Family::Rabbit {
                for side in [-1., 1.] {
                    emit(
                        (
                            [-1., -8., -0.5],
                            [1., 0., 0.5],
                            [0., 0.],
                            [side * 2., 19., -6.],
                        ),
                        pitch(0.15),
                    );
                }
            }
        }
        Family::SnowGolem => {
            emit(
                ([-5., -5., -5.], [5., 5., 5.], [0., 0.], [0., 19., 0.]),
                glam::Quat::IDENTITY,
            );
            emit(
                ([-3., -4., -3.], [3., 4., 3.], [0., 16.], [0., 10., 0.]),
                glam::Quat::IDENTITY,
            );
            emit(
                ([-4., -8., -4.], [4., 0., 4.], [0., 0.], [0., 6., 0.]),
                glam::Quat::IDENTITY,
            );
            for side in [-1., 1.] {
                emit(
                    (
                        [0., -0.5, -0.5],
                        [10., 0.5, 0.5],
                        [0., 0.],
                        [side * 3., 10., 0.],
                    ),
                    glam::Quat::from_rotation_z(if side < 0. { std::f32::consts::PI } else { 0. }),
                );
            }
        }
    }
}

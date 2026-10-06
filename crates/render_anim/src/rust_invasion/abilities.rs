//! Small block-shaped ability effects in the existing Minecraft entity pass.
use super::*;
use minecraft_terrain::mesh::Vertex;
use sim::invasion::{EffectKind, ProjectileKind};

pub(super) fn white_texel(atlas: &Atlas) -> [f32; 2] {
    let (x, y, _) = atlas
        .pixels
        .enumerate_pixels()
        .filter(|(_, _, p)| p[3] == 255)
        .max_by_key(|(_, _, p)| p[0].min(p[1]).min(p[2]))
        .expect("opaque entity atlas texel");
    [
        (x as f32 + 0.5) / atlas.pixels.width() as f32,
        (y as f32 + 0.5) / atlas.pixels.height() as f32,
    ]
}
fn cube(
    mesh: &mut ChunkMesh,
    center: Vec3,
    half: Vec3,
    rotation: Quat,
    color: [f32; 3],
    uv: [f32; 2],
) {
    let first = mesh.vertices.len() as u32;
    for z in [-1.0, 1.0] {
        for y in [-1.0, 1.0] {
            for x in [-1.0, 1.0] {
                let point = center + rotation * (Vec3::new(x, y, z) * half);
                let block = sim::voxel::to_block([0.0; 3], point.to_array());
                mesh.vertices.push(Vertex {
                    position: block.map(|v| v as f32),
                    uv,
                    color: [color[0], color[1], color[2], 1.0],
                    sky_light: 15.0,
                    block_light: 15.0,
                });
            }
        }
    }
    // Both windings keep the tiny boxes readable across the map-to-block handedness change.
    for [a, b, c, d] in [
        [0, 1, 3, 2],
        [4, 6, 7, 5],
        [0, 4, 5, 1],
        [2, 3, 7, 6],
        [0, 2, 6, 4],
        [1, 5, 7, 3],
    ] {
        mesh.indices.extend([
            first + a,
            first + b,
            first + c,
            first + a,
            first + c,
            first + d,
            first + c,
            first + b,
            first + a,
            first + d,
            first + c,
            first + a,
        ]);
    }
}
fn rod(mesh: &mut ChunkMesh, from: Vec3, to: Vec3, width: f32, color: [f32; 3], uv: [f32; 2]) {
    let delta = to - from;
    let rotation = Quat::from_rotation_arc(Vec3::X, delta.normalize_or_zero());
    cube(
        mesh,
        (from + to) * 0.5,
        Vec3::new(delta.length() * 0.5, width, width),
        rotation,
        color,
        uv,
    );
}
pub(super) fn held_weapon(mesh: &mut ChunkMesh, mob: &Mob, uv: [f32; 2]) {
    if !matches!(
        mob.kind,
        Kind::Skeleton | Kind::Stray | Kind::Bogged | Kind::Parched | Kind::Illusioner
    ) {
        return;
    }
    let forward = Vec3::new(mob.yaw.cos(), mob.yaw.sin(), 0.0);
    let side = Vec3::new(-forward.y, forward.x, 0.0);
    let center = mob.position
        + Vec3::Z * (43.0 * mob.scale)
        + forward * (18.0 * mob.scale)
        + side * (9.0 * mob.scale);
    let upper = center + Vec3::Z * (17.0 * mob.scale);
    let lower = center - Vec3::Z * (17.0 * mob.scale);
    let bend = center + forward * (9.0 * mob.scale);
    rod(mesh, lower, bend, 1.8 * mob.scale, [0.55, 0.28, 0.1], uv);
    rod(mesh, bend, upper, 1.8 * mob.scale, [0.55, 0.28, 0.1], uv);
    let pull = center - forward * (if mob.windup > 0.0 { 8.0 } else { 0.0 } * mob.scale);
    rod(mesh, lower, pull, 0.5 * mob.scale, [0.95, 0.92, 0.8], uv);
    rod(mesh, pull, upper, 0.5 * mob.scale, [0.95, 0.92, 0.8], uv);
}
pub(super) fn append(mesh: &mut ChunkMesh, arena: &Arena, uv: [f32; 2]) {
    for p in &arena.projectiles {
        let direction = p.velocity.normalize_or_zero();
        let rotation = Quat::from_rotation_arc(Vec3::X, direction);
        match p.kind {
            ProjectileKind::Arrow | ProjectileKind::PoisonArrow => {
                rod(
                    mesh,
                    p.position - direction * 13.0,
                    p.position + direction * 10.0,
                    0.75,
                    [0.57, 0.33, 0.13],
                    uv,
                );
                cube(
                    mesh,
                    p.position + direction * 11.0,
                    Vec3::new(3.0, 1.8, 1.8),
                    rotation,
                    [0.75, 0.78, 0.8],
                    uv,
                );
                cube(
                    mesh,
                    p.position - direction * 10.0,
                    Vec3::new(3.0, 3.5, 0.65),
                    rotation,
                    [0.9, 0.9, 0.85],
                    uv,
                );
                cube(
                    mesh,
                    p.position - direction * 10.0,
                    Vec3::new(3.0, 0.65, 3.5),
                    rotation,
                    [0.9, 0.9, 0.85],
                    uv,
                );
            }
            ProjectileKind::PoisonPotion | ProjectileKind::HarmPotion => {
                let spin = Quat::from_rotation_y(arena.clock * 7.0);
                cube(
                    mesh,
                    p.position,
                    Vec3::new(4.0, 4.0, 5.0),
                    spin,
                    if p.kind == ProjectileKind::PoisonPotion {
                        [0.28, 0.75, 0.16]
                    } else {
                        [0.65, 0.08, 0.65]
                    },
                    uv,
                );
                cube(
                    mesh,
                    p.position + spin * Vec3::Z * 7.0,
                    Vec3::new(2.0, 2.0, 2.0),
                    spin,
                    [0.68, 0.42, 0.22],
                    uv,
                );
            }
            _ => {
                let size = match p.kind {
                    ProjectileKind::DragonFireball => 18.0,
                    ProjectileKind::Fireball => 14.0,
                    _ => 6.0,
                };
                let color = if p.kind == ProjectileKind::DragonFireball {
                    [0.72, 0.15, 0.85]
                } else {
                    [1.0, 0.36, 0.04]
                };
                let spin = Quat::from_euler(
                    EulerRot::XYZ,
                    arena.clock * 4.0,
                    arena.clock * 5.0,
                    arena.clock * 2.0,
                );
                cube(mesh, p.position, Vec3::splat(size), spin, color, uv);
                cube(
                    mesh,
                    p.position + direction * size * 0.4,
                    Vec3::splat(size * 0.62),
                    spin,
                    [1.0, 0.86, 0.42],
                    uv,
                );
                for i in 1..5 {
                    cube(
                        mesh,
                        p.position - direction * (i as f32 * size),
                        Vec3::splat(size * (1.0 - i as f32 / 5.0) * 0.65),
                        spin,
                        color,
                        uv,
                    );
                }
            }
        }
    }
    for effect in &arena.effects {
        let age = effect.age;
        let t = (age / 0.9).clamp(0.0, 1.0);
        let count = if effect.kind == EffectKind::Explosion {
            28
        } else {
            14
        };
        for i in 0..count {
            let angle = i as f32 * 2.39996;
            let elevation = ((i * 17 % 23) as f32 / 23.0) * 1.6 - 0.4;
            let direction = Vec3::new(angle.cos(), angle.sin(), elevation).normalize();
            let at =
                effect.position + direction * (effect.radius * t * 0.75) + Vec3::Z * (age * 22.0);
            let color = match effect.kind {
                EffectKind::Explosion if age < 0.18 => [1.0, 0.65, 0.17],
                EffectKind::Explosion => [0.48 + t * 0.2, 0.46 + t * 0.2, 0.43 + t * 0.2],
                EffectKind::Teleport => [0.68, 0.12, 0.85],
                EffectKind::Potion => {
                    if i % 2 == 0 {
                        [0.35, 0.8, 0.2]
                    } else {
                        [0.68, 0.16, 0.8]
                    }
                }
                EffectKind::Spark => [1.0, 0.8, 0.4],
            };
            let size = match effect.kind {
                EffectKind::Explosion => 10.0,
                EffectKind::Spark => 2.0,
                _ => 4.0,
            };
            cube(
                mesh,
                at,
                Vec3::splat(size * (1.0 - t)),
                Quat::from_rotation_z(angle + t),
                color,
                uv,
            );
        }
    }
}

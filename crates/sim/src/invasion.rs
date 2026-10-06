//! Local Rust-arena survival. Movement queries IW4 collision directly; the
//! Minecraft target bridge never substitutes voxel terrain for the map.
use glam::Vec3;
mod behavior;
pub mod streaks;
pub use behavior::{Effect, EffectKind, ProjectileKind, SoundCue};

pub use crate::invasion_roster::{Family, Kind};

#[derive(Clone, Debug)]
pub struct Mob {
    pub id: u64,
    pub kind: Kind,
    pub position: Vec3,
    pub previous: Vec3,
    pub scale: f32,
    pub health: f32,
    pub max_health: f32,
    pub yaw: f32,
    pub age: f32,
    pub walk: f32,
    pub hurt: f32,
    pub fuse: f32,
    pub windup: f32,
    shots: u32,
    teleport_cooldown: f32,
    cooldown: f32,
    velocity: Vec3,
    orbit: f32,
}
impl Mob {
    pub fn new(id: u64, kind: Kind, scale: f32, position: Vec3) -> Self {
        let health = kind.health() * 0.75 * scale.max(0.5).powf(1.3);
        Self {
            id,
            kind,
            position,
            previous: position,
            scale,
            health,
            max_health: health,
            yaw: 0.0,
            age: 0.0,
            walk: 0.0,
            hurt: 0.0,
            fuse: 0.0,
            windup: 0.0,
            shots: 0,
            teleport_cooldown: 0.0,
            cooldown: 1.0 + (id % 7) as f32 * 0.19,
            velocity: Vec3::ZERO,
            orbit: id as f32 * 0.4,
        }
    }
    pub fn bounds(&self) -> [f32; 6] {
        let (r, h) = self.kind.dimensions();
        let (r, h) = (r * self.scale, h * self.scale);
        [
            self.position.x - r,
            self.position.y - r,
            self.position.z,
            self.position.x + r,
            self.position.y + r,
            self.position.z + h,
        ]
    }
    /// Separate dragon body, head and wings: a boss is hittable beyond its
    /// central body without a single giant box hitting empty sky.
    pub fn hitboxes(&self) -> Vec<[f32; 6]> {
        let mut boxes = vec![self.bounds()];
        if self.kind == Kind::Dragon {
            let forward = Vec3::new(self.yaw.cos(), self.yaw.sin(), 0.0);
            let side = Vec3::new(-forward.y, forward.x, 0.0);
            let wing_half = forward.abs() * 35.0 + side.abs() * 120.0 + Vec3::Z * 18.0;
            for (offset, half) in [
                (forward * 170.0 + Vec3::Z * 85.0, Vec3::splat(36.0)),
                (side * 140.0 + Vec3::Z * 65.0, wing_half),
                (-side * 140.0 + Vec3::Z * 65.0, wing_half),
            ] {
                let center = self.position + offset * self.scale;
                let half = half * self.scale;
                boxes.push([
                    center.x - half.x,
                    center.y - half.y,
                    center.z - half.z,
                    center.x + half.x,
                    center.y + half.y,
                    center.z + half.z,
                ]);
            }
        }
        boxes
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sweep {
    pub fraction: f32,
    pub normal: Vec3,
    pub solid: bool,
}
impl Sweep {
    pub fn clear() -> Self {
        Self {
            fraction: 1.0,
            normal: Vec3::ZERO,
            solid: false,
        }
    }
}
/// `radius` and `height` are the creature's swept footprint in map units.
pub trait Collision {
    fn sweep(&self, from: Vec3, to: Vec3, radius: f32, height: f32) -> Sweep;
}
impl<F: Fn(Vec3, Vec3, f32, f32) -> Sweep> Collision for F {
    fn sweep(&self, a: Vec3, b: Vec3, r: f32, h: f32) -> Sweep {
        self(a, b, r, h)
    }
}

#[derive(Clone, Debug)]
pub struct Projectile {
    pub position: Vec3,
    pub velocity: Vec3,
    pub ttl: f32,
    pub kind: ProjectileKind,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Attack {
    pub damage: i32,
    pub from: [f32; 3],
    pub impulse: Vec3,
    pub nonlethal: bool,
}
#[derive(Default)]
pub struct Arena {
    pub streaks: streaks::Streaks,
    hurt_until: f32,
    pub mobs: Vec<Mob>,
    pub projectiles: Vec<Projectile>,
    pub effects: Vec<Effect>,
    sounds: Vec<SoundCue>,
    poison_until: f32,
    next_poison: f32,
    pub wave: u32,
    pub kills: u32,
    pub clock: f32,
    pub intermission: f32,
    spawns: Vec<Vec3>,
    center: Vec3,
    next_id: u64,
    pending: Vec<(Kind, f32, f32)>,
}

impl Arena {
    pub fn new(spawns: Vec<[f32; 3]>) -> Self {
        let spawns: Vec<Vec3> = spawns.into_iter().map(Vec3::from_array).collect();
        let center = if spawns.is_empty() {
            Vec3::ZERO
        } else {
            spawns.iter().copied().sum::<Vec3>() / spawns.len() as f32
        };
        Self {
            spawns,
            center,
            next_id: 1_000_000,
            ..Default::default()
        }
    }
    pub fn damage(&mut self, key: u64, damage: f32) {
        if damage.is_finite() && damage > 0.0 {
            if let Some(mob) = self
                .mobs
                .iter_mut()
                .find(|mob| mob.id == key && mob.health > 0.0)
            {
                mob.health -= damage;
                mob.hurt = 0.22;
                if mob.health <= 0.0 {
                    self.kills += 1;
                    self.streaks.award_kill();
                }
            }
        }
    }
    pub fn explode(&mut self, center: Vec3, collision: &impl Collision) {
        self.blast(center, 250.0, 260.0, 0.0, collision);
    }
    pub fn blast(
        &mut self,
        center: Vec3,
        radius: f32,
        max: f32,
        min: f32,
        collision: &impl Collision,
    ) {
        if !radius.is_finite() || radius <= 0.0 || !max.is_finite() || !min.is_finite() {
            return;
        }
        let hits: Vec<(u64, f32)> = self
            .mobs
            .iter()
            .filter_map(|mob| {
                let at = mob.position + Vec3::Z * mob.kind.dimensions().1 * mob.scale * 0.5;
                let distance = center.distance(at);
                let trace = collision.sweep(center, at, 0.0, 0.0);
                (mob.health > 0.0 && distance < radius && !trace.solid && trace.fraction > 0.99)
                    .then_some((mob.id, max + (min - max) * distance / radius))
            })
            .collect();
        for (id, damage) in hits {
            self.damage(id, damage);
        }
    }
    pub fn boss(&self) -> Option<&Mob> {
        self.mobs
            .iter()
            .find(|m| m.kind == Kind::Dragon && m.health > 0.0)
    }
    pub fn target_population(&self) -> usize {
        Kind::ALL.len() * 2
    }
    pub fn pending_respawns(&self) -> usize {
        self.pending.len()
    }
    fn spawn_wave(&mut self, player: Vec3, collision: &impl Collision) {
        self.wave += 1;
        for &kind in Kind::ALL {
            for copy in 0..2 {
                let scale = kind.base_scale() * if copy == 0 { 0.8 } else { 1.2 };
                self.pending.push((kind, scale, self.clock));
            }
        }
        self.refill(player, collision);
    }
    fn refill(&mut self, player: Vec3, collision: &impl Collision) {
        let mut pending = Vec::new();
        for (kind, scale, ready) in std::mem::take(&mut self.pending) {
            if ready > self.clock {
                pending.push((kind, scale, ready));
                continue;
            }
            let mut candidates: Vec<Vec3> = self
                .spawns
                .iter()
                .copied()
                .filter(|p| p.distance(player) > 250.0)
                .collect();
            candidates.sort_by(|a, b| {
                b.distance_squared(player)
                    .total_cmp(&a.distance_squared(player))
            });
            if candidates.is_empty() {
                candidates.push(self.center + Vec3::new(600.0, 0.0, 0.0));
            }
            let offset = self.next_id as usize % candidates.len();
            let (r, h) = kind.dimensions();
            let mut position = None;
            for i in 0..candidates.len() {
                let at = candidates[(i + offset) % candidates.len()] + Vec3::Z * 5.0;
                let down = collision.sweep(
                    at + Vec3::Z * 100.0,
                    at - Vec3::Z * 400.0,
                    r * scale,
                    h * scale,
                );
                if !down.solid && down.fraction < 1.0 && down.normal.z > 0.5 {
                    position = Some(at + Vec3::Z * (100.0 - 500.0 * down.fraction + 0.5));
                    break;
                }
            }
            if let Some(mut at) = position {
                if kind.flying() {
                    at.z += kind.flight_height();
                }
                self.mobs.push(Mob::new(self.next_id, kind, scale, at));
                self.next_id += 1;
            } else {
                pending.push((kind, scale, self.clock + 0.5));
            }
        }
        self.pending = pending;
    }
    pub fn tick(&mut self, dt: f32, player: Vec3, collision: &impl Collision) -> Vec<Attack> {
        let dt = dt.clamp(0.0, 0.05);
        self.clock += dt;
        let mut living = Vec::new();
        for mob in self.mobs.drain(..) {
            if mob.health > 0.0 {
                living.push(mob);
            } else {
                self.pending.push((mob.kind, mob.scale, self.clock + 3.0));
            }
        }
        self.mobs = living;
        if self.wave == 0 {
            self.spawn_wave(player, collision);
        } else {
            self.refill(player, collision);
        }
        self.wave = 1 + self.kills / self.target_population().max(1) as u32;
        let mut attacks = self.advance_behaviors(dt, player, collision);
        if self.clock < self.hurt_until {
            attacks.clear();
        } else if let Some(hit) = attacks.into_iter().max_by_key(|a| a.damage) {
            self.hurt_until = self.clock + 0.25;
            return vec![hit];
        }
        Vec::new()
    }
}
fn segment_distance(p: Vec3, a: Vec3, b: Vec3) -> f32 {
    let d = b - a;
    let t = if d.length_squared() > 0.0001 {
        ((p - a).dot(d) / d.length_squared()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    p.distance(a + d * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creature_bridge_keeps_original_arena_collision_and_spawns() {
        crate::voxel::activate_mobs([0.0; 3]);
        assert!(crate::voxel::active());
        assert!(!crate::voxel::terrain_active());
        assert!(!crate::voxel::active_for(&[]));
        crate::voxel::deactivate();
        assert!(!crate::voxel::active());
    }
    fn floor(a: Vec3, b: Vec3, _r: f32, _h: f32) -> Sweep {
        if a.z < 0.0 {
            return Sweep {
                fraction: 0.0,
                normal: Vec3::Z,
                solid: true,
            };
        }
        if b.z < 0.0 {
            Sweep {
                fraction: a.z / (a.z - b.z),
                normal: Vec3::Z,
                solid: false,
            }
        } else {
            Sweep::clear()
        }
    }
    #[test]
    fn waves_include_boss_and_varied_sizes_on_real_spawn_positions() {
        let mut arena = Arena::new(vec![
            [700.0, 0.0, 0.0],
            [-700.0, 0.0, 0.0],
            [0.0, 700.0, 0.0],
        ]);
        arena.tick(0.05, Vec3::ZERO, &floor);
        assert!(arena.boss().is_some());
        assert!(arena.mobs.len() > 6);
        assert!(arena.mobs.iter().any(|m| m.scale < 1.0));
        assert!(arena.mobs.iter().any(|m| m.scale >= 2.0));
        for _ in 0..100 {
            arena.tick(0.05, Vec3::ZERO, &floor);
        }
        assert!(
            arena
                .mobs
                .iter()
                .all(|m| m.position.z >= 0.0 && m.position.is_finite())
        );
    }
    #[test]
    fn every_roster_entry_has_two_copies_and_killed_mobs_return() {
        let mut arena = Arena::new(vec![[700.0, 0.0, 0.0], [-700.0, 0.0, 0.0]]);
        arena.tick(0.05, Vec3::ZERO, &floor);
        assert_eq!(arena.mobs.len(), Kind::ALL.len() * 2);
        for &kind in Kind::ALL {
            assert_eq!(arena.mobs.iter().filter(|m| m.kind == kind).count(), 2);
        }
        let dead = arena
            .mobs
            .iter()
            .find(|m| m.kind == Kind::Zombie)
            .unwrap()
            .clone();
        arena.damage(dead.id, 99999.0);
        arena.damage(dead.id, 99999.0);
        assert_eq!(arena.kills, 1);
        arena.tick(0.05, Vec3::ZERO, &floor);
        assert_eq!(arena.pending_respawns(), 1);
        for _ in 0..62 {
            arena.tick(0.05, Vec3::ZERO, &floor);
        }
        assert_eq!(
            arena.mobs.iter().filter(|m| m.kind == Kind::Zombie).count(),
            2
        );
        assert!(arena.mobs.iter().all(|m| m.id != dead.id));
        assert_eq!(arena.mobs.len(), arena.target_population());
    }
    #[test]
    fn both_bosses_respawn_without_waiting_for_the_other_enemies() {
        let mut arena = Arena::new(vec![[700.0, 0.0, 0.0], [-700.0, 0.0, 0.0]]);
        arena.tick(0.05, Vec3::ZERO, &floor);
        let ids: Vec<u64> = arena
            .mobs
            .iter()
            .filter(|m| m.kind.boss())
            .map(|m| m.id)
            .collect();
        for id in ids {
            arena.damage(id, 99999.0);
        }
        for _ in 0..65 {
            arena.tick(0.05, Vec3::ZERO, &floor);
        }
        for kind in [Kind::Dragon, Kind::Wither, Kind::Warden] {
            assert_eq!(arena.mobs.iter().filter(|m| m.kind == kind).count(), 2);
        }
    }
    #[test]
    fn bullets_and_boss_have_scaled_real_hitboxes() {
        let little = Mob::new(1, Kind::Zombie, 0.55, Vec3::ZERO);
        let giant = Mob::new(2, Kind::Zombie, 1.75, Vec3::ZERO);
        assert!(giant.bounds()[5] > little.bounds()[5] * 3.0);
        let mut arena = Arena::default();
        arena.mobs.push(little);
        arena.damage(1, 30.0);
        assert!(arena.mobs[0].health < arena.mobs[0].max_health);
        assert_eq!(
            Mob::new(3, Kind::Dragon, 1.0, Vec3::ZERO).hitboxes().len(),
            4
        );
    }
    #[test]
    fn mobs_and_projectiles_do_not_damage_through_walls() {
        let blocked = |_a: Vec3, _b: Vec3, _r: f32, _h: f32| Sweep {
            fraction: 0.0,
            normal: Vec3::X,
            solid: true,
        };
        let mut arena = Arena::default();
        arena.mobs.push(Mob::new(1, Kind::Zombie, 1.0, Vec3::ZERO));
        arena.mobs[0].cooldown = 0.0;
        assert!(
            arena
                .tick(0.05, Vec3::new(20.0, 0.0, 0.0), &blocked)
                .is_empty()
        );
        assert_eq!(arena.mobs[0].position, Vec3::ZERO);
    }
}

/// Health floor for this local survival mode, without changing ordinary matches.
pub fn player_health(authored: i32) -> i32 {
    if std::env::var("IW4L_RUST_INVASION").as_deref() == Ok("1") {
        authored.max(300)
    } else {
        authored
    }
}

//! Creature abilities for the local, collision-backed Rust arena.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectileKind {
    Arrow,
    PoisonArrow,
    Fireball,
    SmallFireball,
    DragonFireball,
    PoisonPotion,
    HarmPotion,
    Orb,
}
impl ProjectileKind {
    pub fn gravity(self) -> f32 {
        match self {
            Self::Arrow | Self::PoisonArrow => 160.0,
            Self::PoisonPotion | Self::HarmPotion => 500.0,
            _ => 0.0,
        }
    }
    pub fn is_potion(self) -> bool {
        matches!(self, Self::PoisonPotion | Self::HarmPotion)
    }
    pub fn blast_radius(self) -> f32 {
        match self {
            Self::Fireball => 145.0,
            Self::DragonFireball => 180.0,
            Self::SmallFireball => 65.0,
            Self::PoisonPotion | Self::HarmPotion => 110.0,
            _ => 0.0,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectKind {
    Explosion,
    Teleport,
    Potion,
    Spark,
}
#[derive(Clone, Copy, Debug)]
pub struct Effect {
    pub kind: EffectKind,
    pub position: Vec3,
    pub radius: f32,
    pub age: f32,
}
#[derive(Clone, Copy, Debug)]
pub struct SoundCue {
    pub name: &'static str,
    pub position: Vec3,
    pub volume: f32,
}
fn cue(sounds: &mut Vec<SoundCue>, name: &'static str, position: Vec3, volume: f32) {
    sounds.push(SoundCue {
        name,
        position,
        volume,
    });
}
fn visible(collision: &impl Collision, from: Vec3, to: Vec3) -> bool {
    let hit = collision.sweep(from, to, 0.0, 0.0);
    !hit.solid && hit.fraction > 0.99
}
fn projectile_kind(mob: &Mob) -> ProjectileKind {
    match mob.kind {
        Kind::Skeleton
        | Kind::Stray
        | Kind::Parched
        | Kind::Pillager
        | Kind::Piglin
        | Kind::Illusioner => ProjectileKind::Arrow,
        Kind::Bogged => ProjectileKind::PoisonArrow,
        Kind::Ghast => ProjectileKind::Fireball,
        Kind::Blaze => ProjectileKind::SmallFireball,
        Kind::Dragon => ProjectileKind::DragonFireball,
        Kind::Witch if mob.shots % 2 == 0 => ProjectileKind::PoisonPotion,
        Kind::Witch => ProjectileKind::HarmPotion,
        _ => ProjectileKind::Orb,
    }
}
fn shoot(
    mob: &mut Mob,
    target: Vec3,
    projectiles: &mut Vec<Projectile>,
    sounds: &mut Vec<SoundCue>,
) {
    let kind = projectile_kind(mob);
    let from = mob.position + Vec3::Z * (mob.kind.dimensions().1 * mob.scale * 0.72);
    let distance = from.distance(target);
    let speed = match kind {
        ProjectileKind::Arrow | ProjectileKind::PoisonArrow => 720.0,
        ProjectileKind::SmallFireball => 530.0,
        _ if kind.is_potion() => 440.0,
        _ => 360.0,
    };
    let flight = distance / speed;
    let aim = target + Vec3::Z * (0.5 * kind.gravity() * flight * flight);
    projectiles.push(Projectile {
        position: from,
        velocity: (aim - from).normalize_or_zero() * speed,
        ttl: 6.0,
        kind,
    });
    cue(
        sounds,
        match kind {
            ProjectileKind::Arrow | ProjectileKind::PoisonArrow => "entity.skeleton.shoot",
            ProjectileKind::SmallFireball => "entity.blaze.shoot",
            ProjectileKind::Fireball => "entity.ghast.shoot",
            ProjectileKind::DragonFireball => "entity.ender_dragon.shoot",
            _ if kind.is_potion() => "entity.witch.throw",
            _ => "entity.blaze.shoot",
        },
        from,
        if mob.kind.boss() { 2.0 } else { 1.0 },
    );
    mob.shots += 1;
}

impl Arena {
    pub fn take_sounds(&mut self) -> Vec<SoundCue> {
        std::mem::take(&mut self.sounds)
    }
    pub fn clear_player_effects(&mut self) {
        self.streaks.reset_life();
        self.poison_until = 0.0;
        self.next_poison = 0.0;
        self.projectiles.clear();
    }
    pub fn poisoned(&self) -> bool {
        self.poison_until > self.clock
    }

    pub(super) fn advance_behaviors(
        &mut self,
        dt: f32,
        player: Vec3,
        collision: &impl Collision,
    ) -> Vec<Attack> {
        self.effects.iter_mut().for_each(|e| e.age += dt);
        self.effects.retain(|e| e.age < 0.9);
        let mut attacks = Vec::new();
        let mut blasts = Vec::new();
        let target = player + Vec3::Z * 35.0;
        for mob in &mut self.mobs {
            if mob.health <= 0.0 {
                continue;
            }
            mob.previous = mob.position;
            mob.age += dt;
            mob.hurt = (mob.hurt - dt).max(0.0);
            mob.cooldown = (mob.cooldown - dt).max(0.0);
            mob.teleport_cooldown = (mob.teleport_cooldown - dt).max(0.0);
            if mob.kind == Kind::Enderman
                && mob.teleport_cooldown <= 0.0
                && (mob.hurt > 0.0
                    || (mob.position.distance(player) > 280.0
                        && !visible(collision, mob.position + Vec3::Z * 60.0, target)))
            {
                if let Some(to) = teleport_destination(mob, player, collision) {
                    for at in [mob.position, to] {
                        self.effects.push(Effect {
                            kind: EffectKind::Teleport,
                            position: at + Vec3::Z * 35.0,
                            radius: 65.0 * mob.scale,
                            age: 0.0,
                        });
                        cue(&mut self.sounds, "entity.enderman.teleport", at, 1.2);
                    }
                    mob.position = to;
                    mob.previous = to;
                    mob.velocity = Vec3::ZERO;
                }
                mob.teleport_cooldown = 3.0;
            }
            let delta = player - mob.position;
            let distance = Vec3::new(delta.x, delta.y, 0.0).length();
            let direction = Vec3::new(delta.x, delta.y, 0.0).normalize_or_zero();
            let (radius, height) = mob.kind.dimensions();
            let (radius, height) = (radius * mob.scale, height * mob.scale);
            let eye = mob.position + Vec3::Z * (height * 0.72);
            let sees = visible(collision, eye, target);
            mob.yaw = direction.y.atan2(direction.x);

            if mob.kind.explosive() {
                // The wider continuation radius avoids restarting the hiss at the trigger boundary.
                let trigger = if mob.fuse > 0.0 { 160.0 } else { 105.0 } * mob.scale;
                if distance < trigger && delta.z.abs() < height + 20.0 && sees {
                    if mob.fuse == 0.0 {
                        cue(&mut self.sounds, "entity.creeper.primed", mob.position, 1.3);
                    }
                    mob.fuse += dt;
                } else {
                    mob.fuse = (mob.fuse - dt * 2.0).max(0.0);
                }
                if mob.fuse >= 1.5 {
                    let power = if mob.kind == Kind::ChargedCreeper {
                        1.7
                    } else {
                        1.0
                    };
                    blasts.push((
                        mob.position + Vec3::Z * (height * 0.4),
                        210.0 * mob.scale * power,
                        110.0 * power,
                    ));
                    mob.health = 0.0;
                    continue;
                }
            }

            if mob.kind.flying() {
                mob.orbit += dt * 0.32;
                let orbit_radius = if mob.kind.boss() {
                    680.0
                } else if mob.kind == Kind::Ghast {
                    440.0
                } else {
                    180.0
                };
                let anchor = if mob.kind.boss() { self.center } else { player };
                let goal = anchor
                    + Vec3::new(
                        mob.orbit.cos() * orbit_radius,
                        mob.orbit.sin() * orbit_radius,
                        mob.kind.flight_height() + (mob.orbit * 2.0).sin() * 20.0,
                    );
                let next = mob.position + (goal - mob.position).clamp_length_max(270.0 * dt);
                let swept = collision.sweep(mob.position, next, radius, height);
                if !swept.solid {
                    mob.position += (next - mob.position) * swept.fraction.clamp(0.0, 1.0);
                }
                if swept.fraction < 0.99 {
                    let up = mob.position + Vec3::Z * (90.0 * dt);
                    let hit = collision.sweep(mob.position, up, radius, height);
                    if !hit.solid {
                        mob.position += (up - mob.position) * hit.fraction.clamp(0.0, 1.0);
                    }
                }
            } else if mob.fuse <= 0.0 {
                let retreat = mob.kind.ranged() && sees && distance < 220.0;
                let approach = distance
                    > if mob.kind.ranged() && sees {
                        420.0
                    } else {
                        radius + 25.0
                    };
                let motion = if retreat {
                    -direction
                } else if approach {
                    direction
                } else {
                    Vec3::ZERO
                };
                ground_move(mob, motion, dt, collision);
            }
            apply_velocity(mob, dt, collision);
            mob.walk += (mob.position - mob.previous).length() / 24.0;
            if mob.position.z < self.center.z - 1500.0 {
                mob.health = 0.0;
                continue;
            }

            if mob.kind.ranged() && distance < 1300.0 && sees {
                if mob.cooldown <= 0.0 && mob.windup == 0.0 {
                    mob.windup = if mob.kind == Kind::Ghast { 0.8 } else { 0.35 };
                    if matches!(mob.kind, Kind::Ghast | Kind::Blaze) {
                        cue(
                            &mut self.sounds,
                            if mob.kind == Kind::Ghast {
                                "entity.ghast.warn"
                            } else {
                                "entity.blaze.burn"
                            },
                            mob.position,
                            1.0,
                        );
                    }
                }
                if mob.windup > 0.0 {
                    mob.windup -= dt;
                    if mob.windup <= 0.0 {
                        shoot(mob, target, &mut self.projectiles, &mut self.sounds);
                        mob.windup = 0.0;
                        mob.cooldown = match mob.kind {
                            Kind::Blaze if mob.shots % 3 != 0 => 0.25,
                            Kind::Blaze => 4.0,
                            Kind::Witch => 3.5,
                            Kind::Ghast | Kind::Dragon => 3.2,
                            _ => 1.8,
                        };
                    }
                }
            } else {
                mob.windup = 0.0;
                if !mob.kind.explosive()
                    && distance < radius + 40.0
                    && delta.z.abs() < height + 40.0
                    && sees
                    && mob.cooldown <= 0.0
                {
                    attacks.push(Attack {
                        damage: mob.kind.damage(),
                        from: mob.position.to_array(),
                        ..Default::default()
                    });
                    if mob.kind == Kind::CaveSpider {
                        self.poison_until = self.clock + 4.0;
                    }
                    mob.cooldown = 1.1;
                }
            }
        }
        for (center, radius, power) in blasts {
            self.creature_blast(center, radius, power, player, collision, &mut attacks);
        }
        let mut keep = Vec::new();
        for mut projectile in std::mem::take(&mut self.projectiles) {
            projectile.ttl -= dt;
            projectile.velocity.z -= projectile.kind.gravity() * dt;
            let next = projectile.position + projectile.velocity * dt;
            let wall = collision.sweep(projectile.position, next, 2.0, 4.0);
            let end =
                projectile.position + (next - projectile.position) * wall.fraction.clamp(0.0, 1.0);
            let hit = !wall.solid && segment_distance(target, projectile.position, end) < 28.0;
            if !hit && !wall.solid && wall.fraction > 0.99 && projectile.ttl > 0.0 {
                projectile.position = next;
                keep.push(projectile);
                continue;
            }
            if projectile.ttl <= 0.0 {
                continue;
            }
            let point = if hit { target } else { end + wall.normal * 3.0 };
            let kind = projectile.kind;
            let radius = kind.blast_radius();
            if kind.is_potion() {
                self.effects.push(Effect {
                    kind: EffectKind::Potion,
                    position: point,
                    radius,
                    age: 0.0,
                });
                cue(&mut self.sounds, "entity.splash_potion.break", point, 1.0);
                if target.distance(point) < radius && visible(collision, point, target) {
                    if kind == ProjectileKind::PoisonPotion {
                        self.poison_until = self.clock + 6.0;
                    } else {
                        attacks.push(Attack {
                            damage: 18,
                            from: point.to_array(),
                            ..Default::default()
                        });
                    }
                }
            } else if radius > 0.0 {
                self.creature_blast(
                    point,
                    radius,
                    if kind == ProjectileKind::SmallFireball {
                        20.0
                    } else {
                        42.0
                    },
                    player,
                    collision,
                    &mut attacks,
                );
            } else {
                self.effects.push(Effect {
                    kind: EffectKind::Spark,
                    position: point,
                    radius: 12.0,
                    age: 0.0,
                });
                cue(&mut self.sounds, "entity.arrow.hit", point, 0.6);
                if hit {
                    attacks.push(Attack {
                        damage: 12,
                        from: projectile.position.to_array(),
                        ..Default::default()
                    });
                    if kind == ProjectileKind::PoisonArrow {
                        self.poison_until = self.clock + 4.0;
                    }
                }
            }
        }
        self.projectiles = keep;
        if self.poison_until > self.clock && self.next_poison <= self.clock {
            attacks.push(Attack {
                damage: 3,
                from: player.to_array(),
                nonlethal: true,
                ..Default::default()
            });
            self.next_poison = self.clock + 0.8;
        }
        if self.effects.len() > 128 {
            self.effects.drain(..self.effects.len() - 128);
        }
        attacks
    }

    fn creature_blast(
        &mut self,
        center: Vec3,
        radius: f32,
        power: f32,
        player: Vec3,
        collision: &impl Collision,
        attacks: &mut Vec<Attack>,
    ) {
        self.effects.push(Effect {
            kind: EffectKind::Explosion,
            position: center,
            radius,
            age: 0.0,
        });
        cue(&mut self.sounds, "entity.generic.explode", center, 2.0);
        let target = player + Vec3::Z * 35.0;
        let distance = target.distance(center);
        if distance < radius && visible(collision, center, target) {
            let falloff = 1.0 - distance / radius;
            attacks.push(Attack {
                damage: (power * falloff).max(1.0) as i32,
                from: center.to_array(),
                impulse: blast_impulse(target - center, falloff),
                nonlethal: false,
            });
        }
        for mob in self.mobs.iter_mut().filter(|m| m.health > 0.0) {
            let at = mob.position + Vec3::Z * (mob.kind.dimensions().1 * mob.scale * 0.5);
            let d = at.distance(center);
            if d < radius && visible(collision, center, at) {
                let falloff = 1.0 - d / radius;
                mob.health -= power * falloff;
                mob.hurt = 0.22;
                mob.velocity += blast_impulse(at - center, falloff);
                if mob.health <= 0.0 {
                    self.kills += 1;
                }
            }
        }
    }
}
fn blast_impulse(delta: Vec3, strength: f32) -> Vec3 {
    let flat = Vec3::new(delta.x, delta.y, 0.0).normalize_or_zero();
    (flat * 330.0 + Vec3::Z * 220.0) * strength
}
fn teleport_destination(mob: &Mob, player: Vec3, collision: &impl Collision) -> Option<Vec3> {
    let (radius, height) = mob.kind.dimensions();
    for i in 0..12 {
        let angle = mob.orbit + i as f32 * 2.39996;
        let candidate = player + Vec3::new(angle.cos() * 260.0, angle.sin() * 260.0, 0.0);
        let start = candidate + Vec3::Z * 180.0;
        let end = candidate - Vec3::Z * 240.0;
        let down = collision.sweep(start, end, radius * mob.scale, height * mob.scale);
        if down.solid || down.fraction >= 1.0 || down.normal.z < 0.6 {
            continue;
        }
        let at = start.lerp(end, down.fraction) + Vec3::Z * 0.5;
        if at.distance(mob.position) < 100.0 {
            continue;
        }
        let clear = collision.sweep(
            at,
            at + Vec3::Z * 1.0,
            radius * mob.scale,
            height * mob.scale,
        );
        if !clear.solid
            && clear.fraction > 0.99
            && visible(collision, at + Vec3::Z * 50.0, player + Vec3::Z * 35.0)
        {
            return Some(at);
        }
    }
    None
}
fn ground_move(mob: &mut Mob, direction: Vec3, dt: f32, collision: &impl Collision) {
    if direction == Vec3::ZERO {
        return;
    }
    let (radius, height) = mob.kind.dimensions();
    let (radius, height) = (radius * mob.scale, height * mob.scale);
    let reach = collision.sweep(
        mob.position,
        mob.position + direction * 24.0,
        radius,
        height,
    );
    if mob.kind.family() == Family::Spider
        && !reach.solid
        && reach.fraction < 0.99
        && reach.normal.z.abs() < 0.4
    {
        let next = mob.position + Vec3::Z * (95.0 * dt);
        let up = collision.sweep(mob.position, next, radius, height);
        if !up.solid && up.fraction > 0.99 {
            mob.position = next;
            mob.velocity.z = 800.0 * dt;
            return;
        }
    }
    let mut best: Option<(Vec3, f32)> = None;
    for angle in [0.0f32, 0.6, -0.6, 1.2, -1.2, 1.8, -1.8, 2.5, -2.5] {
        let dir = Vec3::new(
            direction.x * angle.cos() - direction.y * angle.sin(),
            direction.x * angle.sin() + direction.y * angle.cos(),
            0.0,
        );
        let trace = collision.sweep(
            mob.position + Vec3::Z,
            mob.position + dir * 65.0 + Vec3::Z,
            radius,
            height,
        );
        let score = trace.fraction * 1.5 + dir.dot(direction);
        if !trace.solid && best.is_none_or(|(_, s)| score > s) {
            best = Some((dir, score));
        }
        if angle == 0.0 && trace.fraction > 0.995 && !trace.solid {
            break;
        }
    }
    if let Some((dir, _)) = best {
        let stride = dir * mob.kind.speed() * dt;
        let trace = collision.sweep(mob.position, mob.position + stride, radius, height);
        if trace.solid {
            return;
        }
        let mut next = mob.position + stride * trace.fraction.clamp(0.0, 1.0);
        if trace.fraction < 0.99 {
            let top = mob.position + Vec3::Z * 22.0;
            let up = collision.sweep(mob.position, top, radius, height);
            let over = collision.sweep(top, top + stride, radius, height);
            if up.fraction > 0.99 && over.fraction > 0.99 && !up.solid && !over.solid {
                next = top + stride;
            }
        }
        let ground = collision.sweep(next + Vec3::Z * 3.0, next - Vec3::Z * 45.0, radius, height);
        if !ground.solid && ground.fraction < 1.0 && ground.normal.z > 0.5 {
            next.z += 3.0 - 48.0 * ground.fraction + 0.5;
            mob.position = next;
        } else if mob.kind.family() == Family::Spider {
            // Advance onto the top once a climb clears the wall.
            mob.position = next;
        }
    }
}
fn apply_velocity(mob: &mut Mob, dt: f32, collision: &impl Collision) {
    if !mob.kind.flying() {
        mob.velocity.z -= 800.0 * dt;
    }
    let next = mob.position + mob.velocity * dt;
    let (radius, height) = mob.kind.dimensions();
    let hit = collision.sweep(mob.position, next, radius * mob.scale, height * mob.scale);
    if hit.solid {
        mob.velocity = Vec3::ZERO;
        return;
    }
    mob.position += (next - mob.position) * hit.fraction.clamp(0.0, 1.0);
    if hit.fraction < 1.0 {
        mob.velocity -= hit.normal * mob.velocity.dot(hit.normal).min(0.0);
        mob.position += hit.normal * 0.25;
    }
    mob.velocity.x *= (1.0 - dt * 4.0).max(0.0);
    mob.velocity.y *= (1.0 - dt * 4.0).max(0.0);
    if mob.kind.flying() {
        mob.velocity.z *= (1.0 - dt * 4.0).max(0.0);
    }
}

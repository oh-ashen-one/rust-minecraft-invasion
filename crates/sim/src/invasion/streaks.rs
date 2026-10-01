//! MW2's complete reward ladder, adapted to the local Minecraft survival arena.
use super::*;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reward {
    Uav,
    CarePackage,
    CounterUav,
    Sentry,
    Predator,
    Precision,
    Harrier,
    Helicopter,
    Emergency,
    PaveLow,
    Stealth,
    ChopperGunner,
    Ac130,
    Emp,
    Nuke,
}
impl Reward {
    pub const ALL: [Self; 15] = [
        Self::Uav,
        Self::CarePackage,
        Self::CounterUav,
        Self::Sentry,
        Self::Predator,
        Self::Precision,
        Self::Harrier,
        Self::Helicopter,
        Self::Emergency,
        Self::PaveLow,
        Self::Stealth,
        Self::ChopperGunner,
        Self::Ac130,
        Self::Emp,
        Self::Nuke,
    ];
    pub fn cost(self) -> u32 {
        match self {
            Self::Uav => 3,
            Self::CarePackage | Self::CounterUav => 4,
            Self::Sentry | Self::Predator => 5,
            Self::Precision => 6,
            Self::Harrier | Self::Helicopter => 7,
            Self::Emergency => 8,
            Self::PaveLow | Self::Stealth => 9,
            Self::ChopperGunner | Self::Ac130 => 11,
            Self::Emp => 15,
            Self::Nuke => 25,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Uav => "UAV",
            Self::CarePackage => "CARE PACKAGE",
            Self::CounterUav => "COUNTER-UAV",
            Self::Sentry => "SENTRY GUN",
            Self::Predator => "PREDATOR MISSILE",
            Self::Precision => "PRECISION AIRSTRIKE",
            Self::Harrier => "HARRIER STRIKE",
            Self::Helicopter => "ATTACK HELICOPTER",
            Self::Emergency => "EMERGENCY AIRDROP",
            Self::PaveLow => "PAVE LOW",
            Self::Stealth => "STEALTH BOMBER",
            Self::ChopperGunner => "CHOPPER GUNNER",
            Self::Ac130 => "AC-130",
            Self::Emp => "EMP",
            Self::Nuke => "TACTICAL NUKE",
        }
    }
    pub fn remote(self) -> bool {
        matches!(self, Self::Predator | Self::ChopperGunner | Self::Ac130)
    }
}
#[derive(Clone, Debug)]
pub struct Support {
    pub reward: Reward,
    pub position: Vec3,
    pub target: Vec3,
    pub age: f32,
    pub duration: f32,
    pub heading: f32,
    pub shots: u32,
    pub mode: u8,
    pub aim: [f32; 3],
    cooldown: f32,
    input_angles: [f32; 3],
}
#[derive(Clone, Debug)]
pub struct Supply {
    pub position: Vec3,
    pub ground: Vec3,
    pub age: f32,
    pub reward: Reward,
}
#[derive(Clone, Debug)]
pub struct Tracer {
    pub from: Vec3,
    pub to: Vec3,
    pub age: f32,
}
#[derive(Default)]
pub struct Streaks {
    pub count: u32,
    pub queue: VecDeque<Reward>,
    pub active: Vec<Support>,
    pub supplies: Vec<Supply>,
    pub tracers: Vec<Tracer>,
    pub banner: String,
    pub banner_until: f32,
    pub radar_until: f32,
    pub jam_until: f32,
    pub emp_until: f32,
    pub nuke_flash_until: f32,
    awarded: u16,
    serial: usize,
    pub fire: bool,
    pub alt: bool,
    last_alt: bool,
    pub view_angles: [f32; 3],
    pub heal: bool,
}
impl Streaks {
    pub fn award_kill(&mut self, clock: f32) {
        self.count += 1;
        for (i, reward) in Reward::ALL.into_iter().enumerate() {
            if self.count >= reward.cost() && self.awarded & (1 << i) == 0 {
                self.awarded |= 1 << i;
                self.queue.push_back(reward);
                self.banner = format!("{} EARNED — D-pad Right / 4", reward.name());
                self.banner_until = clock + 4.0;
            }
        }
    }
    pub fn reset_life(&mut self) {
        self.count = 0;
        self.awarded = 0;
        self.active.retain(|s| !s.reward.remote());
        self.fire = false;
    }
    pub fn remote(&self) -> Option<&Support> {
        self.active.iter().find(|s| s.reward.remote())
    }
    pub fn next_cost(&self) -> Option<u32> {
        Reward::ALL
            .into_iter()
            .find(|r| r.cost() > self.count)
            .map(Reward::cost)
    }
}
fn cue(sounds: &mut Vec<SoundCue>, name: &'static str, position: Vec3, volume: f32) {
    sounds.push(SoundCue {
        name,
        position,
        volume,
    });
}
fn sight(c: &impl Collision, a: Vec3, b: Vec3) -> bool {
    let h = c.sweep(a, b, 0.0, 0.0);
    !h.solid && h.fraction > 0.99
}
fn forward(angles: [f32; 3]) -> Vec3 {
    let (sp, cp) = angles[0].to_radians().sin_cos();
    let (sy, cy) = angles[1].to_radians().sin_cos();
    Vec3::new(cp * cy, cp * sy, -sp)
}
fn angle_delta(a: f32, b: f32) -> f32 {
    (a - b + 180.0).rem_euclid(360.0) - 180.0
}
fn supported(at: Vec3, collision: &impl Collision) -> Option<Vec3> {
    let start = at + Vec3::Z * 160.0;
    let end = at - Vec3::Z * 500.0;
    let hit = collision.sweep(start, end, 12.0, 40.0);
    (!hit.solid && hit.fraction < 1.0 && hit.normal.z > 0.5)
        .then(|| start.lerp(end, hit.fraction) + Vec3::Z * 0.5)
}
impl Arena {
    pub fn use_streak(&mut self, player: Vec3, angles: [f32; 3], collision: &impl Collision) {
        if self.streaks.remote().is_some() {
            self.streaks.active.retain(|s| !s.reward.remote());
            self.streaks.banner = "RETURNED TO PLAYER".into();
            self.streaks.banner_until = self.clock + 2.0;
            return;
        }
        let Some(reward) = self.streaks.queue.front().copied() else {
            return;
        };
        if reward == Reward::Nuke && self.streaks.active.iter().any(|s| s.reward == Reward::Nuke) {
            return;
        }
        let eye = player + Vec3::Z * 55.0;
        let target = self.aim_point(eye, forward(angles), collision);
        let target = self.center + (target - self.center).clamp_length_max(1500.0);
        let direction = Vec3::new(
            angles[1].to_radians().cos(),
            angles[1].to_radians().sin(),
            0.0,
        );
        let placement = supported(player + direction * 90.0, collision);
        if reward == Reward::Sentry && placement.is_none() {
            self.streaks.banner = "SENTRY NEEDS CLEAR GROUND — MOVE AND TRY AGAIN".into();
            self.streaks.banner_until = self.clock + 3.0;
            return;
        }
        self.streaks.queue.pop_front();
        self.streaks.banner = format!("{} DEPLOYED", reward.name());
        self.streaks.banner_until = self.clock + 4.0;
        cue(&mut self.sounds, "block.note_block.pling", player, 1.0);
        self.streaks.serial += 1;
        let duration = match reward {
            Reward::Sentry => 50.0,
            Reward::Harrier | Reward::Helicopter => 40.0,
            Reward::PaveLow => 60.0,
            Reward::ChopperGunner | Reward::Ac130 => 35.0,
            Reward::Predator => 15.0,
            Reward::Nuke => 10.0,
            _ => 7.0,
        };
        match reward {
            Reward::Uav => self.streaks.radar_until = self.clock + 30.0,
            Reward::CounterUav => self.streaks.jam_until = self.clock + 30.0,
            Reward::Emp => {
                self.streaks.emp_until = self.clock + 20.0;
                self.projectiles.clear();
                self.effects.push(Effect {
                    kind: EffectKind::Teleport,
                    position: player + Vec3::Z * 50.0,
                    radius: 500.0,
                    age: 0.0,
                });
            }
            Reward::CarePackage | Reward::Emergency => {
                let count = if reward == Reward::Emergency { 4 } else { 1 };
                let rewards = [
                    Reward::Uav,
                    Reward::Sentry,
                    Reward::Predator,
                    Reward::Precision,
                    Reward::Helicopter,
                    Reward::Harrier,
                    Reward::PaveLow,
                    Reward::ChopperGunner,
                    Reward::Ac130,
                    Reward::Emp,
                ];
                for i in 0..count {
                    let ahead = player
                        + direction * (100.0 + i as f32 * 38.0)
                        + Vec3::new(-direction.y, direction.x, 0.0) * (i as f32 - 1.5) * 45.0;
                    let ground = supported(ahead, collision).unwrap_or(player);
                    let prize = rewards[(self.streaks.serial * 7 + i * 3) % rewards.len()];
                    self.streaks.supplies.push(Supply {
                        position: ground + Vec3::Z * 700.0,
                        ground,
                        age: 0.0,
                        reward: prize,
                    });
                }
            }
            _ => {
                let position = if reward == Reward::Sentry {
                    placement.unwrap()
                } else if reward.remote() {
                    player
                        + Vec3::new(
                            -300.0,
                            0.0,
                            if reward == Reward::Predator {
                                1800.0
                            } else if reward == Reward::Ac130 {
                                1000.0
                            } else {
                                750.0
                            },
                        )
                } else {
                    target + Vec3::Z * 650.0
                };
                let mut aim = angles;
                aim[0] = 65.0;
                self.streaks.active.push(Support {
                    reward,
                    position,
                    target,
                    age: 0.0,
                    duration,
                    heading: 0.0,
                    shots: 0,
                    mode: 0,
                    aim,
                    cooldown: 0.0,
                    input_angles: angles,
                });
            }
        }
    }
    fn aim_point(&self, from: Vec3, dir: Vec3, collision: &impl Collision) -> Vec3 {
        let end = from + dir * 6000.0;
        let wall = collision.sweep(from, end, 0.0, 0.0);
        let end = from.lerp(end, wall.fraction.clamp(0.0, 1.0));
        let mut best: Option<(f32, Vec3)> = None;
        for mob in self.mobs.iter().filter(|m| m.health > 0.0) {
            for bb in mob.hitboxes() {
                if let Some(t) = segment_box(from, end, bb) {
                    if best.is_none_or(|(old, _)| t < old) {
                        best = Some((t, from.lerp(end, t)));
                    }
                }
            }
        }
        best.map(|(_, p)| p).unwrap_or(end)
    }
    fn support_blast(&mut self, at: Vec3, radius: f32, damage: f32, collision: &impl Collision) {
        self.effects.push(Effect {
            kind: EffectKind::Explosion,
            position: at,
            radius,
            age: 0.0,
        });
        cue(&mut self.sounds, "entity.generic.explode", at, 2.0);
        for mob in self.mobs.iter_mut().filter(|m| m.health > 0.0) {
            let mid = mob.position + Vec3::Z * (mob.kind.dimensions().1 * mob.scale * 0.5);
            let d = mid.distance(at);
            if d < radius && sight(collision, at, mid) {
                mob.health -= damage * (1.0 - d / radius).max(0.2);
                mob.hurt = 0.22;
                if mob.health <= 0.0 {
                    self.kills += 1;
                }
            }
        }
    }
    fn support_bullet(&mut self, from: Vec3, at: Vec3, damage: f32, collision: &impl Collision) {
        let point = self.aim_point(from, (at - from).normalize_or_zero(), collision);
        self.streaks.tracers.push(Tracer {
            from,
            to: point,
            age: 0.0,
        });
        let mut best: Option<(u64, f32)> = None;
        for mob in self.mobs.iter().filter(|m| m.health > 0.0) {
            for bb in mob.hitboxes() {
                if let Some(t) =
                    segment_box(from, point + (point - from).normalize_or_zero() * 2.0, bb)
                {
                    if best.is_none_or(|(_, old)| t < old) {
                        best = Some((mob.id, t));
                    }
                }
            }
        }
        if let Some((id, _)) = best {
            if let Some(mob) = self.mobs.iter_mut().find(|m| m.id == id) {
                mob.health -= damage;
                mob.hurt = 0.22;
                if mob.health <= 0.0 {
                    self.kills += 1;
                }
            }
        }
        self.effects.push(Effect {
            kind: EffectKind::Spark,
            position: point,
            radius: 18.0,
            age: 0.0,
        });
    }
    pub fn advance_streaks(&mut self, dt: f32, player: Vec3, collision: &impl Collision) {
        self.streaks.tracers.iter_mut().for_each(|t| t.age += dt);
        self.streaks.tracers.retain(|t| t.age < 0.12);
        let alt = self.streaks.alt && !self.streaks.last_alt;
        self.streaks.last_alt = self.streaks.alt;
        let mut active = Vec::new();
        for mut support in std::mem::take(&mut self.streaks.active) {
            support.age += dt;
            support.cooldown -= dt;
            if support.reward == Reward::Nuke {
                if support.age >= 10.0 {
                    let mut wiped = 0;
                    for mob in self.mobs.iter_mut().filter(|m| m.health > 0.0) {
                        mob.health = 0.0;
                        wiped += 1;
                    }
                    self.kills += wiped;
                    self.projectiles.clear();
                    self.streaks.nuke_flash_until = self.clock + 1.3;
                    self.streaks.heal = true;
                    self.streaks.count = 0;
                    self.streaks.awarded = 0;
                    self.streaks.banner = format!("TACTICAL NUKE — {wiped} MOBS ELIMINATED");
                    self.streaks.banner_until = self.clock + 6.0;
                    cue(&mut self.sounds, "entity.generic.explode", player, 3.0);
                    continue;
                }
                if support.cooldown <= 0.0 {
                    support.cooldown = 1.0;
                    cue(&mut self.sounds, "block.note_block.bell", player, 1.5);
                }
                active.push(support);
                continue;
            }
            if support.age >= support.duration {
                continue;
            }
            if support.reward.remote() {
                let now = self.streaks.view_angles;
                support.aim[0] = (support.aim[0] + angle_delta(now[0], support.input_angles[0]))
                    .clamp(15.0, 89.0);
                support.aim[1] += angle_delta(now[1], support.input_angles[1]);
                support.input_angles = now;
                if support.reward == Reward::Predator {
                    let next = support.position
                        + forward(support.aim)
                            * dt
                            * if self.streaks.fire { 1500.0 } else { 480.0 };
                    let hit = collision.sweep(support.position, next, 3.0, 6.0);
                    support.position = support.position.lerp(next, hit.fraction.clamp(0.0, 1.0));
                    if hit.solid || hit.fraction < 0.99 {
                        self.support_blast(
                            support.position + hit.normal * 5.0,
                            330.0,
                            330.0,
                            collision,
                        );
                        continue;
                    }
                } else {
                    support.heading += dt * 0.14;
                    support.position = support.target
                        + Vec3::new(
                            support.heading.cos() * 500.0,
                            support.heading.sin() * 500.0,
                            if support.reward == Reward::Ac130 {
                                1050.0
                            } else {
                                720.0
                            },
                        );
                    if support.reward == Reward::Ac130 && alt {
                        support.mode = (support.mode + 1) % 3;
                    }
                    if self.streaks.fire && support.cooldown <= 0.0 {
                        let at = self.aim_point(support.position, forward(support.aim), collision);
                        if support.reward == Reward::ChopperGunner {
                            self.support_bullet(support.position, at, 90.0, collision);
                            support.cooldown = 0.10;
                        } else {
                            let (radius, power, delay) = match support.mode {
                                0 => (80.0, 85.0, 0.16),
                                1 => (170.0, 220.0, 0.8),
                                _ => (320.0, 500.0, 2.5),
                            };
                            self.support_blast(at + Vec3::Z * 5.0, radius, power, collision);
                            support.cooldown = delay;
                            self.streaks.tracers.push(Tracer {
                                from: support.position,
                                to: at,
                                age: 0.0,
                            });
                        }
                        cue(
                            &mut self.sounds,
                            "entity.firework_rocket.blast",
                            support.position,
                            1.0,
                        );
                    }
                }
            } else if matches!(support.reward, Reward::Precision | Reward::Stealth)
                || (support.reward == Reward::Harrier && support.age < 4.0)
            {
                support.position =
                    support.target + Vec3::new((support.age - 3.0) * 600.0, 0.0, 650.0);
                if support.cooldown <= 0.0 {
                    let count = if support.reward == Reward::Stealth {
                        9
                    } else {
                        3
                    };
                    if support.shots < count {
                        let offset = Vec3::new(
                            (support.shots as f32 - (count - 1) as f32 * 0.5) * 140.0,
                            0.0,
                            0.0,
                        );
                        let at = supported(support.target + offset, collision)
                            .unwrap_or(support.target + offset)
                            + Vec3::Z * 25.0;
                        self.support_blast(
                            at,
                            if count == 9 { 220.0 } else { 270.0 },
                            300.0,
                            collision,
                        );
                        support.shots += 1;
                        support.cooldown = 0.55;
                    }
                }
            } else {
                if support.reward != Reward::Sentry {
                    support.heading += dt * 0.25;
                    support.position = self.center
                        + Vec3::new(
                            support.heading.cos() * 560.0,
                            support.heading.sin() * 560.0,
                            650.0,
                        );
                }
                if support.cooldown <= 0.0 {
                    let from = support.position + Vec3::Z * 30.0;
                    let nearest = self
                        .mobs
                        .iter()
                        .filter(|m| m.health > 0.0)
                        .filter_map(|m| {
                            let target =
                                m.position + Vec3::Z * m.kind.dimensions().1 * m.scale * 0.55;
                            let d = from.distance_squared(target);
                            (d < 1600.0f32.powi(2) && sight(collision, from, target))
                                .then_some((d, target))
                        })
                        .min_by(|a, b| a.0.total_cmp(&b.0));
                    if let Some((_, at)) = nearest {
                        self.support_bullet(
                            from,
                            at,
                            if support.reward == Reward::PaveLow {
                                95.0
                            } else {
                                65.0
                            },
                            collision,
                        );
                        if support.reward == Reward::Sentry {
                            support.heading = (at.y - from.y).atan2(at.x - from.x);
                        }
                        support.cooldown = if support.reward == Reward::PaveLow {
                            0.13
                        } else {
                            0.25
                        };
                        support.shots += 1;
                        if support.shots % 4 == 0 {
                            cue(&mut self.sounds, "entity.firework_rocket.blast", from, 0.7);
                        }
                    } else {
                        support.cooldown = 0.25;
                    }
                }
            }
            active.push(support);
        }
        self.streaks.active = active;
        for mut supply in std::mem::take(&mut self.streaks.supplies) {
            supply.age += dt;
            supply.position = supply.position.lerp(supply.ground, (dt * 1.6).min(1.0));
            if supply.position.z - supply.ground.z < 10.0 && player.distance(supply.ground) < 90.0 {
                self.streaks.queue.push_back(supply.reward);
                self.streaks.heal = true;
                self.streaks.banner =
                    format!("SUPPLY COLLECTED — {} + FULL HEALTH", supply.reward.name());
                self.streaks.banner_until = self.clock + 4.0;
                cue(
                    &mut self.sounds,
                    "entity.experience_orb.pickup",
                    player,
                    1.0,
                );
            } else if supply.age < 90.0 {
                self.streaks.supplies.push(supply);
            }
        }
    }
}
fn segment_box(a: Vec3, b: Vec3, bb: [f32; 6]) -> Option<f32> {
    let d = b - a;
    let (mut near, mut far) = (0.0f32, 1.0f32);
    for i in 0..3 {
        if d[i].abs() < 1e-6 {
            if a[i] < bb[i] || a[i] > bb[i + 3] {
                return None;
            }
            continue;
        }
        let (u, v) = ((bb[i] - a[i]) / d[i], (bb[i + 3] - a[i]) / d[i]);
        near = near.max(u.min(v));
        far = far.min(u.max(v));
        if near > far {
            return None;
        }
    }
    Some(near)
}

use super::args::{float, vector};
use super::entities::EntityKind;
use crate::script::{ArrayKey, Namespace, NativeRegistry, Runtime, Value};
use bevy_ecs::prelude::World;
use std::collections::BTreeMap;

pub const HUD_MODULE: &str = "maps/mp/iw4l_invasion";
pub const HUD_SOURCE: &str = r#"main()
{
    for (;;)
    {
        level waittill("connected", player);
        player thread rewardStack();
    }
}

rewardStack()
{
    self endon("disconnect");
    icons = [];
    counts = [];
    for (i = 0; i < 15; i++)
    {
        icons[i] = newClientHudElem(self);
        icons[i].horzAlign = "left";
        icons[i].vertAlign = "bottom";
        icons[i].alignX = "left";
        icons[i].alignY = "bottom";
        icons[i].x = 24;
        icons[i].y = -74 - i * 24;
        icons[i].sort = 10;
        icons[i].hideWhenInMenu = true;
        icons[i].alpha = 0;
        counts[i] = newClientHudElem(self);
        counts[i].horzAlign = "left";
        counts[i].vertAlign = "bottom";
        counts[i].alignX = "left";
        counts[i].alignY = "bottom";
        counts[i].x = 75;
        counts[i].y = -78 - i * 24;
        counts[i].fontScale = 1.2;
        counts[i].sort = 10;
        counts[i].hideWhenInMenu = true;
        counts[i].alpha = 0;
    }
    for (;;)
    {
        names = [];
        totals = [];
        if (isDefined(self.pers["killstreaks"]) && self.sessionstate == "playing")
        {
            foreach (reward in self.pers["killstreaks"])
            {
                if (!isDefined(reward) || !isDefined(reward.streakName))
                    continue;
                name = reward.streakName;
                if (!isDefined(totals[name]))
                {
                    names[names.size] = name;
                    totals[name] = 0;
                }
                totals[name]++;
            }
        }
        for (i = 0; i < 15; i++)
        {
            icons[i].alpha = 0;
            counts[i].alpha = 0;
            if (i >= names.size)
                continue;
            shader = tableLookup("mp/killstreakTable.csv", 1, names[i], 14);
            if (shader == "")
                continue;
            icons[i] setShader(shader, 48, 24);
            icons[i].alpha = 1;
            icons[i].color = (1,1,1);
            if (totals[names[i]] > 1)
            {
                counts[i] setValue(totals[names[i]]);
                counts[i].alpha = 1;
            }
        }
        wait .1;
    }
}
"#;

pub fn adapt_source(module: &str, mut source: String) -> Result<String, String> {
    let functions: &[&str] = match module {
        "maps/mp/killstreaks/_helicopter" => {
            &["getPosNearEnemies", "updateAreaNodes", "heli_targeting"]
        }
        "maps/mp/killstreaks/_harrier" => &["harrierGetTargets"],
        _ => &[],
    };
    for name in functions {
        edit_function(&mut source, name, |body| {
            body.replace("level.players", "iw4l_invasion_targets()")
        })?;
    }
    match module {
        "maps/mp/killstreaks/_airstrike" => {
            edit_function(&mut source, "losRadiusDamage", |body| {
                format!("\n iw4l_invasion_blast(pos, radius, max, min);{body}")
            })?
        }
        "maps/mp/killstreaks/_nuke" => edit_function(&mut source, "nukeDeath", |body| {
            format!("\n iw4l_invasion_nuke();{body}")
        })?,
        _ => {}
    }
    Ok(source)
}
fn edit_function(
    source: &mut String,
    name: &str,
    edit: impl FnOnce(&str) -> String,
) -> Result<(), String> {
    let needle = format!("\n{name}(");
    let start = source
        .find(&needle)
        .ok_or_else(|| format!("Invasion integration needs {name}"))?;
    let body = start + source[start..].find('{').ok_or("Missing function body")? + 1;
    let end = body + source[body..].find("\n}").ok_or("Missing function end")?;
    source.replace_range(body..end, &edit(&source[body..end]));
    Ok(())
}

pub(crate) fn register(registry: &mut NativeRegistry) {
    registry.register(
        Namespace::Function,
        "iw4l_invasion_targets",
        |world, _, _| {
            let mut runtime = world.resource_mut::<Runtime>();
            let id = runtime.next_object;
            runtime.next_object += 1;
            let values = runtime
                .invasion_targets
                .values()
                .enumerate()
                .map(|(i, object)| (ArrayKey::Integer(i as i32), Value::Object(*object)))
                .collect();
            runtime.arrays.insert(id, values);
            Ok(Value::Array(id))
        },
    );
    registry.register(Namespace::Function, "iw4l_invasion_blast", |_, _, args| {
        crate::voxel::push_invasion_blast(
            vector(args, 0)?,
            float(args, 1)?,
            float(args, 2)?,
            float(args, 3)?,
        );
        Ok(Value::Undefined)
    });
    registry.register(Namespace::Function, "iw4l_invasion_nuke", |_, _, _| {
        for (key, _, _) in crate::voxel::mob_targets() {
            crate::voxel::push_mob_shot(key, 1_000_000.0, [0.0; 3]);
        }
        Ok(Value::Undefined)
    });
}

pub(crate) fn is_target(world: &World, receiver: &Value) -> bool {
    matches!(receiver, Value::Object(object) if world.resource::<Runtime>().invasion_targets.values().any(|id| id == object))
}

pub(crate) fn advance(world: &mut World) {
    if !crate::voxel::active() || crate::voxel::terrain_active() {
        return;
    }
    let mut bounds: BTreeMap<u64, ([f32; 3], [f32; 3])> = BTreeMap::new();
    for (key, min, max) in crate::voxel::mob_targets() {
        bounds
            .entry(key)
            .and_modify(|(a, b)| {
                for i in 0..3 {
                    a[i] = a[i].min(min[i]);
                    b[i] = b[i].max(max[i]);
                }
            })
            .or_insert((min, max));
    }
    let removed: Vec<_> = world
        .resource::<Runtime>()
        .invasion_targets
        .iter()
        .filter(|(key, _)| !bounds.contains_key(key))
        .map(|(key, id)| (*key, *id))
        .collect();
    for (key, id) in removed {
        super::super::runtime::raise(world, Value::Object(id), "death", Vec::new());
        let mut runtime = world.resource_mut::<Runtime>();
        runtime.invasion_targets.remove(&key);
        runtime.delete_entity(id);
    }
    let mut runtime = world.resource_mut::<Runtime>();
    for (key, (min, max)) in bounds {
        let object = match runtime.invasion_targets.get(&key).copied() {
            Some(id) => id,
            None => {
                let Ok(id) = runtime.create_entity(EntityKind::Spawned, "invasion_target") else {
                    continue;
                };
                let pers = runtime.next_object;
                runtime.next_object += 1;
                runtime.arrays.insert(
                    pers,
                    BTreeMap::from([(ArrayKey::String("team".into()), Value::string("none"))]),
                );
                runtime.set_object_field(id, "pers", Value::Array(pers));
                for field in ["team", "sessionteam"] {
                    runtime.set_object_field(id, field, Value::string("none"));
                }
                runtime.set_object_field(id, "sessionstate", Value::string("playing"));
                runtime.set_object_field(id, "health", Value::Int(100));
                runtime.set_object_field(id, "score", Value::Int(0));
                runtime.set_object_field(id, "angles", Value::Vector([0.0; 3]));
                runtime.invasion_targets.insert(key, id);
                id
            }
        };
        let origin = [
            (min[0] + max[0]) * 0.5,
            (min[1] + max[1]) * 0.5,
            (min[2] + max[2]) * 0.5 - 40.0,
        ];
        runtime.set_object_field(object, "origin", Value::Vector(origin));
        if let Some(entity) = runtime.entities.get_mut(&object) {
            entity.cylinder = Some(((max[0] - min[0]).max(max[1] - min[1]) * 0.5, 80.0));
        }
    }
    drop(runtime);
    for (client, name) in crate::voxel::take_invasion_rewards() {
        super::players::give_killstreak(world, client, name);
    }
}

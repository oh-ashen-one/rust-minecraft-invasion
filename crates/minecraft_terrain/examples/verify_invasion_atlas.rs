fn main() {
    use minecraft_terrain::{
        pack::{PackStack, ResourceId},
        scene::HandcraftedScene,
    };
    let root = std::env::args_os().nth(1).expect("resource-pack path");
    let packs = PackStack::open(vec![root.into()]).expect("Mojang resource pack");
    let built =
        minecraft_terrain::mesh::build(&HandcraftedScene::default(), &packs).expect("entity atlas");
    let roster: serde_json::Value =
        serde_json::from_str(include_str!("../../../docs/INVASION-ROSTER.json")).unwrap();
    for entry in roster["enemies"].as_array().unwrap() {
        let skin = entry["skin"].as_str().unwrap();
        let id = ResourceId::parse(&format!("minecraft:entity/{skin}")).unwrap();
        assert!(
            packs.texture(&id).unwrap().is_some(),
            "missing Mojang skin: {skin}"
        );
        assert!(built.atlas.contains(&id), "missing atlas skin: {skin}");
        let uv = built.atlas.entity_region(&id);
        assert!(uv.iter().all(|v| v.is_finite()) && uv[2] > uv[0] && uv[3] > uv[1]);
        println!("PASS: {skin} in entity atlas");
    }
}

fn main() {
    assets::minecraft_map::prepare();
    match assets::minecraft_setup::wait(std::time::Duration::from_secs(600)) {
        Some(root) => println!("Minecraft assets ready: {}", root.display()),
        None => {
            eprintln!("{}", assets::minecraft_setup::status());
            std::process::exit(1);
        }
    }
}

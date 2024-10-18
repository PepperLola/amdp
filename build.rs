use std::env;

fn main() {
    dotenv::dotenv().ok();

    if let Ok(discord_id) = env::var("DISCORD_ID") {
        println!("cargo:rustc-env=DISCORD_ID={}", discord_id);
    }
}

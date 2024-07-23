use rocket::{fairing::AdHoc, launch, routes, Build, Rocket, tokio::sync::Mutex};
use rustdirect::{storage, handlers, RedirectConfig};

#[launch]
fn entry() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![handlers::redirect, handlers::create_new])
        .attach(AdHoc::try_on_ignite("Storage", |rocket| async move {
            let storage: String = rocket.figment().extract_inner("store").expect("store");
            let store = if storage == ":memory:" {
                println!("Setting memory store");
                RedirectConfig::new(storage::MemoryStore::new())
            } else {
                println!("Load mapping from {storage}");
                let file = storage::SimpleFile::new(&storage).expect("storage to be ok");
                RedirectConfig::new(file)
            };
            Ok(rocket.manage(Mutex::new(store)))
        }))
}

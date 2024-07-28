use rocket::{fairing::AdHoc, fs::FileServer, launch, routes, tokio::sync::Mutex, Build, Rocket};
use rustdirect::{storage, handlers, RedirectConfig};

#[launch]
fn entry() -> Rocket<Build> {
    let routes = routes![handlers::index, handlers::redirect, handlers::create_new];
    #[cfg(feature = "noui")]
    routes.extend(routes![handlers::create_new_form]);
    let rocket = rocket::build()
        .mount("/", routes)
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
        }));
    #[cfg(not(feature = "noui"))]
    let rocket = rocket.mount("/_internal/ui", FileServer::from("ui/dist"));
    rocket
}

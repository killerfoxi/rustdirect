use rocket::fairing::AdHoc;
use rocket::http::uri::{Absolute, Reference};
use rocket::outcome::{try_outcome, IntoOutcome};
use rocket::request::FromRequest;
use rocket::response::Redirect;
use rocket::{get, http::Status, launch, Build, Rocket};
use rocket::{routes, State};
use std::path::PathBuf;
use std::sync::Mutex;

trait UrlMap {
    fn lookup(&self, key: &str) -> Option<&Absolute<'static>>;
    fn maybe_insert(&mut self, key: &str, value: Absolute<'static>) -> bool;
}

mod storage {
    use super::UrlMap;
    use rocket::http::uri::Absolute;
    use std::collections::HashMap;
    use std::fs::OpenOptions;
    use std::io::{BufRead, Write};
    use std::iter::FromIterator;

    pub struct MemoryStore(HashMap<Box<str>, Absolute<'static>>);

    impl MemoryStore {
        pub fn new() -> Self {
            Self(HashMap::new())
        }
    }

    impl FromIterator<(Box<str>, Absolute<'static>)> for MemoryStore {
        fn from_iter<T>(iter: T) -> Self
        where
            T: IntoIterator<Item = (Box<str>, Absolute<'static>)>,
        {
            Self(iter.into_iter().collect())
        }
    }

    impl UrlMap for MemoryStore {
        fn lookup(&self, key: &str) -> Option<&Absolute<'static>> {
            self.0.get(key)
        }

        fn maybe_insert(&mut self, key: &str, value: Absolute<'static>) -> bool {
            if self.0.contains_key(key) {
                false
            } else {
                self.0.insert(key.into(), value);
                true
            }
        }
    }

    pub struct SimpleFile {
        file: std::fs::File,
        cache: MemoryStore,
    }

    impl SimpleFile {
        pub fn new(path: &str) -> Result<Self, std::io::Error> {
            let f = OpenOptions::new()
                .read(true)
                .append(true)
                .create(true)
                .open(path)?;
            let cache = std::io::BufReader::new(&f)
                .lines()
                .map(|line| -> Result<_, std::io::Error> {
                    let binding = line?;
                    let (name, url) = binding
                        .split_once('\0')
                        .ok_or(std::io::Error::other("invalid delimiter"))?;
                    Ok((
                        name.to_owned().into_boxed_str(),
                        Absolute::parse_owned(url.to_owned()).map_err(|err| {
                            std::io::Error::other(format!("unable to parse {url}: {err}"))
                        })?,
                    ))
                })
                .collect::<Result<MemoryStore, _>>()?;
            Ok(Self { file: f, cache })
        }
    }

    impl UrlMap for SimpleFile {
        fn lookup(&self, key: &str) -> Option<&Absolute<'static>> {
            self.cache.lookup(key)
        }

        fn maybe_insert(&mut self, key: &str, value: Absolute<'static>) -> bool {
            let line = format!("{key}\0{}\n", value);
            if self.cache.maybe_insert(key, value) {
                let _ = self.file.write_all(line.as_bytes());
                return true;
            }
            return false;
        }
    }
}

struct RedirectConfig(Box<dyn UrlMap + 'static + Send>);

impl RedirectConfig {
    fn new<S: UrlMap + 'static + Send>(store: S) -> Self {
        Self(Box::new(store))
    }

    fn add(&mut self, name: &str, url: Absolute<'static>) -> bool {
        self.0.maybe_insert(name, url)
    }

    fn lookup(&self, name: &str) -> Option<&Absolute<'static>> {
        self.0.lookup(name)
    }
}

struct RedirectToken {
    url: Absolute<'static>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RedirectToken {
    type Error = ();

    async fn from_request(
        request: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let token: &str = try_outcome!(request.param(0).or_forward(Status::NotFound)).unwrap();
        let config = try_outcome!(request
            .rocket()
            .state::<Mutex<RedirectConfig>>()
            .or_forward(Status::InternalServerError))
        .lock()
        .expect("lock");
        config
            .lookup(token)
            .map(|url| RedirectToken { url: url.clone() })
            .or_forward(Status::NotFound)
    }
}

#[get("/<_>/<additional..>")]
fn redirect(token: RedirectToken, additional: PathBuf) -> Redirect {
    let path = additional.as_path().to_str().unwrap_or("");
    let to: Reference<'static> = Reference::parse_owned(format!("{}/{}", token.url, path)).unwrap();
    Redirect::to(to.into_normalized())
}

#[get("/_internal/new/<name>?<to>")]
fn create_new(
    redirects: &State<Mutex<RedirectConfig>>,
    name: &str,
    to: String,
) -> (Status, String) {
    match Absolute::parse_owned(to) {
        Ok(url) => {
            let outcome = redirects.lock().expect("lock").add(name, url);
            if outcome {
                (Status::Created, format!("{name} has been created"))
            } else {
                (Status::Conflict, format!("{name} already exists"))
            }
        }
        Err(err) => (
            Status::BadGateway,
            format!("provided url isn't valid: {err}"),
        ),
    }
}

#[launch]
fn entry() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![redirect, create_new])
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

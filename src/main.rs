use rocket::http::uri::{Absolute, Reference};
use rocket::outcome::{try_outcome, IntoOutcome};
use rocket::request::FromRequest;
use rocket::response::{status, Redirect};
use rocket::{get, http::Status, launch, Build, Rocket};
use rocket::{routes, uri, State};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

struct RedirectConfig(HashMap<Box<str>, Absolute<'static>>);

impl RedirectConfig {
    fn new() -> Self {
        Self(HashMap::from([(
            "goog".into(),
            uri!("https://google.ch"),
        )]))
    }

    fn add(&mut self, name: &str, url: Absolute<'static>) -> bool {
        if self.0.contains_key(name) {
            return false;
        }
        self.0.insert(name.to_owned().into_boxed_str(), url);
        return true;
    }

    fn lookup(&self, name: &str) -> Option<&Absolute<'static>> {
        self.0.get(name)
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
        let config = try_outcome!(request.rocket().state::<Mutex<RedirectConfig>>().or_forward(Status::InternalServerError)).lock().expect("lock");
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
fn create_new(redirects: &State<Mutex<RedirectConfig>>, name: &str, to: String) -> (Status, String) {
    match Absolute::parse_owned(to) {
    Ok(url) => {
        let outcome = redirects.lock().expect("lock").add(name, url);
        if outcome {
            (Status::Created, format!("{name} has been created"))
        } else {
            (Status::Conflict, format!("{name} already exists"))
        }
    },
    Err(err) => (Status::BadGateway, format!("provided url isn't valid: {err}")),
    }
}

#[launch]
fn entry() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![redirect, create_new])
        .manage(Mutex::new(RedirectConfig::new()))
}

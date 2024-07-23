use rocket::{get, http::{uri::{Absolute, Reference}, Status}, response::Redirect, tokio::sync::Mutex, State};

use std::path::PathBuf;
use crate::{RedirectConfig, RedirectToken};

#[get("/<_>/<additional..>")]
pub fn redirect(token: RedirectToken, additional: PathBuf) -> Redirect {
    let path = additional.as_path().to_str().unwrap_or("");
    let to: Reference<'static> = Reference::parse_owned(format!("{}/{}", token.url, path)).unwrap();
    Redirect::to(to.into_normalized())
}

#[get("/_internal/new/<name>?<to>")]
pub async fn create_new(
    redirects: &State<Mutex<RedirectConfig>>,
    name: &str,
    to: String,
) -> (Status, String) {
    match Absolute::parse_owned(to) {
        Ok(url) => {
            let outcome = redirects.lock().await.add(name, url);
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
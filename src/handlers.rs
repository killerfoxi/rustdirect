use maud::{html, Markup};
use rocket::{
    get,
    http::{
        uri::{Absolute, Reference},
        Status,
    },
    response::Redirect,
    tokio::sync::Mutex,
    State,
};

use crate::{RedirectConfig, RedirectToken};
use std::path::PathBuf;

mod rendering {
    use maud::{html, Markup};
    use rocket::uri;
    use std::path::PathBuf;

    pub fn redirect_link(name: &str) -> Markup {
        let link = uri!(super::redirect(name, PathBuf::new())).to_string();
        html! {
            a href=(link) { (name) }
        }
    }

    pub fn bad_gateway(info: Markup) -> Markup {
        html! {
            h1 { "Whoopsie the gateway has gone bad"}
            (info)
        }
    }

    pub fn conflict(resource: &str) -> Markup {
        html! {
            h1 { "I am conflicted" }
            p { "The resource " (redirect_link(resource)) " already exists. Oh no!"  }
        }
    }

    pub fn created(resource: &str) -> Markup {
        html! {
            h1 { "I will now point there" }
            p {
                (redirect_link(resource))
                " now points to something."
            }
        }
    }
}

#[cfg(not(feature = "noui"))]
#[get("/")]
pub fn index() -> Redirect {
    Redirect::to("/_internal/ui/")
}

#[get("/favicon.ico")]
pub fn favicon() -> Result<(), ()> {
    Err(())
}

#[cfg(feature = "noui")]
#[get("/")]
pub fn index() -> Redirect {
    Redirect::to(uri!(create_new_form("")))
}

#[get("/<_>/<additional..>")]
pub fn redirect(token: RedirectToken, additional: PathBuf) -> Redirect {
    let path = additional.as_path().to_str().unwrap_or("");
    let to: Reference<'static> = Reference::parse_owned(format!("{}/{}", token.url, path)).unwrap();
    Redirect::to(to.into_normalized())
}

#[cfg(feature = "noui")]
#[get("/_internal/new/<name..>")]
pub async fn create_new_form(name: PathBuf) -> Markup {
    use std::fs::Component;
    let name = name
        .components()
        .filter(|c| matches!(c, Component::Normal(_)))
        .take(1)
        .next()
        .and_then(|p| p.as_os_str().to_str().map(|str| str.to_owned()))
        .unwrap_or(String::new());
    html! {
        h1 { "Create new redirect" }
        form {
            label for="name" { "Name of the link" }
            input type="text" name="name" value=(name) required;
            label for="url" { "Redirect to here" }
            input type="url" name="to" pattern="https?://.+" required;
            input type="submit" value="Create" formaction="/_internal/new";
        }
    }
}

#[get("/_internal/new?<name>&<to>")]
pub async fn create_new(
    redirects: &State<Mutex<RedirectConfig>>,
    name: &str,
    to: String,
) -> (Status, Markup) {
    match Absolute::parse_owned(to) {
        Ok(url) => {
            let outcome = redirects.lock().await.add(name, url);
            if outcome {
                (Status::Created, rendering::created(name))
            } else {
                (Status::Conflict, rendering::conflict(name))
            }
        }
        Err(err) => (
            Status::BadGateway,
            rendering::bad_gateway(html! {
                p { "The provided url isn't valid." }
                p { (err) }
            })
        ),
    }
}

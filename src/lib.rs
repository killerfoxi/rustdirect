use rocket::{http::{uri::Absolute, Status}, outcome::{try_outcome, IntoOutcome}, request::FromRequest, tokio::sync::Mutex};

pub mod storage;
pub mod handlers;

pub trait UrlMap {
    fn lookup(&self, key: &str) -> Option<&Absolute<'static>>;
    fn maybe_insert(&mut self, key: &str, value: Absolute<'static>) -> bool;
}

pub struct RedirectConfig(Box<dyn UrlMap + 'static + Send>);

impl RedirectConfig {
    pub fn new<S: UrlMap + 'static + Send>(store: S) -> Self {
        Self(Box::new(store))
    }

    pub fn add(&mut self, name: &str, url: Absolute<'static>) -> bool {
        self.0.maybe_insert(name, url)
    }

    pub fn lookup(&self, name: &str) -> Option<&Absolute<'static>> {
        self.0.lookup(name)
    }
}

pub struct RedirectToken {
    pub url: Absolute<'static>,
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
        .await;
        config
            .lookup(token)
            .map(|url| RedirectToken { url: url.clone() })
            .or_forward(Status::NotFound)
    }
}
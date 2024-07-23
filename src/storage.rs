use super::UrlMap;
use rocket::http::uri::Absolute;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufRead, Write};
use std::iter::FromIterator;

#[derive(Default)]
pub struct MemoryStore(HashMap<Box<str>, Absolute<'static>>);

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
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

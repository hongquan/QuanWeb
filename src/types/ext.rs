// Traits to extend the types from the external crates

use std::str::FromStr;

use http::uri::{Uri, PathAndQuery, Builder};

pub trait UriExt {
    fn join(&self, path: &str) -> Uri;
}

impl UriExt for Uri {
    fn join(&self, to_join: &str) -> Uri {
        let path_alone = self.path();
        let new_path = format!("{}{}", path_alone, to_join).replace("//", "/");
        let mut builder = Builder::new();
        if let Some(scheme) = self.scheme() {
            builder = builder.scheme(scheme.as_str());
        }
        if let Some(authority) = self.authority() {
            builder = builder.authority(authority.as_str());
        }
        if let Ok(p_and_q) = PathAndQuery::from_str(new_path.as_str()) {
            builder = builder.path_and_query(p_and_q);
        }
        builder.build().unwrap_or(self.clone())
    }
}

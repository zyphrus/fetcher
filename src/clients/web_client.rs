//! A Web client to Fetcher Web UI
//!
//!
use std::io::Read;
use url::{Url, ParseError};

use hyper;
use hyper::client::response::Response;
use hyper::header::ContentType;
use hyper::mime::Mime;
use hyper::header::Connection;
use hyper::header::ConnectionOption;
use hyper::error::HttpResult;

use rustc_serialize::json;
use rustc_serialize::json::Json;

use clients::{Client, Series, Provider, BaseProvider};

///
///
///
pub struct WebClient {
    url: Url,
    client: hyper::Client,
}

impl WebClient {
    ///
    /// #Example
    ///
    /// ```
    /// use fetcher::clients::WebClient;
    ///
    /// let client = WebClient::new("http://127.0.0.1/").unwrap();
    /// ```
    pub fn new(url: &str) -> Result<WebClient, ParseError> {
        let url_parsed = match Url::parse(url) {
            Ok(u) => u,
            Err(e) => return Err(e),
        };

        Ok(WebClient {
            url: url_parsed,
            client: hyper::Client::new(),
        })
    }

    ///
    ///
    ///
    fn build_url(&self, to_add: &str) -> Result<Url, ParseError> {
        let base_url = &self.url.to_string();
        let new_url: String;
        if base_url.ends_with('/') ^ to_add.starts_with('/') {
            // One trailing or starting '/'
            new_url = format!("{}{}", base_url, to_add).to_string();
        } else if base_url.ends_with('/') && to_add.starts_with('/') {
            // Pair of trailing and starting '/'
            new_url = format!("{}{}", base_url.trim_right_matches('/'), to_add).to_string();
        } else {
            // No trailing or starting '/'
            new_url = format!("{}/{}", base_url, to_add).to_string();
        }

        Url::parse(&new_url)
    }

    /// Raw access to using the HTTP `GET` method at a given `path`
    /// from the base url given at construction.
    ///
    /// #Example
    ///
    /// ```
    /// use fetcher::clients::Client;
    /// use fetcher::clients::WebClient;
    ///
    /// let mut client = WebClient::new("http://127.0.0.1/").unwrap();
    ///
    /// client.get("/api/");
    /// ```
    pub fn get(&mut self, path: &str) -> HttpResult<Response> {
        // FIXME: build_url cannot be asserted to be valid
        let full_url = self.build_url(path).unwrap();
        self.client.get(full_url)
            .header(Connection(vec![ConnectionOption::Close]))
            .header(ContentType("application/json".parse::<Mime>().unwrap()))
            .send()
    }
}

///
///
///
impl Client for WebClient {


    /// Get a list of Series
    ///
    /// # Example
    ///
    /// ```
    /// use fetcher::clients::Client;
    /// use fetcher::clients::WebClient;
    ///
    /// let mut client = WebClient::new("http://127.0.0.1/").unwrap();
    ///
    /// client.get_series();
    /// ```
    fn get_series(&mut self) -> Result<Vec<Series>, String> {
        let mut res = match self.get("/series?format=fetch") {
            Ok(r) => r,
            Err(e) => return Err(format!("Error during GET: {}", e)),
        };

        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();

        let mut series = vec![];

        match Json::from_str(&body) {
            Ok(series_json) => match series_json {
                Json::Array(arr) => {
                    for obj in arr {
                        let s = match json::decode::<Series>(&format!("{}", obj)) { // FIXME: There has to be a better way
                            Ok(val) => val,
                            Err(e)  => return Err(format!("JSON decode error: {}", e)),
                        };
                        series.push(s);
                    }
                },
                other => return Err(format!("Expected Array but got {:?}", other)),
            },
            Err(e) => return Err(format!("JSON parse error: {}", e)),
        };
        Ok(series)
    }

    ///
    ///
    ///
    fn get_providers(&mut self) -> Result<Vec<Provider>, ()> {
        unimplemented!();
    }

    ///
    ///
    ///
    fn get_base_providers(&mut self) -> Result<Vec<BaseProvider>, ()> {
        unimplemented!();
    }

}

#[cfg(test)]
mod test{
    extern crate url;

    use url::Url;
    use clients::Client;
    use super::*;


    #[test]
    fn test_url_build() {
        for (url, path, built) in vec![
                ("http://e.co", "/p/t/f.html", "http://e.co/p/t/f.html"),
                ("http://e.co/", "/p/t/f.html", "http://e.co/p/t/f.html"),
                ("http://e.co", "p/t/f.html", "http://e.co/p/t/f.html"),
                ("http://e.co:8080", "p/t/f.html", "http://e.co:8080/p/t/f.html"),
                ("http://e.co", "p/t/f.html?var=1", "http://e.co/p/t/f.html?var=1"),
            ] {
            let c = WebClient::new(url).unwrap();
            let built_url = c.build_url(path);
            assert!(Url::parse(built).unwrap() == built_url.unwrap(),
                    "assertion failed: url:{} path:{} built:{} != expected:{}", url, path, built, Url::parse(built).unwrap());
        }
    }

    #[test]
    fn test_get_series() {
        let mut client = WebClient::new("http://127.0.0.1:8000/").unwrap();
        // TODO: Check if a valid server exists, if not skip test
        client.get_series();
    }
}

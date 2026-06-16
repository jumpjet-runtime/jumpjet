use reqwest::{Method, StatusCode};
use web_transport::{Client, Server, Session};

use crate::network::HttpMethod;

pub type NetworkClient = Client;
pub type NetworkServer = Server;
pub type NetworkHttpClient = reqwest::Client;
pub type NetworkConnection = Session;

impl Into<Method> for HttpMethod {
    fn into(self) -> Method {
        match self {
            HttpMethod::Delete => Method::DELETE,
            HttpMethod::Head => Method::HEAD,
            HttpMethod::Get => Method::GET,
            HttpMethod::Post => Method::POST,
            HttpMethod::Put => Method::PUT
        }
    }
}

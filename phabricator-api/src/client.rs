use super::ser::serialize_phab;
use serde::Serialize;
use serde::{de::DeserializeOwned, Deserialize};
use thiserror::Error;
use url::Url;

pub trait ApiRequest: Serialize {
    type Reply: DeserializeOwned + std::fmt::Debug;
    const ROUTE: &'static str;

    fn route(&self) -> &'static str {
        Self::ROUTE
    }
}

#[derive(Debug, Serialize)]
struct Request<'a, R: Serialize> {
    #[serde(rename = "api.token")]
    token: &'a str,
    #[serde(flatten, serialize_with = "serialize_phab")]
    request: &'a R,
}

#[derive(Debug, Deserialize)]
struct Reply<R> {
    error_code: Option<String>,
    error_info: Option<String>,
    result: Option<R>,
}

#[derive(Debug, Clone)]
pub struct Client {
    base: Url,
    token: String,
    client: reqwest::Client,
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("API Error ({code}): {info}")]
    Api { code: String, info: String },
    #[error("Incomplete reply")]
    Incomplete,
    #[error("Request failure {0}")]
    Request(#[from] reqwest::Error),
}

impl Client {
    pub fn new(base: Url, token: String) -> Client {
        let client = reqwest::Client::new();
        Client {
            base,
            token,
            client,
        }
    }

    pub async fn request<R>(&self, request: &R) -> Result<R::Reply, RequestError>
    where
        R: ApiRequest,
    {
        let u = self.base.join(request.route()).unwrap();
        let t = Request {
            token: &self.token,
            request,
        };

        let response = self.client.post(u).form(&t).send().await?;
        let reply: Reply<R::Reply> = response.error_for_status()?.json().await?;

        match reply {
            Reply {
                result: Some(r), ..
            } => Ok(r),
            Reply {
                error_code: Some(code),
                error_info: Some(info),
                ..
            } => Err(RequestError::Api { code, info }),
            _ => Err(RequestError::Incomplete),
        }
    }
}

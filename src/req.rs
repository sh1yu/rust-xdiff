use crate::{ExtraArgs, ResponseProfile};
use anyhow::{anyhow, Result};
use dialoguer::Validator;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{header, Client, Method, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestProfile {
    #[serde(with = "http_serde::method", default)]
    pub method: Method,
    pub url: Url,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub params: Option<serde_json::Value>,
    #[serde(
        skip_serializing_if = "HeaderMap::is_empty",
        with = "http_serde::header_map",
        default
    )]
    pub headers: HeaderMap,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub body: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct ResponseExt(Response);

impl RequestProfile {
    pub fn new(
        method: Method,
        url: Url,
        params: Option<serde_json::Value>,
        headers: HeaderMap,
        body: Option<serde_json::Value>,
    ) -> Self {
        Self {
            method,
            url,
            params,
            headers,
            body,
        }
    }

    pub async fn send(&self, args: &ExtraArgs) -> Result<ResponseExt> {
        let (headers, query, body) = self.generate(args)?;
        let res = Client::new()
            .request(self.method.clone(), self.url.clone())
            .query(&query)
            .headers(headers)
            .body(body)
            .send()
            .await?;
        Ok(ResponseExt(res))
    }

    pub fn generate(&self, args: &ExtraArgs) -> Result<(HeaderMap, serde_json::Value, String)> {
        let mut headers = self.headers.clone();
        let mut query = self.params.clone().unwrap_or(json!({}));
        let mut body = self.body.clone().unwrap_or(json!({}));

        for (k, v) in &args.headers {
            headers.insert(HeaderName::from_str(k)?, HeaderValue::from_str(v)?);
        }
        if !headers.contains_key(header::CONTENT_TYPE) {
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
        }
        for (k, v) in &args.query {
            query[k] = v.parse()?;
        }
        for (k, v) in &args.body {
            body[k] = v.parse()?;
        }

        let content_type = get_content_type(&headers);
        let body = match content_type.as_deref() {
            Some("application/json") => serde_json::to_string(&body)?,
            Some("application/x-www-form-urlencoded" | "multipart/form-data") => {
                serde_urlencoded::to_string(&body)?
            }
            _ => Err(anyhow!("unsupported content-type"))?,
        };

        Ok((headers, query, body))
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if let Some(params) = self.params.as_ref() {
            if !params.is_object() {
                Err(anyhow!(
                    "params must be an object:\n{}",
                    serde_yaml::to_string(params)?
                ))?;
            }
        }
        if let Some(body) = self.body.as_ref() {
            if !body.is_object() {
                Err(anyhow!(
                    "body must be an object:\n{}",
                    serde_yaml::to_string(body)?
                ))?;
            }
        }
        Ok(())
    }
}

impl FromStr for RequestProfile {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut profile: RequestProfile = serde_yaml::from_str(s)?;
        profile.validate()?;
        Ok(profile)
    }
}

impl ResponseExt {
    pub async fn filter_text(self, profile: &ResponseProfile) -> Result<String> {
        let mut output = String::new();
        output.push_str(&format!("{:?} {}\n", self.0.version(), self.0.status()));

        let headers = self.0.headers();
        for (k, v) in headers.iter() {
            if !profile.skip_headers.iter().any(|sh| sh == k.as_str()) {
                output.push_str(&format!("{}: {:?}\n", k, v))
            }
        }

        output.push_str("\n");

        let content_type = get_content_type(&headers);
        let text = self.0.text().await?;
        let text = match content_type.as_deref() {
            Some("application/json") => filter_json(&text, &profile.skip_body)?,
            _ => text,
        };
        output.push_str(&text);

        Ok(output)
    }
}

fn get_content_type(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().split(";").next())
        .flatten()
        .map(|v| v.to_string())
}

fn filter_json(text: &str, skip: &[String]) -> Result<String> {
    let mut json: serde_json::Value = serde_json::from_str(text)?;
    match json {
        serde_json::Value::Object(ref mut obj) => {
            for k in skip {
                obj.remove(k);
            }
        }
        _ => {}
    }
    Ok(serde_json::to_string_pretty(&json)?)
}

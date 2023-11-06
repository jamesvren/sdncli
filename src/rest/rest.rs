use crate::{
    config,
    rest::output::{json_output, json_to_table},
    rest::resource::ResourceBuilder,
};
use anyhow::anyhow;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::{self, Client, Method, RequestBuilder, Response};
use serde_json::{self, json, Value};
use std::{
    collections::HashMap,
    io::{self, Write},
    time::Instant,
};
use url::Url;
use uuid::Uuid;

pub struct Rest {
    rest: config::Rest,
    auth: config::Auth,
    client: Client,
    token: String,
}

#[async_trait]
pub trait RestBench {
    async fn send_bench(self, bench: bool) -> anyhow::Result<Response>;
}

#[async_trait]
impl RestBench for RequestBuilder {
    async fn send_bench(self, bench: bool) -> anyhow::Result<Response> {
        let now = Instant::now();
        let response = self.send().await?;
        if bench {
            println!(
                "time: {} [status: {} length: {:?}]",
                now.elapsed().as_secs_f32(),
                response.status(),
                response.content_length(),
            );
        }
        debug!("{:#?}", response);
        match response.error_for_status_ref() {
            Ok(_) => Ok(response),
            Err(e) => Err(anyhow!("{}\n{}", e, response.text().await?)),
        }
    }
}

#[async_trait]
pub trait Output {
    async fn output(self, fmt: &str, fields: Option<Vec<String>>) -> anyhow::Result<()>;
}

#[async_trait]
impl Output for Response {
    async fn output(self, fmt: &str, fields: Option<Vec<String>>) -> anyhow::Result<()> {
        match self.content_length() {
            Some(0) => Ok(()),
            _ => {
                let text = self.text().await?;
                debug!("Output Response: {}", text);
                match serde_json::from_str::<Value>(&text) {
                    Ok(json_value) if fmt == "table" => Ok(json_to_table(&json_value, fields)),
                    Ok(json_value) => Ok(json_output(&json_value)),
                    Err(_) => Ok(println!("{}", text)),
                }
            }
        }
    }
}

impl Rest {
    pub fn new(cfg: &config::Config) -> Self {
        Self {
            rest: cfg.api.clone(),
            auth: cfg.auth.clone(),
            client: Client::new(),
            token: String::new(),
        }
    }
    // request token
    // send request
    // measure time
    // output result with special format

    pub fn set_rest_port(&mut self, port: u32) {
        self.rest.port = port;
    }

    pub async fn request(
        &mut self,
        method: Method,
        uri: &str,
        body: Option<Value>,
    ) -> anyhow::Result<RequestBuilder> {
        if self.token.is_empty() {
            self.token = get_token(&self.auth).await?;
        };
        // Use auth host if rest host does not set
        let host = match (&self.rest.host, &self.auth.host) {
            (Some(host), _) => host,
            (_, host) => host,
        };
        let url = Url::parse(&format!("http://{}:{}/", host, self.rest.port))?.join(uri)?;
        if body.is_some() {
            info!("curl -D - -s -X {} {} -H \"Content-Type:application/json\" -H \"X-Auth-Token:{}\" -d '{}'",
                  method.as_str(), url, self.token, body.as_ref().unwrap());
        } else {
            info!(
                "curl -D - -s -X {} {} -H \"Content-Type:application/json\" -H \"X-Auth-Token:{}\"",
                method.as_str(),
                url,
                self.token
            );
        }
        let builder = self
            .client
            .request(method, url)
            .header("Content-Type", "application/json")
            .header("X-Auth-Token", &self.token);
        if let Some(body) = body {
            return Ok(builder.json(&body));
        }
        Ok(builder)
    }

    pub async fn post(&mut self, uri: &str, body: Value) -> anyhow::Result<Response> {
        self.request(reqwest::Method::POST, uri, Some(body))
            .await?
            .send_bench(true)
            .await
    }

    pub async fn get(&mut self, uri: &str) -> anyhow::Result<Response> {
        self.request(reqwest::Method::GET, uri, None)
            .await?
            .send_bench(true)
            .await
    }

    pub async fn name_to_id(&mut self, uri: &str, name: &str) -> anyhow::Result<Uuid> {
        let body = ResourceBuilder::new()
            .res_type(uri)
            .filters(json!({"name": name}))
            .build()?;
        let response: Vec<HashMap<String, Value>> = self
            .request(reqwest::Method::POST, uri, Some(body))
            .await?
            .send_bench(false)
            .await?
            .error_for_status()?
            .json()
            .await?;
        debug!("FQ Name: {:#?}", response);
        let wanted: Vec<HashMap<String, Value>> = response
            .into_iter()
            .filter(|res| res.get("name") == Some(json! {name}).as_ref())
            .collect();
        match wanted.len() {
            0 => Err(anyhow!("{} {} Not Found", uri.split("/").last().unwrap(), name)),
            1 => Ok(Uuid::parse_str(wanted[0]["id"].as_str().unwrap())?),
            _ => {
                println!("@@ Found multiple {}:", name);
                wanted
                    .iter()
                    .enumerate()
                    .for_each(|(i, res)| println!("{} = {}:{}", i, res["id"], res["fq_name"]));
                print!("Please select: ");
                io::stdout().flush().unwrap();

                // Get index from stdin user input
                let mut number = String::new();
                io::stdin().read_line(&mut number)?;
                let index = number.trim().parse::<usize>()?;
                if index >= wanted.len() {
                    Err(anyhow!(
                        "Your select {} is not in range 0-{}",
                        index,
                        wanted.len() - 1
                    ))
                } else {
                    Ok(Uuid::parse_str(wanted[index]["id"].as_str().unwrap())?)
                }
            }
        }
    }
}

pub async fn get_token(cfg: &config::Auth) -> anyhow::Result<String> {
    let version = cfg.version.to_lowercase();
    let (uri, body) = match &version as &str {
        "v3" => (
            format!("http://{}:{}/v3/auth/tokens", cfg.host, cfg.port),
            json!({
                "auth": {
                    "identity": {
                        "methods":["password"],
                        "password": {
                            "user": {
                                "name": cfg.user,
                                "password": cfg.password,
                                "domain": { "name": "Default" }
                            }
                        }
                    }
                }
            }),
        ),
        _ => (
            format!("http://{}:{}/v2.0/tokens", cfg.host, cfg.port),
            json!({
                "auth": {
                    "tenantName": cfg.project,
                    "passwordCredentials": {
                        "username": cfg.user,
                        "password": cfg.password,
                    }
                }
            }),
        ),
    };
    info!(
        "curl -D - -s -X POST {} -H \"Content-Type:application/json\" -d '{}'",
        uri, body
    );
    let response = reqwest::Client::new()
        .post(uri)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;
    debug!("{:#?}", response);
    match &version as &str {
        "v3" => Ok(response.headers()["x-subject-token"]
            .to_str()
            .unwrap()
            .to_owned()),
        _ => {
            let token: Value = response.json().await?;
            Ok(token["access"]["token"]["id"].as_str().unwrap().to_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rest() -> Result<(), anyhow::Error> {
        let cfg = config::read_config()?;
        let mut api = Rest::new(&cfg);
        let body = json!({
            "data": {
                "fields": [],
                "filters": {}
            },
            "context": {
                "user_id": "ad9b5511b71342ef94c4c65f834dde55",
                "tenant_id": "b00c06f6d02e44939073b566f409b496",
                "is_admin": true,
                "request_id": "req-b5693b56-589c-4f24-80a2-6e97c3dc8ab2",
                "operation": "READALL",
                "type": "network"
            }
        });
        let _response = api
            .request(Method::POST, String::from("neutron/network"), body)
            .await?;
        Ok(())
    }
}

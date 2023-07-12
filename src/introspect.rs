use reqwest::{self, Client, Response, RequestBuilder, Method};
use roxmltree::{
    Document,
};
use log::debug;

pub struct Introspect {
    http: Client,
    base_url: String,
    service: String,
}

impl Introspect {
    pub fn new(ip: &str, port: u32, service: String) -> Self {
        Self {
            http: Client::new(),
            service: service,
            base_url: format!("http://{}:{}/", ip, port),
        }
    }
    pub async fn get(self, url: &str) -> anyhow::Result<()> {
        let response = self.http.get(self.base_url + url)
            .send()
            .await?
            .text()
            .await?;
        debug!("{response}");
        xml_parser(&response)
    }
    pub async fn set_logging(self, level: &str) -> anyhow::Result<()> {
        let url = format!(
            "Snh_SandeshLoggingParamsSet?enable=&category=&log_level={}&trace_print=&enable_flow_log=",
            level
            );
        let response = self.http.get(self.base_url + url.as_str())
            .send()
            .await?
            .text()
            .await?;
        xml_parser(&response)
    }
}

fn xml_parser(data: &str) -> anyhow::Result<()> {
    let doc = Document::parse(data)?;
    debug!("{doc:#?}");

    // Parse `a` tag
    for node in doc.descendants() {
        if node.is_element() && node.has_tag_name("a") {
            match node.text() {
                Some("\n") | None => (),
                Some(n) => println!("{n}"),
            }
        }
    }

    // Handle log level
    if let Some(node) = doc.descendants().find(|n| n.has_tag_name("log_level")) {
        //if node.is_element() {
        //    println!("{:#?}", node);
        //}
        if let Some(level) = node.text() {
            println!("Current LOG Level: {level}");
        }
    }
    Ok(())
}


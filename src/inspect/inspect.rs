use chrono::DateTime;
use colored::Colorize;
use log::{debug, info};
use quick_xml::escape::unescape;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use reqwest::{self, Client};
use std::ops::Deref;
use url::Url;

pub struct Introspect {
    http: Client,
    root: String,
}

impl Introspect {
    pub fn new(ip: &str, port: u32) -> Self {
        Self {
            http: Client::new(),
            root: format!("http://{}:{}/", ip, port),
        }
    }

    pub async fn get(self, url: &str) -> anyhow::Result<()> {
        let uri = Url::parse(&self.root)?.join(url)?;
        info!("Request: {}", uri.as_str());
        let response = self.http.get(uri.as_str()).send().await?.text().await?;
        info!("Response: {response}");
        xml_parser(&response, url)
    }

    pub async fn set_logging(self, level: &str) -> anyhow::Result<()> {
        let url = format!(
            "Snh_SandeshLoggingParamsSet?enable=&category=&log_level={}&trace_print=&enable_flow_log=",
            level
            );
        self.get(&url).await
    }

    pub async fn get_logging(self) -> anyhow::Result<()> {
        self.get("Snh_SandeshLoggingParamsSet?enable=&category=&log_level=&trace_print=&enable_flow_log=").await
    }
    pub async fn get_trace(self, buffer: &Option<String>) -> anyhow::Result<()> {
        match buffer {
            Some(buffer) => {
                let url = format!("Snh_SandeshTraceRequest?x={}", buffer);
                self.get(&url).await
            }
            None => self.get("Snh_SandeshTraceBufferListRequest?").await,
        }
    }
    pub async fn get_uve(self, uve: &Option<String>) -> anyhow::Result<()> {
        match uve {
            Some(uve) => {
                let url = format!("Snh_SandeshUVECacheReq?tname={}", uve);
                self.get(&url).await
            }
            None => self.get("Snh_SandeshUVETypesReq?").await,
        }
    }
}

fn xml_parser(xml: &str, url: &str) -> anyhow::Result<()> {
    let mut reader = Reader::from_str(xml);
    let mut txt = Vec::new();
    let mut is_sandesh = false;
    let mut is_trace = false;
    let mut is_root = false;
    let mut is_uve = false;

    match url {
        x if x.starts_with("Snh_SandeshTrace") => is_trace = true,
        x if x.starts_with("Snh_SandeshUVETypesReq") => is_uve = true,
        x if x.ends_with(".xml") => is_sandesh = true,
        "/" => is_root = true,
        x if x.starts_with("Snh_") => {
            return format_xml(xml, false);
        }
        _ => unreachable!(),
    }
    //if url.starts_with("Snh_SandeshTrace") {
    //    is_trace = true;
    //} if else if url.ends_with(".xml") {
    //    is_sandesh = true;
    //} else if url == "/" {
    //    is_root = true;
    //} else if url.starts_with("Snh_") {
    //    return format_xml(xml, false);
    //}

    //reader.trim_text(true);
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                debug!("{e:#?}");
                match e.name().as_ref() {
                    b"trace_buf_name" | b"log_level" => {
                        txt.push(reader.read_text(e.name())?.to_string())
                    }
                    b"type_name" => {
                        if is_uve {
                            txt.push(reader.read_text(e.name())?.to_string())
                        }
                    }
                    b"element" => {
                        let text = reader.read_text(e.name())?;
                        if is_trace {
                            // Convert timestamp to datetime
                            let mut text = text.splitn(2, ' ');
                            let timestamp = text.next().unwrap();
                            let tms =
                                DateTime::from_timestamp_micros(timestamp.parse::<i64>()?)
                                    .unwrap();
                            txt.push(
                                format!("{tms} \u{1F449} {0} 👌", unescape(text.next().unwrap())?),
                            );
                        } else {
                            txt.push(text.to_string());
                        }
                    }
                    _ => {
                        if is_sandesh {
                            let sandesh = e
                                .attributes()
                                .filter(|a| {
                                    a.clone().unwrap().key.as_ref() == b"type"
                                        && a.clone().unwrap().unescape_value().unwrap() == "sandesh"
                                })
                                .collect::<Vec<_>>();
                            if !sandesh.is_empty() {
                                txt.push(std::str::from_utf8(e.name().into_inner())?.to_string());
                            }
                        } else if is_root {
                            let attr = e
                                .attributes()
                                .map(|a| a.unwrap().unescape_value().unwrap().into_owned())
                                .collect::<Vec<_>>();
                            for i in attr {
                                txt.push(i);
                            }
                        }
                    }
                }
            }
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => break,
            //ev => println!("{:?}", ev),
            _ => (),
        }
    }
    for t in txt {
        println!("{t}");
    }
    Ok(())
}

pub fn format_xml(xml: &str, include_attr: bool) -> anyhow::Result<()> {
    let mut indent = 0;
    let mut reader = Reader::from_str(xml);
    const STEP: usize = 4;
    let mut start = String::new();
    let mut attr = String::new();
    let mut text = String::new();
    let mut end;
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                if !start.is_empty() {
                    if include_attr {
                        println!("{}<{} {}>", " ".repeat(indent), start.yellow(), attr);
                    } else {
                        println!("{}<{}>", " ".repeat(indent), start.yellow());
                    }
                    indent += STEP;
                }
                let name = reader.decoder().decode(e.name().into_inner()).unwrap();
                if include_attr {
                    attr = reader
                        .decoder()
                        .decode(e.attributes_raw())
                        .unwrap()
                        .to_string();
                }
                start = name.to_string();
            }
            Ok(Event::Text(e)) => {
                text = reader.decoder().decode(e.deref()).unwrap().to_string();
            }
            Ok(Event::End(e)) => {
                end = reader.decoder().decode(e.deref()).unwrap();
                if end == start {
                    if include_attr {
                        println!(
                            "{}<{} {}>{}<{}>",
                            " ".repeat(indent),
                            start.yellow(),
                            attr,
                            text.italic().bright_purple(),
                            end
                        );
                        attr = String::new();
                    } else {
                        println!(
                            "{}<{}>{}<{}>",
                            " ".repeat(indent),
                            start.yellow(),
                            text.italic().bright_purple(),
                            end
                        );
                    }
                    start = String::new();
                    text = String::new();
                } else {
                    indent -= STEP;
                    println!("{}</{}>", " ".repeat(indent), end);
                }
            }
            Ok(Event::PI(e)) => {
                if include_attr {
                    println!("{}", reader.decoder().decode(e.deref()).unwrap());
                }
            }
            Ok(Event::CData(e)) => {
                if include_attr {
                    println!("{}", reader.decoder().decode(e.deref()).unwrap());
                }
            }
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => break,
            _ => (),
        }
    }
    Ok(())
}

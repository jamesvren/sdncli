mod cli;
mod config;
mod db;
mod inspect;
mod rest;

use chrono::NaiveDateTime;
use clap::{ArgMatches, FromArgMatches as _};
use cli::{Operations, Opts, OutputFormat, Method, BUILDIN_CMD};
use inspect::format_xml;
use rest::resource::ResourceBuilder;
use rest::rest::get_token;
use rest::rest::Output;
use rest::rest::Rest;
use serde_json::{json, Value};
use uuid::Uuid;
use flate2::read::ZlibDecoder;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let cli = cli::build_cli()?;
    let cli = db::cli::build_cli(cli);
    let cli = inspect::cli::build_cli(cli);

    let matches = cli.get_matches();
    let opt = cli::cli_matches(&matches);
    if handle_cli(&opt).await? {
        return Ok(());
    }
    if handle_buildin(&matches).await? {
        return Ok(());
    }
    handle_rest(&matches, &opt).await?;
    db::cli::handle_cli(&matches).await?;
    inspect::cli::handle_cli(&matches).await?;

    Ok(())
}

async fn handle_cli(opt: &Opts) -> Result<bool, anyhow::Error> {
    let cfg = config::read_config()?;
    let mut api = Rest::new(&cfg);
    let output_format = opt.output;

    // Request by JSON file
    if let Some(file) = &opt.file {
        let oformat = output_format.unwrap_or(OutputFormat::Table).to_string();
        let json_config = config::read_json_file(file)?;

        if json_config.port.is_some() {
            api.set_rest_port(json_config.port.unwrap());
        }
        if json_config.method == "get" {
            api.get(&json_config.uri)
                .await?
                .output(&oformat, None)
                .await?;
        } else if json_config.method == "post" {
            api.post(&json_config.uri, json_config.body)
                .await?
                .output(&oformat, None)
                .await?;
        }
        println!("API IP: {}", api.host);
        return Ok(true);
    }

    // Request by URL
    if let Some(uri) = &opt.uri {
        if let Some(port) = opt.port {
            api.set_rest_port(port);
        }

        let oformat = output_format.unwrap_or(OutputFormat::Json).to_string();
        if let Some(data) = &opt.data {
            api.post(uri, data.clone())
                .await?
                .output(&oformat, None)
                .await?;
        } else {
            if let Some(method) = &opt.method {
                if method == &Method::Delete {
                    api.delete(uri).await?.output(&oformat, None).await?;
                }
            }
            api.get(uri).await?.output(&oformat, None).await?;
        }
        println!("API IP: {}", api.host);
        return Ok(true);
    }

    // Convert timestamp to datetime
    if let Some(timestamp) = opt.timestamp {
        const A_BILLION: i64 = 1_000_000_000;
        let date = match timestamp / A_BILLION {
            0..=9 => NaiveDateTime::from_timestamp_opt(timestamp, 0),
            10..=9999 => NaiveDateTime::from_timestamp_millis(timestamp),
            _ => NaiveDateTime::from_timestamp_micros(timestamp),
        };
        println!("{date:?}");
        return Ok(true);
    }

    // Format XML string
    if let Some(xml) = &opt.xml {
        let _ = format_xml(xml, true);
        return Ok(true);
    }

    // Format JSON string
    if let Some(text) = &opt.json {
        let json: Value = match hex::decode(text) {
            Ok(txt) => {
                let mut decoder = ZlibDecoder::new(txt.as_slice());
                let mut s = String::new();
                let _ = decoder.read_to_string(&mut s)?;
                println!("{s}");
                println!("{}", "=".repeat(80));
                let s = s
                    .replace("\"", "\\\"")
                    .replace("u'", "\"")
                    .replace("'", "\"")
                    .replace("True", "true")
                    .replace("False", "false")
                    .replace("None", "null")
                    .replace("L", "");
                serde_json::from_str(&s)?
            }
            Err(_) => serde_json::from_str(text)?,
        };

        println!("{json:#}");
        return Ok(true);
    }

    // Get API cache
    if opt.cache {
        let oformat = output_format.unwrap_or(OutputFormat::Json).to_string();
        api.post("/obj-cache", json!({"count": 999999}))
            .await?
            .output(&oformat, None)
            .await?;
        println!("API IP: {}", api.host);
        return Ok(true);
    }

    Ok(false)
}

async fn handle_buildin(matches: &ArgMatches) -> Result<bool, anyhow::Error> {
    let mut handled = false;
    let cfg = config::read_config()?;
    for cmd in &BUILDIN_CMD {
        if let Some(_matches) = matches.subcommand_matches(cmd) {
            handled = true;
            if *cmd == "token" {
                println!("{}", get_token(&cfg.auth).await?);
            }
        }
    }
    Ok(handled)
}

async fn handle_rest(matches: &ArgMatches, opt: &Opts) -> Result<(), anyhow::Error> {
    let cfg = config::read_config()?;
    let mut api = Rest::new(&cfg);
    let oformat = opt.output.unwrap_or(OutputFormat::Table).to_string();
    let mut builder = ResourceBuilder::new();

    for res in cfg.resource {
        if let Some(matches) = matches.subcommand_matches(res.cmd.as_str()) {
            let uri: String;
            if res.resource == "member" {
                let pool = matches.get_one::<String>("pool").unwrap();
                let pool_id = match Uuid::parse_str(pool) {
                    Ok(id) => id,
                    Err(_) => api.name_to_id("/neutron/pool", pool).await?,
                };
                uri = format!("/neutron/pool/{pool_id}/member");
            } else {
                uri = res.uri;
            }
            let opers = Operations::from_arg_matches(matches)
                .map_err(|err| err.exit())
                .unwrap();
            let oper = opers.oper();
            builder.res_type(&res.resource).oper(&oper);

            let (names, attr, field, filter) = match opers {
                Operations::Create { name, attr } => (Some(vec![name]), attr, None, None),
                Operations::Update { names, attr } => (Some(names), Some(attr), None, None),
                Operations::Delete { names } => (Some(names), None, None, None),
                Operations::Show { names, field } => (Some(names), None, field, None),
                Operations::List { filter, field } => (None, None, field, filter),
                Operations::Oper {
                    name, attr, field, ..
                } => (Some(vec![name]), attr, field, None),
            };

            if let Some(filters) = filter {
                builder.filters(filters.clone());
            };

            if let Some(fields) = &field {
                builder.fields(fields.to_vec());
            };

            if let Some(attr) = attr {
                for a in attr {
                    builder.resource(a.as_object().unwrap().clone());
                }
            }

            // This should be last action since it will send request.
            if let Some(names) = &names {
                for name in names {
                    if oper == "CREATE" {
                        builder.name(name);
                    } else {
                        match Uuid::parse_str(name) {
                            Ok(id) => builder.id(id),
                            Err(_) => {
                                let id = api.name_to_id(&uri, name).await?;
                                builder.name(name).id(id)
                            }
                        };
                    }
                    let body = builder.build()?;
                    api.post(&uri, body)
                        .await?
                        .output(&oformat, field.clone())
                        .await?;
                }
            } else {
                let body = builder.build()?;
                api.post(&uri, body).await?.output(&oformat, field).await?;
            }
            println!("API IP: {}", api.host);
        }
    }

    Ok(())
}

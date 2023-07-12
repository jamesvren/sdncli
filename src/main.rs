use clap::Parser;
use anyhow;
use uuid::Uuid;

use sdncli::{
    rest::{self, Rest, Output},
    config,
    Arg,
    arg::Commands,
    resource::ResourceBuilder,
    introspect::Introspect,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {

    env_logger::init();
    let cfg = config::read_config()?;
    let opts = Arg::parse();
    let output_format = opts.output;
    let mut builder = ResourceBuilder::new();
    let mut api = Rest::new(&cfg);

    let oformat = match output_format {
        Some(o) => o.to_string(),
        None => "table".to_string(),
    };
    // Read request from JSON file
    if let Some(file) = &opts.file {
        let json_config = config::read_json_file(file)?;
        if json_config.port.is_some() {
            api.set_rest_port(json_config.port.unwrap());
        }
        if json_config.method == "get" {
            api.get(&json_config.uri).await?
               .output(&oformat, None).await?;
        } else if json_config.method == "post" {
            api.post(&json_config.uri, json_config.body).await?
               .output(&oformat, None).await?;
        }
        return Ok(())
    }

    // Do request from URI
    if let Some(uri) = &opts.uri {
        let oformat = match output_format {
            Some(o) => o.to_string(),
            None => "json".to_string(),
        };
        api.get(uri).await?
            .output(&oformat, None).await?;
        return Ok(())
    }
    // Else use command to build request body
    if let Some(cmd) = &opts.command {
        match cmd {
            Commands::Create { resource, name , attr, .. } |
            Commands::Oper { resource, name , attr, .. } => {
                let res = cfg.get_resource(resource)?;
                let oper = cmd.oper();
                if oper == "CREATE" {
                    builder.name(name);
                } else {
                    match Uuid::parse_str(name) {
                        Ok(id) => builder.id(id),
                        Err(_) => {
                            let id = api.name_to_id(&res.uri, name).await?;
                            builder.name(name).id(id)
                        },
                    };
                }
                if let Some(attr) = attr {
                    for a in attr {
                        builder.resource(a.as_object().unwrap().clone());
                    }
                }
                let body = builder
                    .res_type(&res.resource)
                    .oper(&oper)
                    .build()?;

                api.post(&res.uri, body).await?.output(&oformat, None).await?;
            },
            Commands::Update { resource, names , attr } => {
                let res = cfg.get_resource(resource)?;
                if let Some(attr) = attr {
                    for a in attr {
                        builder.resource(a.as_object().unwrap().clone());
                    }
                }
                builder
                    .res_type(&res.resource)
                    .oper(&cmd.oper());
                for name in names {
                    match Uuid::parse_str(name) {
                        Ok(id) => builder.id(id),
                        Err(_) => {
                            let id = api.name_to_id(&res.uri, name).await?;
                            builder.name(name).id(id)
                        },
                    };
                    let body = builder.build()?;

                    api.post(&res.uri, body).await?.output(&oformat, None).await?;
                }
            },
            Commands::Show { resource, names, field } |
            Commands::Delete { resource, names, field } => {
                let res = cfg.get_resource(resource)?;
                if let Some(fields) = field {
                    builder.fields(fields.to_vec());
                };
                builder
                    .res_type(&res.resource)
                    .oper(&cmd.oper());
                for name in names {
                    match Uuid::parse_str(name) {
                        Ok(id) => builder.id(id),
                        Err(_) => {
                            let id = api.name_to_id(&res.uri, name).await?;
                            builder.name(name).id(id)
                        },
                    };
                    let body = builder.build()?;

                    api.post(&res.uri, body).await?.output(&oformat, field.clone()).await?;
                }
            },
            Commands::List { resource, filter, field } => {
                let res = cfg.get_resource(resource)?;
                if let Some(filters) = filter {
                    builder.filters(filters.clone());
                };
                if let Some(fields) = field {
                    builder.fields(fields.to_vec());
                };
                let body = builder
                    .res_type(&res.resource)
                    .oper(&cmd.oper())
                    .build()?;

                api.post(&res.uri, body).await?.output(&oformat, field.clone()).await?;
            },
            Commands::Token => {
               println!("{}", rest::get_token(&cfg.auth).await?);
               return Ok(());
            },
            Commands::Inspect(inspect) => {
                let ist = Introspect::new(&inspect.ip,
                                           inspect.service.to_port(),
                                           inspect.service.to_string());
                if let Some(level) = inspect.log {
                    ist.set_logging(level.to_syslog()).await?;
                } else {
                    ist.get("Snh_SandeshUVECacheReq?tname=NodeStatus").await?;
                }
            },
        }
    }

    Ok(())
}

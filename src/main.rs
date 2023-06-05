use clap::Parser;
use anyhow;
use uuid::Uuid;

use sdncli::{
    rest::{self, Rest, Output},
    config,
    Arg,
    arg::Commands,
    resource::ResourceBuilder,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {

    env_logger::init();
    let cfg = config::read_config()?;
    //println!("{:#?}", cfg.as_table().unwrap());
    //println!("{:#?}", cfg.resource);
    //for c in cfg.as_table().unwrap().iter() {
    //    println!("{:#?}", c);
    //}
    //println!("{:#?}", cfg.iter());
    //for i in 0..10 {
    //    println!("{:#?}", cfg.get(i));
    //}

    let opts = Arg::parse();
    let output_format = opts.output.to_string();
    let mut builder = ResourceBuilder::new();
    let mut api = Rest::new(&cfg);

    // Read request from JSON file
    if let Some(file) = &opts.file {
        let json_config = config::read_json_file(file)?;
        if json_config.port.is_some() {
            api.set_rest_port(json_config.port.unwrap());
        }
        if json_config.method == "get" {
            api.get(&json_config.uri).await?.output(&output_format, None).await?;
        } else if json_config.method == "post" {
            api.post(&json_config.uri, json_config.body).await?.output(&output_format, None).await?;
        }
        return Ok(())
    }

    // Else use command to build request body
    if let Some(cmd) = &opts.command {
        match cmd {
            Commands::Create { resource, name , attr } |
            Commands::Update { resource, name , attr } |
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
                    builder.resource(attr.as_object().unwrap().clone());
                }
                let body = builder
                    .res_type(&res.resource)
                    .oper(&oper)
                    .build()?;

                api.post(&res.uri, body).await?.output(&output_format, None).await?;
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

                    api.post(&res.uri, body).await?.output(&output_format, field.clone()).await?;
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

                api.post(&res.uri, body).await?.output(&output_format, field.clone()).await?;
            },
            Commands::Token => {
               println!("{}", rest::get_token(&cfg.auth).await?);
               return Ok(());
            }
        }
    }

    Ok(())
}

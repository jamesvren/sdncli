use super::config::read_config;
use clap::{
    arg, command, ArgMatches, Args, Command, FromArgMatches as _, Parser, Subcommand as _,
    ValueEnum,
};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Args)]
pub struct Opts {
    /// Read JSON from file. Please refer to `resource.json`
    #[arg(short, long, group = "input")]
    pub file: Option<PathBuf>,

    /// API Port for Request URI
    #[arg(short, long, requires = "url")]
    pub port: Option<u32>,

    /// Request method
    #[arg(short, long, value_enum, requires = "url")]
    pub method: Option<Method>,

    /// Request data
    #[arg(short, long, value_parser = json_parser, requires = "url")]
    pub data: Option<Value>,

    /// Request URI
    #[arg(short, long, group = "input", group = "url")]
    pub uri: Option<String>,

    ///// Print debug message
    //#[arg(long, action=ArgAction::SetFalse)]
    //pub debug: bool,
    #[arg(
        short,
        long,
        //default_value = OutputFormat::Table,
        value_enum,
        help = "Output format for response",
    )]
    pub output: Option<OutputFormat>,

    /// Convert UNIX timestamp to UTC datetime
    #[arg(short, long)]
    pub timestamp: Option<i64>,

    /// Format XML string
    #[arg(short, long)]
    pub xml: Option<String>,

    /// Format JSON string
    #[arg(short, long)]
    pub json: Option<String>,

    /// Get API cache
    #[arg(long)]
    pub cache: bool,
}

pub const BUILDIN_CMD: [&str; 3] = ["token", "loadbalance", "vgws"];

pub fn build_cli() -> Result<Command, anyhow::Error> {
    let cmd = command!()
        .propagate_version(false)
        .subcommand_required(false)
        .arg_required_else_help(true);
    build_dynamic_cli(Opts::augment_args(cmd))
}

fn build_dynamic_cli(mut cli: Command) -> Result<Command, anyhow::Error> {
    let cfg = read_config()?;

    for cmd in &BUILDIN_CMD {
        let mut sub = Command::new(cmd).about("Buildin CMD");
        if cmd == &"loadbalance" {
            sub = Operations::augment_subcommands(sub).subcommand_required(true);
        }
        //let sub = SubArgs::augment_args(sub);
        cli = cli.subcommand(sub);
    }

    // custom command
    for res in cfg.resource {
        let mut sub = Command::new(res.cmd).about(format!("- {}", res.resource));
        if res.resource == "member" {
            sub = sub.arg(
                arg!(
                    -p --pool <POOL> "The pool this member belong to"
                )
                .required(true),
            );
        }
        let sub = Operations::augment_subcommands(sub);
        cli = cli.subcommand(sub);
    }
    Ok(cli)
}

pub fn cli_matches(matches: &ArgMatches) -> Opts {
    Opts::from_arg_matches(matches)
        .map_err(|err| err.exit())
        .unwrap()
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Text,
}

impl ToString for OutputFormat {
    fn to_string(&self) -> String {
        match self {
            OutputFormat::Table => String::from("table"),
            OutputFormat::Json => String::from("json"),
            OutputFormat::Text => String::from("text"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Method {
    Post,
    Put,
    Get,
    Delete,
}

impl ToString for Method {
    fn to_string(&self) -> String {
        match self {
            Method::Post => String::from("post"),
            Method::Put => String::from("put"),
            Method::Get => String::from("get"),
            Method::Delete => String::from("delete"),
        }
    }
}

#[derive(Parser)]
pub enum Operations {
    /// Create a resouce
    Create {
        /// Name of resource to be created
        name: String,
        /// Resource attributes, add `'` for list or dict.
        /// Example: -a binding:vif_details='{"port_filter":true}'
        #[arg(short, long, value_parser = key_val_parser)]
        attr: Option<Vec<Value>>,
    },
    /// Update a resource with some attributes
    Update {
        /// ID or Name of resource(s) to be updated, example: --names=james,james1
        #[arg(value_delimiter = ',', required = true)]
        names: Vec<String>,
        /// Resource attributes, add `'` for list or dict.
        /// Example: -a binding:vif_details='{"port_filter":true}'
        #[arg(short, long, value_parser = key_val_parser)]
        attr: Vec<Value>,
    },
    /// Delete resources
    Delete {
        /// ID or Name of resource(s) to be deleted, example: --names=james,james1
        #[arg(value_delimiter = ',', required = true)]
        names: Vec<String>,
    },
    /// Display detail for some resource(s)
    Show {
        /// ID or Name of resource(s) to be displayed, example: --names=james,james1
        #[arg(value_delimiter = ',', required = true)]
        names: Vec<String>,
        /// Fields to be displayed, example: --field=name,id
        #[arg(short, long, value_delimiter = ',')]
        field: Option<Vec<String>>,
    },
    /// Display all resources
    List {
        /// JSON format filter, example: --filter='{"id":["6bd0768b-0beb-4b30-9916-a3c445fede1c"],"marker":0,"limit":10}'
        #[arg(long, value_parser = json_parser)]
        filter: Option<Value>,
        /// Fields to be displayed, example: --field=name,id
        #[arg(short, long, value_delimiter = ',')]
        field: Option<Vec<String>>,
    },
    /// Set operation for a resource
    Oper {
        /// ID or Name of resource to be updated
        #[arg(short, long, required = true)]
        name: String,
        /// Resource attributes, add `'` for list or dict.
        /// Example: -a binding:vif_details='{"port_filter":true}'
        #[arg(short, long, value_parser = key_val_parser)]
        attr: Option<Vec<Value>>,
        /// Fields to be displayed, example: --field=name,id
        #[arg(short, long, value_delimiter = ',')]
        field: Option<Vec<String>>,
        /// Operation string for resource. Please refer to Rest API doc
        #[arg(long)]
        cmd: String,
    },
}

impl self::Operations {
    pub fn oper(&self) -> String {
        match self {
            Operations::Show { .. } => String::from("READ"),
            Operations::List { .. } => String::from("READALL"),
            Operations::Create { .. } => String::from("CREATE"),
            Operations::Update { .. } => String::from("UPDATE"),
            Operations::Delete { .. } => String::from("DELETE"),
            Operations::Oper { cmd, .. } => cmd.to_uppercase(),
        }
    }
}

fn key_val_parser(s: &str) -> Result<Value, String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;

    let json = format!(
        "{l}\"{key}\": {value}{r}",
        l = '{',
        r = '}',
        key = &s[..pos],
        value = &s[pos + 1..]
    );
    match json_parser(&json) {
        Ok(v) => Ok(v),
        Err(e) => {
            if s[pos + 1..].starts_with('[') || s[pos + 1..].starts_with('{') {
                Err(e)
            } else {
                Ok(serde_json::json!({
                    &s[..pos]: &s[pos + 1..],
                }))
            }
        }
    }
}

fn json_parser(s: &str) -> Result<Value, String> {
    match serde_json::from_str(s) {
        Ok(v) => Ok(v),
        Err(e) => {
            let usage = "Don't miss `\"` if it contains String. \
                         You may need `'` for whole list / dict, \
                         or `\\` for `\"` inside list / dict";
            Err(format!("Cannot parse to json - {e:?}\n{usage}"))
        }
    }
}

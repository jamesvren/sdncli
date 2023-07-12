use clap::{
    Parser,
    Args,
    Subcommand,
    ValueEnum,
    builder::{
        IntoResettable, OsStr,
        Resettable::{self, *},
    }
};
use std::{
    path::PathBuf,
};
use serde_json::Value;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Text,
}

impl IntoResettable<OsStr> for OutputFormat {
    fn into_resettable(self) -> Resettable<OsStr> {
        match self {
            OutputFormat::Table => Value(OsStr::from("table")),
            OutputFormat::Json  => Value(OsStr::from("json")),
            OutputFormat::Text  => Value(OsStr::from("text")),
        }
    }
}

impl ToString for OutputFormat {
    fn to_string(&self) -> String {
        match self {
            OutputFormat::Table => String::from("table"),
            OutputFormat::Json  => String::from("json"),
            OutputFormat::Text  => String::from("text"),
        }
    }
}

#[derive(Parser)]
#[command(author = "James R. <jamesvren@163.com>",
          version,
          about = "A command line to manipulate SDN resources (Introspect tools inside).",
          long_about = None)]
#[command(help_template(
    "\
{before-help}{name}({version}){tab}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}\
"
))]
#[command(arg_required_else_help = true)]
pub struct Arg {
    /// Read JSON from file. Please refer to `resource.json`
    #[arg(short, long, group = "input")]
    pub file: Option<PathBuf>,

    /// Request data
    #[arg(short, long, group = "input")]
    pub data: Option<String>,

    /// Request URI
    #[arg(short, long)]
    pub uri: Option<String>,

    ///// Print debug message
    //#[arg(long, action=ArgAction::SetFalse)]
    //pub debug: bool,

    #[arg(
        short,
        long,
        // default_value = OutputFormat::Table,
        value_enum,
        help = "Output format for response",
    )]
    pub output: Option<OutputFormat>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Display detail for some resource(s)
    Show {
        /// Resource type
        resource: String,
        /// ID or Name of resource(s) to be displayed, example: --names=james,james1
        #[arg(value_delimiter = ',', required = true)]
        names: Vec<String>,
        /// Fields to be displayed, example: --field=name,id
        #[arg(short, long, value_delimiter = ',')]
        field: Option<Vec<String>>,
    },
    /// Display all resources
    List {
        /// Resource command, the same with `cmd` in config.toml
        resource: String,
        /// JSON format filter, example: --filter='{"id":["6bd0768b-0beb-4b30-9916-a3c445fede1c"],"marker":0,"limit":10}'
        #[arg(long, value_parser = json_parser)]
        filter: Option<Value>,
        /// Fields to be displayed, example: --field=name,id
        #[arg(short, long, value_delimiter = ',')]
        field: Option<Vec<String>>,
    },
    /// Create a resouce
    Create {
        /// Resource command, the same with `cmd` in config.toml
        resource: String,
        /// Name of resource to be created
        name: String,
        /// Resource attributes, add `'` for list or dict.
        /// Example: -a binding:vif_details='{"port_filter":true}'
        #[arg(short, long, value_parser = key_val_parser)]
        attr: Option<Vec<Value>>,

        #[command(subcommand)]
        lb: Option<LoadBalancer>,
    },
    /// Update a resource with some attributes
    Update {
        /// Resource command, the same with `cmd` in config.toml
        resource: String,
        /// ID or Name of resource(s) to be updated, example: --names=james,james1
        #[arg(value_delimiter = ',', required = true)]
        names: Vec<String>,
        /// Resource attributes, add `'` for list or dict.
        /// Example: -a binding:vif_details='{"port_filter":true}'
        #[arg(short, long, value_parser = key_val_parser)]
        attr: Option<Vec<Value>>,
    },
    /// Delete resources
    Delete {
        /// Resource command, the same with `cmd` in config.toml
        resource: String,
        /// ID or Name of resource(s) to be deleted, example: --names=james,james1
        #[arg(value_delimiter = ',', required = true)]
        names: Vec<String>,
        /// Fields to be displayed, example: --field=name,id
        #[arg(short, long, value_delimiter = ',')]
        field: Option<Vec<String>>,
    },
    /// Set operation for a resource
    Oper {
        /// Resource command, the same with `cmd` in config.toml
        resource: String,
        /// ID or Name of resource to be updated
        #[arg(short, long, required = true)]
        name: String,
        /// Resource attributes, add `'` for list or dict.
        /// Example: -a binding:vif_details='{"port_filter":true}'
        #[arg(short, long, value_parser = key_val_parser)]
        attr: Option<Vec<Value>>,
        /// Operation string for resource. Please refer to Rest API doc
        #[arg(long)]
        cmd: String,
    },
    /// Get auth token
    Token,
    /// Inspect service for each node
    Inspect(Inspect),
}

#[derive(Subcommand)]
pub enum LoadBalancer {
    Lb {
        /// Subnet of VIP
        #[arg(short, long)]
        subnet: String,
        /// VIP of LoadBalancer
        #[arg(short, long)]
        vip: String,
        /// Max connections of LoadBalancer
        #[arg(short, long)]
        connections: Option<u32>,
        /// Thread number of LoadBalancer
        #[arg(short, long)]
        threads: Option<u8>,
        
        /// Protocol port
        #[arg(short, long)]
        port: u16,
        /// Protocol (UDP, TCP, HTTP, TERMINATED_HTTPS)
        #[arg(short='P', long)]
        protocol: String,
    },
    Backend {
        /// ID or Name of LoadBalancer Pool
        #[arg(short='P', long)]
        pool: String,
        /// Subnet of backend
        #[arg(short, long)]
        subnet: String,
        /// Protocol port
        #[arg(short, long)]
        port: u16,
        /// Backend IP
        #[arg(short, long)]
        ip: String,
    },
}

impl crate::arg::Commands {
    pub fn oper(&self) -> String {
        match self {
            Commands::Show {..}   => String::from("READ"),
            Commands::List {..}   => String::from("READALL"),
            Commands::Create {..} => String::from("CREATE"),
            Commands::Update {..} => String::from("UPDATE"),
            Commands::Delete {..} => String::from("DELETE"),
            Commands::Oper {resource: _, name: _, attr: _, cmd} => {
                cmd.to_uppercase()
            },
            _ => String::from("INVALID"),
        }
    }
}

fn key_val_parser(s: &str) -> Result<Value, String> {
    let pos = s
        .find('=')
        .ok_or_else(||format!("invalid KEY=value: no `=` found in `{s}`"))?;

    let json = format!("{l}\"{key}\": {value}{r}",
                       l = '{', r = '}',
                       key = &s[..pos], value = &s[pos + 1..]);
    match json_parser(&json) {
        Ok(v) => Ok(v),
        Err(e) => {
            if s[pos + 1..].starts_with('[') ||
               s[pos + 1..].starts_with('{') {
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
        },
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl LogLevel {
    pub fn to_syslog(&self) -> &str {
        match self {
            LogLevel::Error => "SYS_ERROR",
            LogLevel::Warn  => "SYS_WARN",
            LogLevel::Info  => "SYS_INFO",
            LogLevel::Debug => "SYS_DEBUG",
        }
    }
}

impl IntoResettable<OsStr> for LogLevel {
    fn into_resettable(self) -> Resettable<OsStr> {
        match self {
            LogLevel::Error => Value(OsStr::from("error")),
            LogLevel::Warn  => Value(OsStr::from("warn")),
            LogLevel::Info  => Value(OsStr::from("info")),
            LogLevel::Debug => Value(OsStr::from("debug")),
        }
    }
}

impl ToString for LogLevel {
    fn to_string(&self) -> String {
        match self {
            LogLevel::Error => String::from("error"),
            LogLevel::Warn  => String::from("warn"),
            LogLevel::Info  => String::from("info"),
            LogLevel::Debug => String::from("debug"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Service {
    Svc,
    Config,
    Control,
    Vrouter,
}

impl Service {
    pub fn to_port(&self) -> u32 {
        match self {
            Service::Svc     => 9088,
            Service::Vrouter => 8085,
            Service::Config  => 9084,
            Service::Control => 9083,
        }
    }
}

impl IntoResettable<OsStr> for Service {
    fn into_resettable(self) -> Resettable<OsStr> {
        match self {
            Service::Svc     => Value(OsStr::from("svc")),
            Service::Config  => Value(OsStr::from("config")),
            Service::Control => Value(OsStr::from("control")),
            Service::Vrouter => Value(OsStr::from("vrouter")),
        }
    }
}

impl ToString for Service {
    fn to_string(&self) -> String {
        match self {
            Service::Svc     => String::from("svc"),
            Service::Config  => String::from("config"),
            Service::Control => String::from("control"),
            Service::Vrouter => String::from("vrouter"),
        }
    }
}

#[derive(Args)]
pub struct Inspect {
    /// IP address of host
    #[arg(short, long)]
    pub ip: String,
    /// Service want to be inspected
    #[arg(value_enum)]
    pub service: Service,
    /// Log level
    #[arg(short, long, value_enum)]
    pub log: Option<LogLevel>,
}


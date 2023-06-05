use clap::{
    Parser,
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
        default_value = OutputFormat::Table,
        value_enum,
        help = "Output format for response",
    )]
    pub output: OutputFormat,

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
        #[arg(short, long, value_delimiter = ',', required = true)]
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
        #[arg(short, long, required = true)]
        name: String,
        /// JSON format attributes, example: --attr='{"id":["6bd0768b-0beb-4b30-9916-a3c445fede1c"]}'
        #[arg(short, long, value_parser = json_parser)]
        attr: Option<Value>,
    },
    /// Update a resource with some attributes
    Update {
        /// Resource command, the same with `cmd` in config.toml
        resource: String,
        /// ID or Name of resource to be updated
        #[arg(short, long, required = true)]
        name: String,
        /// JSON format attributes, example: --attr='{"id":["6bd0768b-0beb-4b30-9916-a3c445fede1c"]}'
        #[arg(short, long, value_parser = json_parser)]
        attr: Option<Value>,
    },
    /// Delete resources
    Delete {
        /// Resource command, the same with `cmd` in config.toml
        resource: String,
        /// ID or Name of resource(s) to be deleted, example: --names=james,james1
        #[arg(short, long, value_delimiter = ',', required = true)]
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
        /// JSON format attributes, example: --attr='{"id":["6bd0768b-0beb-4b30-9916-a3c445fede1c"]}'
        #[arg(short, long, value_parser = json_parser)]
        attr: Option<Value>,
        /// Operation string for resource. Please refer to Rest API doc
        #[arg(long)]
        cmd: String,
    },
    /// Get auth token
    Token,
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

fn json_parser(s: &str) -> Result<Value, String> {
    match serde_json::from_str(s) {
        Ok(v) => Ok(v),
        Err(e) => Err(format!("Cannot parse to json - {:?}", e)),
    }
}

#[derive(Subcommand)]
enum Introspect {
}


use super::inspect::Introspect;
use clap::{
    builder::{
        IntoResettable, OsStr,
        Resettable::{self, *},
    },
    ArgMatches, Args, Command, FromArgMatches, ValueEnum,
};

#[derive(Args)]
pub struct Opts {
    /// IP address of host
    #[arg(required = true)]
    pub ip: String,

    /// Service want to be inspected
    #[arg(value_enum)]
    pub service: Service,

    /// Log level
    #[arg(short, long, value_enum)]
    pub log: Option<LogLevel>,
}

pub fn build_cli(cmd: Command) -> Command {
    let cli = Command::new("inspect").about("Inspect service for each node");
    let cli = Opts::augment_args(cli);
    cmd.subcommand(cli)
}

pub async fn handle_cli(matches: &ArgMatches) -> Result<(), anyhow::Error> {
    if let Some(matches) = matches.subcommand_matches("inspect") {
        let cmd = Opts::from_arg_matches(matches)
            .map_err(|err| err.exit())
            .unwrap();
        let ist = Introspect::new(&cmd.ip, cmd.service.to_port(), cmd.service.to_string());
        if let Some(level) = cmd.log {
            ist.set_logging(level.to_syslog()).await?;
        } else {
            ist.get("Snh_SandeshLoggingParamsSet?enable=&category=&log_level=&trace_print=&enable_flow_log=").await?;
            //ist.get("Snh_SandeshUVECacheReq?tname=NodeStatus").await?;
        }
    }

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl LogLevel {
    pub fn to_syslog(self) -> &'static str {
        match self {
            LogLevel::Error => "SYS_ERROR",
            LogLevel::Warn => "SYS_WARN",
            LogLevel::Info => "SYS_INFO",
            LogLevel::Debug => "SYS_DEBUG",
        }
    }
}

impl IntoResettable<OsStr> for LogLevel {
    fn into_resettable(self) -> Resettable<OsStr> {
        match self {
            LogLevel::Error => Value(OsStr::from("error")),
            LogLevel::Warn => Value(OsStr::from("warn")),
            LogLevel::Info => Value(OsStr::from("info")),
            LogLevel::Debug => Value(OsStr::from("debug")),
        }
    }
}

impl ToString for LogLevel {
    fn to_string(&self) -> String {
        match self {
            LogLevel::Error => String::from("error"),
            LogLevel::Warn => String::from("warn"),
            LogLevel::Info => String::from("info"),
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
    pub fn to_port(self) -> u32 {
        match self {
            Service::Svc => 9088,
            Service::Vrouter => 8085,
            Service::Config => 9084,
            Service::Control => 9083,
        }
    }
}

impl IntoResettable<OsStr> for Service {
    fn into_resettable(self) -> Resettable<OsStr> {
        match self {
            Service::Svc => Value(OsStr::from("svc")),
            Service::Config => Value(OsStr::from("config")),
            Service::Control => Value(OsStr::from("control")),
            Service::Vrouter => Value(OsStr::from("vrouter")),
        }
    }
}

impl ToString for Service {
    fn to_string(&self) -> String {
        match self {
            Service::Svc => String::from("svc"),
            Service::Config => String::from("config"),
            Service::Control => String::from("control"),
            Service::Vrouter => String::from("vrouter"),
        }
    }
}

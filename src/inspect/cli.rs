use super::inspect::Introspect;
use clap::{ArgMatches, Args, Command, FromArgMatches, Subcommand, ValueEnum};

#[derive(Args)]
struct Opts {
    /// IP address of host
    #[arg(required = true)]
    ip: String,

    /// Service want to be inspected
    #[command(subcommand)]
    service: Service,
}

#[derive(Subcommand)]
enum Common {
    /// Log level
    Log {
        #[arg(value_enum)]
        level: Option<LogLevel>,
    },

    /// Sandesh Trace
    Trace { buffer: Option<String> },

    /// Url to query
    Url { uri: Option<String> },
}

#[derive(Subcommand)]
enum Service {
    /// config-svc-monitor
    Svc {
        #[command(subcommand)]
        common: Common,
    },
    /// config-schema
    Schema {
        #[command(subcommand)]
        common: Common,
    },
    /// config-api
    Config {
        #[command(subcommand)]
        common: Common,
    },
    /// control
    Control {
        #[command(subcommand)]
        common: Common,
    },
    /// collector
    Collector {
        #[command(subcommand)]
        common: Common,
    },
    /// query-engine
    Qe {
        #[command(subcommand)]
        common: Common,
    },
    /// vrouter-agent
    Vrouter {
        #[command(subcommand)]
        common: Common,
    },
    /// config-nodemgr
    CfgNodeMgr {
        #[command(subcommand)]
        common: Common,
    },
    /// control-nodemgr
    CtrlNodeMgr {
        #[command(subcommand)]
        common: Common,
    },
    /// vrouter-nodemgr
    VrNodeMgr {
        #[command(subcommand)]
        common: Common,
    },
    /// database-nodemgr
    DbNodeMgr {
        #[command(subcommand)]
        common: Common,
    },
    /// snmp-collector
    Snmp {
        #[command(subcommand)]
        common: Common,
    },
    /// snmp-topology
    Topology {
        #[command(subcommand)]
        common: Common,
    },
}

impl Service {
    fn to_port(&self) -> u32 {
        match self {
            Service::Svc { .. } => 9088,
            Service::Schema { .. } => 8087,
            Service::Config { .. } => 9084,
            Service::Control { .. } => 9083,
            Service::Collector { .. } => 8089,
            Service::Qe { .. } => 7091,
            Service::Vrouter { .. } => 8085,
            Service::CfgNodeMgr { .. } => 8100,
            Service::CtrlNodeMgr { .. } => 8101,
            Service::VrNodeMgr { .. } => 8102,
            Service::DbNodeMgr { .. } => 8103,
            Service::Snmp { .. } => 6920,
            Service::Topology { .. } => 6921,
        }
    }
    fn get_common(&self) -> &Common {
        match self {
            Service::Svc { common, .. } => common,
            Service::Schema { common, .. } => common,
            Service::Config { common, .. } => common,
            Service::Control { common, .. } => common,
            Service::Collector { common, .. } => common,
            Service::Qe { common, .. } => common,
            Service::Vrouter { common, .. } => common,
            Service::CfgNodeMgr { common, .. } => common,
            Service::CtrlNodeMgr { common, .. } => common,
            Service::VrNodeMgr { common, .. } => common,
            Service::DbNodeMgr { common, .. } => common,
            Service::Snmp { common, .. } => common,
            Service::Topology { common, .. } => common,
        }
    }
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
        let ist = Introspect::new(&cmd.ip, cmd.service.to_port());
        match cmd.service.get_common() {
            Common::Log { level } => match level {
                Some(level) => ist.set_logging(level.to_syslog()).await?,
                None => ist.get_logging().await?,
            },
            Common::Trace { buffer } => {
                ist.get_trace(buffer).await?;
            }
            Common::Url { uri } => match uri {
                Some(uri) => {
                    if uri.ends_with(".xml") {
                        ist.get(uri).await?
                    } else {
                        ist.get(&format!("Snh_{uri}")).await?
                    }
                }
                None => ist.get("/").await?,
            },
        }
    }

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Notice,
}

impl LogLevel {
    fn to_syslog(self) -> &'static str {
        match self {
            LogLevel::Error => "SYS_ERR",
            LogLevel::Warn => "SYS_WARN",
            LogLevel::Info => "SYS_INFO",
            LogLevel::Debug => "SYS_DEBUG",
            LogLevel::Notice => "SYS_NOTICE",
        }
    }
}

use super::inspect::Introspect;
use clap::{
    ArgMatches,
    Args,
    Command,
    FromArgMatches,
    Subcommand,
    ValueEnum,
};

#[derive(Args)]
struct Opts {
    /// IP address of host
    #[arg(required = true)]
    ip: String,

    /// Introspect port number
    #[arg(short, long)]
    port: Option<u32>,

    /// Service want to be inspected
    #[command(subcommand)]
    service: Service,
}

#[derive(Args)]
#[command(subcommand_negates_reqs = true)]
struct VrouterCommand {
    #[command(subcommand)]
    common: Option<Common>,

    /// List all flow keys
    #[arg(short, long, group = "flow", required = true)]
    keys: Option<bool>,

    /// Look up a flow item with a key. Drop reason can be patched.
    #[arg(short, long, group = "flow", required = true)]
    entry: Option<String>,
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

    /// Sandesh UVE
    Uve { uve: Option<String> },
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
    /// analysis-api
    Analysis {
        #[command(subcommand)]
        common: Common,
    },
    /// query-engine
    Qe {
        #[command(subcommand)]
        common: Common,
    },
    /// vrouter-agent
    Vrouter(VrouterCommand),

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
    /// Dns
    Dns {
        #[command(subcommand)]
        common: Common,
    },
    /// Dns
    Dm {
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
            Service::Analysis { .. } => 8090,
            Service::Qe { .. } => 7091,
            Service::Vrouter { .. } => 8085,
            Service::CfgNodeMgr { .. } => 8100,
            Service::CtrlNodeMgr { .. } => 8101,
            Service::VrNodeMgr { .. } => 8102,
            Service::DbNodeMgr { .. } => 8103,
            Service::Snmp { .. } => 6920,
            Service::Topology { .. } => 6921,
            Service::Dns { .. } => 8092,
            Service::Dm { .. } => 8096,
        }
    }
    fn get_common(&self) -> Option<&Common> {
        match self {
            Service::Svc { common, .. } => Some(common),
            Service::Schema { common, .. } => Some(common),
            Service::Config { common, .. } => Some(common),
            Service::Control { common, .. } => Some(common),
            Service::Collector { common, .. } => Some(common),
            Service::Analysis { common, .. } => Some(common),
            Service::Qe { common, .. } => Some(common),
            Service::CfgNodeMgr { common, .. } => Some(common),
            Service::CtrlNodeMgr { common, .. } => Some(common),
            Service::VrNodeMgr { common, .. } => Some(common),
            Service::DbNodeMgr { common, .. } => Some(common),
            Service::Snmp { common, .. } => Some(common),
            Service::Topology { common, .. } => Some(common),
            Service::Dns { common, .. } => Some(common),
            Service::Dm { common, .. } => Some(common),
            Service::Vrouter(VrouterCommand { common, .. }) => common.as_ref(),
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
        let port = match cmd.port {
            Some(port) => port,
            None => cmd.service.to_port(),
        };
        let ist = Introspect::new(&cmd.ip, port);
        let common_cmd = cmd.service.get_common();
        if common_cmd.is_some() {
            match common_cmd.unwrap() {
                Common::Trace { buffer } => {
                    ist.get_trace(buffer).await?;
                }
                Common::Uve { uve } => {
                    ist.get_uve(uve).await?;
                }
                Common::Log { level } => match level {
                    Some(level) => ist.set_logging(level.to_syslog()).await?,
                    None => ist.get_logging().await?,
                },
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
        } else {
            match cmd.service {
                Service::Vrouter(vr_cmd) => {
                    if let Some(key) = vr_cmd.keys {
                        if key {
                            ist.get(&format!("Snh_Inet4FlowTreeReq")).await?;
                        }
                    } else if let Some(entry) = vr_cmd.entry {
                        ist.get(&format!("Snh_FlowsPerInetRouteFlowMgmtKeyReq?x={entry}")).await?;
                    }
                },
                _ => unimplemented!()
            }
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

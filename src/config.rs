use std::{
    fs,
    io,
    env,
    path::PathBuf,
};
use anyhow::{anyhow, Error};
use toml;
use serde::Deserialize;
use json5;
use serde_json::Value;
use log::debug;

#[derive(Deserialize)]
pub struct FileConfig {
    pub method: String,
    pub port: Option<u32>,
    #[serde(rename="api")]
    pub uri: String,
    pub body: Value,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub api: Rest,
    pub auth: Auth,
    pub resource: Vec<Resource>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rest {
    pub host: Option<String>,
    pub port: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Auth {
    pub host: String,
    pub port: u32,
    pub user: String,
    pub password: String,
    pub project: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct Resource {
    pub cmd: String,
    #[serde(rename="type")]
    pub resource: String,
    pub uri: String,
    pub attr: Option<toml::Value>,
}

pub fn read_config() -> Result<Config, Error> {
    match std::fs::read_to_string(get_name(FileType::Toml).unwrap()) {
        Ok(content) => {
            let config: Config = toml::from_str(&content)?;
            debug!("{:#?}", config);
            Ok(config)
        },
        Err(_) => {
            gen_config()?;
            println!("{}\n{}\n{}",
                     "Config file `config.toml` generated in same directory as the executable.",
                     "- Please set correct AUTH information.",
                     "- Add resource information to build command.");
            Err(anyhow!("Please re-run this after `EDIT` config.toml."))
        },
    }
}

pub fn read_json_file(file: &PathBuf) -> Result<FileConfig, Error> {
    let data = fs::read_to_string(file)?;
    match json5::from_str::<FileConfig>(&data) {
        Ok(config) => Ok(config),
        Err(e) => Err(anyhow!("Failed to parse JSON file - {}", e)),
    }
}

impl Config {
    pub fn get_resource(&self, cmd: &str) -> anyhow::Result<&Resource> {
        let cmds: Vec<_> = self.resource.iter().filter(|c| c.cmd == cmd).collect();
        if !cmds.is_empty() {
            return Ok(cmds[0]);
        }
        Err(anyhow!("No reourcce found for command {}, please check config.toml", cmd))
    }
}

fn gen_config() -> io::Result<()> {
    let config_toml = toml::toml! {
        [auth]
        host = "10.130.151.80"
        port = 6000
        user = "ArcherAdmin"
        password = "ArcherAdmin@123"
        project = "ArcherAdmin"
        version = "v2"

        [api]
        port = 8082

        [[resource]]
        cmd = "network"
        type = "network"
        uri = "/neutron/network"
        attr = [
            { key="provider:segmentation_id", value=0 },
            { key="router:external", value=true },
            { key="provider:network_type", value="" },
            { key="subnets", value=[""] },
        ]

        [[resource]]
        cmd = "subnet"
        type = "subnet"
        uri = "/neutron/subnet"

        [[resource]]
        cmd = "port"
        type = "port"
        uri = "/neutron/port"

        [[resource]]
        cmd = "router"
        type = "router"
        uri = "/neutron/router"

        [[resource]]
        cmd = "sg"
        type = "security_group"
        uri = "/neutron/security_group"

        [[resource]]
        cmd = "sgr"
        type = "security_group_rule"
        uri = "/neutron/security_group_rule"

        [[resource]]
        cmd = "fip"
        type = "floatingip"
        uri = "/neutron/floatingip"

        [[resource]]
        cmd = "lb"
        type = "loadbalancer"
        uri = "/neutron/loadbalancer"
        attr = [
            { key="vip_subnet_id", value="" },
            { key="vcpus", value=0 },
            { key="ram", value=0 },
        ]

        [[resource]]
        cmd = "lbl"
        type = "listener"
        uri = "/neutron/listener"

        [[resource]]
        cmd = "lbp"
        type = "pool"
        uri = "/neutron/pool"

        [[resource]]
        cmd = "lbm"
        type = "member"
        uri = "/neutron/pool/<pool_id>/member"

        [[resource]]
        cmd = "fw"
        type = "firewall_group"
        uri = "/neutron/firewall_group"

        [[resource]]
        cmd = "fwp"
        type = "firewall_policy"
        uri = "/neutron/firewall_policy"

        [[resource]]
        cmd = "fwr"
        type = "firewall_rule"
        uri = "/neutron/firewall_rule"

        [[resource]]
        cmd = "sfw"
        type = "segment_firewall_group"
        uri = "/neutron/segment_firewall_group"

        [[resource]]
        cmd = "sfwp"
        type = "segment_firewall_policy"
        uri = "/neutron/segment_firewall_policy"

        [[resource]]
        cmd = "sfwr"
        type = "segment_firewall_rule"
        uri = "/neutron/segment_firewall_rule"

        [[tag]]
        cmd = "tag"
        type = "tag"
        uri = "/neutron/tag"

        [[tag]]
        cmd = "provider"
        type = "net_provider"
        uri = "/neutron/net_provider"
    };

    let json5_str = r#"{
//"port": 8081,
//"method": "get",
"method": "post",
"api": "/neutron/network",
//"api": "/analytics/uves/nicstats/*?cfilt=UveNicStats:status",
//"api": "/analytics/uves/vrouter/*?cfilt=NodeStatus:process_status",
//"api": "/analytics/switch_topology",
"body":
{
    "data": {
        "fields": [],
        "filters": {},
        "resource": {
            "router:external": false,
            "name": "example",
            "provider:physical_network": "self",
            "admin_state_up": true,
            "tenant_id": "ad88dd5d24ce4e2189a6ae7491c33e9d",
            "provider:network_type": "vxlan",
            "shared": false,
            "port_security_enabled": true,
            "provider:segmentation_id": 666666
        },
    },
    "context": {
        "user_id": "44faef681cd34e1c80b8520dd6aebad4",
        "tenant_id": "ad88dd5d24ce4e2189a6ae7491c33e9d",
        "is_admin": true,
        "request_id": "req-b52fae02-899c-4dd5-814c-c1f67bcbf40f",
        "operation": "CREATE",
        "type": "network"
    }
}

}
"#;
    fs::write(get_name(FileType::Toml)?, toml::to_string(&config_toml).unwrap())?;
    fs::write(get_name(FileType::Json)?, json5_str)?;
    Ok(())
}

enum FileType {
    Json,
    Toml,
}

fn get_name(ftype: FileType) -> io::Result<PathBuf> {
    let mut path = env::current_exe()?;
    match ftype {
        FileType::Toml => path.set_file_name("config.toml"),
        FileType::Json => path.set_file_name("resource.json"),
    }
    Ok(path)
}


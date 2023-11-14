use anyhow;
use clap::{ArgMatches, Args, Command, FromArgMatches};
use scylla::{transport::Compression, IntoTypedRows, Session, SessionBuilder};
use serde_json::Value;
use std::{collections::HashSet, time::Duration, time::Instant};

use super::cql::cqlsh;

#[derive(Args)]
struct Opts {
    /// DB hosts to be connected
    #[arg(required = true, requires = "arg")]
    hosts: Vec<String>,

    /// DB port to be connected
    #[arg(short, long, default_value_t = 9041)]
    port: u16,

    /// Show resouce's properties
    #[arg(short, long, group = "arg")]
    uuid: Option<String>,

    /// Show all uuids
    #[arg(long, group = "arg")]
    uuids: bool,

    /// Show resource's fqname
    #[arg(short, long, name = "type", group = "arg")]
    fqname: Option<String>,

    /// Show all fqnames
    #[arg(long, group = "arg")]
    fqnames: bool,

    /// Simulate a port list
    #[arg(long, group = "arg")]
    portlist: bool,

    /// Enter CQL shell
    #[arg(long, group = "arg")]
    cqlsh: bool,
}

pub fn build_cli(cmd: Command) -> Command {
    let cli = Command::new("db").about("Manipulate SDN database");
    let cli = Opts::augment_args(cli);
    cmd.subcommand(cli)
}

pub async fn handle_cli(matches: &ArgMatches) -> Result<(), anyhow::Error> {
    if let Some(matches) = matches.subcommand_matches("db") {
        let cmd = Opts::from_arg_matches(matches)
            .map_err(|err| err.exit())
            .unwrap();
        let nodes: Vec<_> = cmd
            .hosts
            .iter()
            .map(|item| format!("{item}:{0}", cmd.port))
            .collect();
        println!("** Connecting to {nodes:?} ...");

        let session: Session = SessionBuilder::new()
            .known_nodes(nodes)
            .user("sdn", "sdncassandra")
            .connection_timeout(Duration::from_secs(3))
            .cluster_metadata_refresh_interval(Duration::from_secs(10))
            .compression(Some(Compression::Lz4))
            .build()
            .await?;

        println!("** DB connected, ready to send query ...");

        if let Some(uuid) = cmd.uuid {
            let q: &str = r#"SELECT blobAsText(column1), value, WRITETIME(value)
                             FROM config_db_uuid.obj_uuid_table
                             WHERE key = textAsBlob(?)"#;
            query(&session, q, &uuid).await?;
        }

        if let Some(fqname) = cmd.fqname {
            let q: &str = r#"SELECT blobAsText(column1), value, WRITETIME(value)
                             FROM config_db_uuid.obj_fq_name_table
                             WHERE key = textAsBlob(?)"#;
            query(&session, q, &fqname).await?;
        }

        if cmd.uuids {
            let q: &str = r#"SELECT blobastext(key)
                             FROM config_db_uuid.obj_uuid_table"#;
            query_key(&session, q).await?;
        }

        if cmd.fqnames {
            let q: &str = r#"SELECT blobastext(key)
                             FROM config_db_uuid.obj_fq_name_table"#;
            query_key(&session, q).await?;
        }

        if cmd.portlist {
            let now = Instant::now();
            simulate_portlist(&session).await?;
            println!("Time: {}", now.elapsed().as_secs_f32());
        }

        if cmd.cqlsh {
            cqlsh(&session).await?;
        }
    }

    Ok(())
}

async fn query(session: &Session, q: &str, arg: &str) -> Result<(), anyhow::Error> {
    if let Some(rows) = session.query(q, (arg,)).await?.rows {
        for row in rows.into_typed::<(String, String, i64)>() {
            let (prop, value, timestamp) = row?;
            println!(
                "{} | {} | {}",
                prop,
                serde_json::from_str::<Value>(&value)?,
                timestamp
            );
        }
    }
    Ok(())
}

async fn query_key(session: &Session, q: &str) -> Result<(), anyhow::Error> {
    if let Some(rows) = session.query(q, &[]).await?.rows {
        let mut uuids: HashSet<String> = HashSet::new();
        for row in rows.into_typed::<(String,)>() {
            let (uuid,) = row?;
            uuids.insert(uuid);
        }
        for u in &uuids {
            println!("{u}");
        }
        println!("Total: {}", uuids.len());
    }
    Ok(())
}

async fn simulate_portlist(session: &Session) -> Result<(), anyhow::Error> {
    let fqname: &str = r#"SELECT blobAsText(column1), value
                          FROM config_db_uuid.obj_fq_name_table
                          WHERE key = textAsBlob(?)"#;
    let mut uuids: HashSet<String> = HashSet::new();
    // get vmi uuids from fqname
    println!("*** select virtual_machine_interface from fqnme table:");
    if let Some(rows) = session
        .query(fqname, ("virtual_machine_interface",))
        .await?
        .rows
    {
        for row in rows.into_typed::<(String, String)>() {
            let (fq, _) = row?;
            let fq: Vec<_> = fq.split(':').collect();
            uuids.insert(fq[3].to_string());
        }
    }
    for u in &uuids {
        println!("{u}");
    }
    println!("Total: {}", uuids.len());

    // get timestamp for all uuids
    let timestamp: &str = r#"SELECT blobAsText(column1), value, WRITETIME(value)
                             FROM config_db_uuid.obj_uuid_table
                             WHERE key = textAsBlob(?) AND column1 IN (textAsBlob(?)) ALLOW FILTERING"#;
    println!("*** select timestamp from uuid table:");
    for u in &uuids {
        if let Some(rows) = session
            .query(timestamp, (u, "META:latest_col_ts"))
            .await?
            .rows
        {
            for row in rows.into_typed::<(String, String, i64)>() {
                let (p, v, t) = row?;
                println!("{t} | {p} {v}");
            }
        }
    }

    // get vn uuids from fqname
    let mut vn_uuids: HashSet<String> = HashSet::new();
    println!("*** select virtual_network from fqnme table:");
    if let Some(rows) = session.query(fqname, ("virtual_network",)).await?.rows {
        for row in rows.into_typed::<(String, String)>() {
            let (fq, _) = row?;
            let fq: Vec<_> = fq.split(':').collect();
            vn_uuids.insert(fq[3].to_string());
        }
    }

    for u in &vn_uuids {
        println!("{u}");
    }
    println!("Total: {}", vn_uuids.len());

    // get vn from share table
    println!("*** select project from share table:");
    let share: &str = r#"SELECT blobAsText(column1), value
                         FROM config_db_uuid.obj_shared_table
                         WHERE key = textAsBlob(?) AND column1 >= textAsBlob(?) AND column1 <= textAsBlob(?) "#;
    if let Some(rows) = session
        .query(
            share,
            (
                "virtual_network",
                "tenant:e82dedfa-ad3f-4616-9cd3-73baf2bdb4ee:",
                "tenant:e82dedfa-ad3f-4616-9cd3-73baf2bdb4ee;",
            ),
        )
        .await?
        .rows
    {
        for row in rows.into_typed::<(String, String)>() {
            let (p, v) = row?;
            println!("{p} | {v}");
        }
    }
    println!("*** select domain from share table:");
    if let Some(rows) = session
        .query(
            share,
            (
                "virtual_network",
                "domain:e22e4f92-9e1f-4fe5-97f0-6833a4da78ee:",
                "domain:e22e4f92-9e1f-4fe5-97f0-6833a4da78ee;",
            ),
        )
        .await?
        .rows
    {
        for row in rows.into_typed::<(String, String)>() {
            let (p, v) = row?;
            println!("{p} | {v}");
        }
    }
    println!("*** select global from share table:");
    if let Some(rows) = session
        .query(share, ("virtual_network", "global::", "global:;"))
        .await?
        .rows
    {
        for row in rows.into_typed::<(String, String)>() {
            let (p, v) = row?;
            println!("{p} | {v}");
        }
    }

    // get instance ip for all uuids
    println!("*** select instance ip from uuid table:");
    let backref: &str = r#"SELECT blobAsText(column1), value, WRITETIME(value)
                          FROM config_db_uuid.obj_uuid_table
                          WHERE key = textAsBlob(?) AND column1 >= textAsBlob(?) AND column1 <= textAsBlob(?)"#;
    for u in &uuids {
        if let Some(rows) = session
            .query(backref, (u, "backref:instance_ip:", "backref:instance_ip;"))
            .await?
            .rows
        {
            for row in rows.into_typed::<(String, String, i64)>() {
                let (p, v, t) = row?;
                println!("{u}: {p} | {v} | {t}");
            }
        }
    }

    Ok(())
}

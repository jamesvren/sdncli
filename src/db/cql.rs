use anyhow::Result;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, Context, Editor};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
use scylla::{QueryResult, Session};

#[derive(Helper, Highlighter, Validator, Hinter)]
struct CqlHelper;

const CQL_KEYWORDS: &[&str] = &[
    "ADD",
    "AGGREGATE",
    "ALL",
    "ALLOW",
    "ALTER",
    "AND",
    "ANY",
    "APPLY",
    "AS",
    "ASC",
    "ASCII",
    "AUTHORIZE",
    "BATCH",
    "BEGIN",
    "BIGINT",
    "BLOB",
    "BOOLEAN",
    "BY",
    "CLUSTERING",
    "COLUMNFAMILY",
    "COMPACT",
    "CONSISTENCY",
    "COUNT",
    "COUNTER",
    "CREATE",
    "CUSTOM",
    "DECIMAL",
    "DELETE",
    "DESC",
    "DISTINCT",
    "DOUBLE",
    "DROP",
    "EACH_QUORUM",
    "ENTRIES",
    "EXISTS",
    "FILTERING",
    "FLOAT",
    "FROM",
    "FROZEN",
    "FULL",
    "GRANT",
    "IF",
    "IN",
    "INDEX",
    "INET",
    "INFINITY",
    "INSERT",
    "INT",
    "INTO",
    "KEY",
    "KEYSPACE",
    "KEYSPACES",
    "LEVEL",
    "LIMIT",
    "LIST",
    "LOCAL_ONE",
    "LOCAL_QUORUM",
    "MAP",
    "MATERIALIZED",
    "MODIFY",
    "NAN",
    "NORECURSIVE",
    "NOSUPERUSER",
    "NOT",
    "OF",
    "ON",
    "ONE",
    "ORDER",
    "PARTITION",
    "PASSWORD",
    "PER",
    "PERMISSION",
    "PERMISSIONS",
    "PRIMARY",
    "QUORUM",
    "RENAME",
    "REVOKE",
    "SCHEMA",
    "SELECT",
    "SET",
    "STATIC",
    "STORAGE",
    "SUPERUSER",
    "TABLE",
    "TEXT",
    "TIME",
    "TIMESTAMP",
    "TIMEUUID",
    "THREE",
    "TO",
    "TOKEN",
    "TRUNCATE",
    "TTL",
    "TUPLE",
    "TWO",
    "TYPE",
    "UNLOGGED",
    "UPDATE",
    "USE",
    "USER",
    "USERS",
    "USING",
    "UUID",
    "VALUES",
    "VARCHAR",
    "VARINT",
    "VIEW",
    "WHERE",
    "WITH",
    "WRITETIME",
    // Scylla-specific
    "BYPASS",
    "CACHE",
    "SERVICE",
    "LEVEL",
    "LEVELS",
    "ATTACH",
    "ATTACHED",
    "DETACH",
    "TIMEOUT",
    "FOR",
    "PER",
    "PARTITION",
    "LIKE",
];

impl Completer for CqlHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        if !line.is_empty() {
            let start: usize = line[..pos].rfind(' ').map_or(0, |p| p + 1);
            let prefix = &line[start..pos].to_uppercase();
            // NOTICE: yes, linear, but still fast enough for a cli
            // TODO:
            //  * completion from schema information
            //  * completion with context - e.g. INTO only comes after INSERT
            //  * completion for internal commands (once implemented)
            if !prefix.is_empty() {
                let mut matches: Vec<Pair> = Vec::new();
                for keyword in CQL_KEYWORDS {
                    if keyword.starts_with(prefix) {
                        matches.push(Pair {
                            display: keyword.to_string(),
                            replacement: format!("{} ", keyword),
                        })
                    }
                }
                if !matches.is_empty() {
                    return Ok((start, matches));
                }
            }
        }
        Ok((0, vec![]))
    }
}

fn print_result(result: &QueryResult) {
    if result.rows.is_none() {
        println!("OK");
        return;
    }

    for row in result.rows.as_ref().unwrap() {
        for column in &row.columns {
            print!(" | ");
            if let Some(t) = column.as_ref().unwrap().as_text() {
                print!("{t}");
            } else if let Some(b) = column.as_ref().unwrap().as_blob() {
                for i in b {
                    print!("{:x}", i);
                }
            } else if let Some(b) = column.as_ref().unwrap().as_bigint() {
                print!("{b}");
            } else {
                print!("{:?}", column);
            }
        }
        println!(" | ")
    }
}

pub async fn cqlsh(session: &Session) -> Result<()> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .build();
    let mut rl = Editor::with_config(config);
    rl.set_helper(Some(CqlHelper {}));
    loop {
        let readline = rl.readline("sdn@cqlsh>> ");
        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }
                rl.add_history_entry(line.as_str());
                let maybe_res = session.query(line, &[]).await;
                match maybe_res {
                    Err(err) => println!("Error: {}", err),
                    Ok(res) => print_result(&res),
                }
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => println!("Error: {}", err),
        }
    }
    Ok(())
}

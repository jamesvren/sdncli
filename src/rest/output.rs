use prettytable::{format, row, Cell, Row, Table};
use serde_json::Value;

pub fn json_to_table(value: &Value, fields: Option<Vec<String>>) {
    let mut table = Table::new();
    let mut len = 0;
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    if value.is_array() {
        let array = value.as_array().unwrap();
        len = array.len();
        match fields {
            Some(fields) => {
                table.set_titles((&fields).into());
                for obj in array {
                    let mut r = Vec::new();
                    if let Some(dict) = obj.as_object() {
                        fields.iter().for_each(|f| {
                            if let Some(v) = dict.get(f) {
                                r.push(Cell::new(&v.to_string()));
                            }
                        });
                    }
                    table.add_row(Row::new(r));
                }
            }
            None => {
                for obj in array {
                    table.add_row(row![Fguc => "KEY", "VALUE"]);
                    if let Some(dict) = obj.as_object() {
                        dict.iter().for_each(|(k, v)| {
                            table.add_row(row![k, v]);
                        });
                    }
                }
            }
        }
    } else if value.is_object() {
        len = 1;
        let dict = value.as_object().unwrap();
        match fields {
            Some(fields) => {
                table.set_titles((&fields).into());
                let mut r = Vec::new();
                fields.iter().for_each(|f| {
                    if let Some(v) = dict.get(f) {
                        r.push(Cell::new(&v.to_string()));
                    }
                });
                table.add_row(Row::new(r));
            }
            None => {
                dict.iter().for_each(|(k, v)| {
                    table.add_row(row![k, v]);
                });
            }
        }
    }

    if len != 0 {
        table.printstd();
        println!("Total: {}", len);
    }
}

pub fn json_output(value: &Value) {
    let mut len = 0;
    if value.is_array() {
        let array = value.as_array().unwrap();
        len = array.len();
    } else if value.is_object() {
        len = 1;
    }
    println!("{:#}", value);
    println!("Total: {}", len);
}

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::*;
use serde_json::Value;

pub fn json_to_table(value: &Value, fields: Option<Vec<String>>) {
    let mut table = Table::new();
    let mut len = 0;
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
    if value.is_array() {
        let array = value.as_array().unwrap();
        len = array.len();
        match fields {
            Some(fields) => {
                table.set_header(&fields);
                for obj in array {
                    let mut r = Vec::new();
                    if let Some(dict) = obj.as_object() {
                        fields.iter().for_each(|f| {
                            if let Some(v) = dict.get(f) {
                                r.push(Cell::new(&v.to_string()));
                            }
                        });
                    }
                    table.add_row(r);
                }
            }
            None => {
                for obj in array {
                    table.add_row(vec![
                        Cell::new("KEY")
                            .fg(Color::Green)
                            .add_attribute(Attribute::Underlined)
                            .set_alignment(CellAlignment::Center),
                        Cell::new("VALUE")
                            .fg(Color::Green)
                            .add_attribute(Attribute::Underlined)
                            .set_alignment(CellAlignment::Center),
                    ]);
                    if let Some(dict) = obj.as_object() {
                        dict.iter().for_each(|(k, v)| {
                            table.add_row(vec![k, &v.to_string()]);
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
                table.set_header(&fields);
                let mut r = Vec::new();
                fields.iter().for_each(|f| {
                    if let Some(v) = dict.get(f) {
                        r.push(Cell::new(&v.to_string()));
                    }
                });
                table.add_row(r);
            }
            None => {
                dict.iter().for_each(|(k, v)| {
                    table.add_row(vec![k, &v.to_string()]);
                });
            }
        }
    }

    if len != 0 {
        println!("{table}");
        println!("Total: {}", len);
    }
}

pub fn json_output(value: &Value, _fields: Option<Vec<String>>) {
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

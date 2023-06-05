use serde_json::{json, Map, Value};
use serde::Serialize;
use uuid::Uuid;
use anyhow;

#[derive(Serialize)]
pub struct Resource {
    data: Data,
    context: Value,
}

#[derive(Serialize, Default)]
struct Data {
    fields: Vec<String>,
    filters: Value,
    id: Option<Uuid>,
    resource: Map<String, Value>,
}

pub struct ResourceBuilder {
    body: Resource,
}

//impl Default for Data {
//    fn default() -> Self {
//        fields: vec![],
//        filters: HashMap::<&str, &str>::new(),
//        id: None,
//        resource: Value::Map::new(),
//    }
//}

impl ResourceBuilder {
    pub fn new() -> Self {
        Self {
            body: Resource {
                data: Data {
                    filters: Value::Object(Map::new()),
                    ..Data::default()
                },
                context: json!({
                    "is_admin": true,
                    "operation": "READALL",
                    "request_id": format!("req-sdncli-{}", Uuid::new_v4()),
                    "tenant_id": "ad88dd5d24ce4e2189a6ae7491c33e9d",
                    "type": "",
                    "user_id": "44faef681cd34e1c80b8520dd6aebad4",
                }),
            }
        }
    }

    pub fn res_type(&mut self, res: &str) ->&mut Self {
        self.body.context["type"] = res.into();
        self
    }

    pub fn oper(&mut self, oper: &str) -> &mut Self {
        self.body.context["operation"] = oper.into();
        self
    }

    pub fn id(&mut self, id: Uuid) -> &mut Self {
        self.body.data.id = Some(id);
        self.body.data.resource.insert(String::from("id"), json!(id));
        self
    }

    pub fn fields(&mut self, fields: Vec<String>) -> &mut Self {
        self.body.data.fields = fields;
        self
    }

    pub fn filters(&mut self, filters: Value) -> &mut Self {
        self.body.data.filters = filters;
        self
    }

    pub fn name(&mut self, name: &str) -> &mut Self {
        let mut map = Map::new();
        map.insert(String::from("name"), json!(name));
        self.resource(map);
        self
    }

    pub fn resource(&mut self, resource: Map<String, Value>) -> &mut Self {
        //if let Value::Object(ref mut map) = self.body.data.resource {
            for (k, v) in resource {
                //map["resource"].as_object_mut().unwrap().insert(k, v);
                self.body.data.resource.insert(k, v);
            }
        //}
        //self.resource = resource;
        self
    }

    pub fn build(&mut self) -> anyhow::Result<Value> {
        Ok(serde_json::to_value(&self.body)?)
    }
}

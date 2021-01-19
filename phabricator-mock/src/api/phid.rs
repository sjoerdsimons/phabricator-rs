use crate::*;
use serde_json::json;

pub struct Lookup;

impl PhabRespond for Lookup {
    fn respond(&self, server: &PhabMockServer, params: &Params, _: &Request) -> ResponseTemplate {
        let names = params.get_values(&["names"]).expect("Expected names");

        let mut responses = HashMap::new();
        for n in names {
            if let Some(id) = n.strip_prefix("T") {
                let id = id.parse().expect("Couldn't parse task");
                if let Some(t) = server.get_task(id) {
                    responses.insert(
                        n,
                        json!({
                            "phid": t.phid,
                            "uri": format!("{}/T{}", server.uri(), t.id),
                            "typeName":"Maniphest Task",
                            "type":"TASK",
                            "name": format!("T{}", t.id),
                            "fullName": t.full_name,
                            "status":"closed"
                        }),
                    );
                }
            }
        }

        if responses.is_empty() {
            ResponseTemplate::new(200).set_body_json(json!({
                "result": [],
                "error_code":null,
                "error_info":null
            }))
        } else {
            ResponseTemplate::new(200).set_body_json(json!({
                "result": responses,
                "error_code":null,
                "error_info":null
            }))
        }
    }
}

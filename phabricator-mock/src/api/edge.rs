use crate::*;
use serde_json::json;

pub struct Search;
impl PhabRespond for Search {
    fn respond(&self, server: &PhabMockServer, params: &Params, _: &Request) -> ResponseTemplate {
        const SUBTASK: &str = "task.subtask";
        const PARENT: &str = "task.parent";

        let sources = params
            .get_values(&["sourcePHIDs"])
            .expect("Expected source phids");
        let destinations = params.get_values(&["destinationPHIDs"]);
        if destinations.is_some() {
            panic!("edge.search doesnt handle destinations yet")
        }
        let types = params.get_values(&["types"]).expect("Expected edge types");

        if types
            .iter()
            .any(|t| ![PARENT, SUBTASK].contains(&t.as_str()))
        {
            panic!("Unrecognized type in {:?}", types);
        }

        let mut responses = Vec::new();
        for phid in sources {
            let phid = phid.parse().expect("Couldn't parse task");
            // TODO check types
            if let Some(t) = server.find_task(&phid) {
                if types.iter().any(|t| t == PARENT) {
                    responses.extend(t.parents().iter().map(|p| {
                        json!({
                            "sourcePHID": t.phid,
                            "edgeType": PARENT,
                            "destinationPHID": p.phid
                        })
                    }));
                }

                if types.iter().any(|t| t == SUBTASK) {
                    responses.extend(t.subtasks().iter().map(|p| {
                        json!({
                            "sourcePHID": t.phid,
                            "edgeType": SUBTASK,
                            "destinationPHID": p.phid
                        })
                    }));
                }
            }
        }

        // TODO handle cursor
        ResponseTemplate::new(200).set_body_json(json!({
            "result":  {
                "data": responses ,
                "cursor": {
                    "limit": 100,
                    "after": null,
                    "before": null,
                    "order": null,
                },
            },
            "error_code":null,
            "error_info":null
        }))
    }
}

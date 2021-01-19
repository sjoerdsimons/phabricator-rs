use crate::*;
use serde_json::json;
use serde_json::value::Value::Null;

pub struct Info;
impl PhabRespond for Info {
    fn respond(&self, server: &PhabMockServer, params: &Params, _: &Request) -> ResponseTemplate {
        let id = params
            .get(&["task_id"])
            .expect("Expected a task id")
            .parse()
            .expect("Expected a numeric id");

        if let Some(t) = server.get_task(id) {
            ResponseTemplate::new(200).set_body_json(json!({
                "result": {
                    "id": t.id.to_string(),
                    "phid": t.phid,
                    "authorPHID": t.author.phid,
                    "ownerPHID": t.owner.as_ref().map(| u | &u.phid),
                    "ccPHIDs": [
                        // TODO cc's
                    ],
                    "status": t.status.value,
                    "statusName": t.status.name,
                    "isClosed": t.status.closed,
                    "priority": t.priority.name,
                    "priorityColor": t.priority.color,
                    "title": t.full_name,
                    "description": t.description,
                    "projectPHIDs": [
                        //TODO projects
                    ],
                    "uri": format!("{}/T{}", server.uri(), t.id),
                    "auxiliary": { },
                    "objectName": format!("T{}", t.id),
                    "dateCreated": t.date_created.to_string(),
                    "dateModified": t.date_modified.to_string(),
                    "dependsOnTaskPHIDs": [
                        // TODO depends
                    ]
                },
                "error_code": Null,
                "error_info": Null,
            }))
        } else {
            ResponseTemplate::new(200).set_body_json(json!({
              "result": Null,
              "error_code": "ERR_BAD_TASK",
              "error_info":"No such Maniphest task exists."
            }))
        }
    }
}

pub struct Search;

impl Search {
    fn attachment(&self, params: &Params, a: &str) -> bool {
        params
            .get(&["attachments", a])
            .map(|v| match v {
                "true" => true,
                "false" => false,
                _ => panic!("Expected boolean for {}", a),
            })
            .unwrap_or(false)
    }
}

impl PhabRespond for Search {
    fn respond(&self, server: &PhabMockServer, params: &Params, _: &Request) -> ResponseTemplate {
        let ids = params.get_values(&["constraints", "ids"]);
        // TODO support query key
        let phids = params.get_values(&["constraints", "phids"]);
        let querykey = params.get_values(&["queryKey"]);
        let subscribers = self.attachment(params, "subscribers");
        let columns = self.attachment(params, "columns");
        let projects = self.attachment(params, "projects");

        let tasks: Box<dyn Iterator<Item = Task>> = match (ids, phids) {
            (Some(_), Some(_)) => panic!("Only expected one of ids or phds"),
            (None, None) => panic!("Expected either phids or ids constraints"),
            (Some(ref ids), _) => Box::new(ids.iter().filter_map(|id| {
                let id = id.parse().expect("Couldn't parse task");
                server.get_task(id)
            })),
            (_, Some(ref phids)) => Box::new(phids.iter().filter_map(|phid| {
                let phid = phid.parse().expect("Couldn't parse phdi");
                server.find_task(&phid)
            })),
        };

        let responses: Vec<_> = tasks.map(| t | {
                let mut attachments = HashMap::new();

                if subscribers {
                    attachments.insert(
                        "subscribers",
                        json!({
                            "subscriberPHIDs": t.subscribers.iter().map( | u | &u.phid ).collect::<Vec<_>>(),
                            "subscriberCount": t.subscribers.len(),
                            "viewerIsSubscribed": false
                        }),
                    );
                }

                if columns {
                    if t.columns.is_empty() {
                        attachments.insert("columns", json!({ "boards": [] }));
                    } else {
                        attachments.insert(
                            "columns",
                            json!({
                                "boards":
                                    t.columns.iter().map( | c |
                                        (&c.project.phid,
                                         json!({
                                             "columns": [
                                                { "id": c.id, "phid": c.phid, "name": c.name }
                                             ]
                                         })))
                                    .collect::<HashMap<_, _>>(),
                            }),
                        );
                    }
                }

                if projects {
                    let projects: Vec<_> = t
                        .columns
                        .iter()
                        .map(|c| &c.phid)
                        .chain(t.projects.iter().map(|p| &p.phid))
                        .collect();
                    attachments.insert("projects", json!({ "projectPHIDs": projects }));
                }

                json!({
                    "id": t.id,
                    "type": "TASK",
                    "phid": t.phid,
                    "fields": {
                        "name": t.full_name,
                        "description": { "raw": t.description, },
                        "authorPHID": t.author.phid,
                        "ownerPHID": t.owner.as_ref().map(| u | &u.phid),
                        "status": {
                            "value": t.status.value,
                            "name": t.status.name,
                            "color": t.status.color,
                        },
                        "priority": {
                            "value": t.priority.value,
                            "name": t.priority.name,
                            "color": t.priority.color,
                        },
                        "points": t.points,
                        "subtype": "default",
                        "closerPHID": t.closer.as_ref().map(| c| &c.phid),
                        "dateClosed": t.date_closed,
                        "spacePHID": t.space.as_ref().map(| s | &s.phid),
                        "dateCreated": t.date_created,
                        "dateModified": t.date_modified,
                        "policy": {
                            "view": t.policy.view,
                            "interact": t.policy.interact,
                            "edit": t.policy.edit,
                        },
                    },
                    "attachments": attachments,
                })
            })
        .collect();

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
                "maps": {},
                "query": {
                    "queryKey": querykey
                }
            },
            "error_code":null,
            "error_info":null
        }))
    }
}

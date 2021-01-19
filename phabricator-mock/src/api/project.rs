use crate::*;
use serde_json::json;

pub struct Search;
impl Search {
    fn add_project(responses: &mut Vec<serde_json::Value>, p: &Project) {
        responses.push(json!({
            "id": p.id,
            "type": "PROJ",
            "phid": p.phid,
            "fields": {
                "name": p.name,
                "slug": p.slug,
                "subtype": "default",
                "milestone": null,
                "depth": 0,
                "parent": null,
                "icon": {
                    "key": p.icon.key,
                    "name": p.icon.name,
                    "icon": p.icon.icon,
                },
                "color": {
                    "key": p.color.key,
                    "name": p.color.name,
                },
                "spacePHID": p.space.as_ref().map(| s | &s.phid),
                "dateCreated": p.created,
                "dateModified": p.modified,
                "policy": {
                    "view": p.policy.view,
                    "edit": p.policy.edit,
                    "join": p.policy.join,
                },
                "description": p.description,
            },
            "attachments": {}
        }));
    }
}

impl PhabRespond for Search {
    fn respond(&self, server: &PhabMockServer, params: &Params, _: &Request) -> ResponseTemplate {
        let ids = params.get_values(&["constraints", "ids"]);
        let phids = params.get_values(&["constraints", "phids"]);
        // TODO support query key
        let querykey = params.get_values(&["queryKey"]);

        assert!(
            ids.is_none() || phids.is_none(),
            "Both ids and phids constrained"
        );

        let mut responses = Vec::new();

        if let Some(ids) = ids {
            for id in ids {
                let id = id.parse().expect("Couldn't parse task");
                if let Some(p) = server.get_project(id) {
                    Self::add_project(&mut responses, &p);
                }
            }
        }

        if let Some(phids) = phids {
            for phid in phids {
                let phid = phid.parse().expect("Unusable Phid");
                if let Some(p) = server.find_project(&phid) {
                    Self::add_project(&mut responses, &p);
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
                "maps": {},
                "query": {
                    "queryKey": querykey
                }
            },
            "error_code": null,
            "error_info": null
        }))
    }
}

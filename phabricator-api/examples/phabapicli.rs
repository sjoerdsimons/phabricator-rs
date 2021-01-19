use anyhow::Result;
use phabricator_api::phid::Lookup;
use phabricator_api::*;
use std::default::Default;
use structopt::StructOpt;
use tokio;

#[derive(StructOpt, Debug)]
struct PhidLookup {
    names: Vec<String>,
}

#[derive(StructOpt, Debug)]
struct PhidQuery {
    phids: Vec<String>,
}

#[derive(StructOpt, Debug)]
struct ManiphestInfo {
    id: u32,
}

#[derive(StructOpt, Debug)]
struct ManiphestSearch {
    #[structopt(short = "key", long)]
    query_key: Option<String>,
    #[structopt(short, long)]
    query: Option<String>,
    #[structopt(short, long)]
    projects: Option<Vec<String>>,
    #[structopt(short, long)]
    ids: Option<Vec<u32>>,
    #[structopt(short, long)]
    subscribers: bool,
    #[structopt(long)]
    attach_projects: bool,
    #[structopt(short, long)]
    columns: bool,
}

#[derive(StructOpt, Debug)]
struct ProjectSearch {
    #[structopt(short = "k", long)]
    query_key: Option<String>,
    #[structopt(short, long)]
    ids: Option<Vec<u32>>,
    #[structopt(short, long)]
    slugs: Option<Vec<String>>,
    #[structopt(short, long)]
    query: Option<String>,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "phid.lookup")]
    PhidLookup(PhidLookup),
    #[structopt(name = "phid.query")]
    PhidQuery(PhidQuery),
    #[structopt(name = "maniphest.info")]
    ManiphestInfo(ManiphestInfo),
    #[structopt(name = "maniphest.search")]
    ManiphestSearch(ManiphestSearch),
    #[structopt(name = "project.search")]
    ProjectSearch(ProjectSearch),
}

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long, default_value = "https://phabricator.collabora.com/")]
    server: String,
    token: String,
    #[structopt(subcommand)]
    command: Command,
}

async fn phid_lookup(client: Client, l: PhidLookup) -> Result<()> {
    let lookup = Lookup { names: l.names };

    let r = client.request(&lookup).await?;
    println!("=> {:#?}", r);

    Ok(())
}

async fn phid_query(client: Client, l: PhidQuery) -> Result<()> {
    let query = phid::Query { phids: l.phids };

    let r = client.request(&query).await?;
    println!("=> {:#?}", r);

    Ok(())
}

async fn maniphest_info(client: Client, m: ManiphestInfo) -> Result<()> {
    let info = maniphest::info::Info { task_id: m.id };

    let r = client.request(&info).await?;
    println!("=> {:#?}", r);

    Ok(())
}

async fn maniphest_search(client: Client, m: ManiphestSearch) -> Result<()> {
    let mut search: maniphest::search::Search = Default::default();
    search.query_key = m.query_key;
    search.constraints.ids = m.ids;
    search.constraints.query = m.query;
    search.constraints.projects = m.projects;

    search.attachments.subscribers = m.subscribers;
    search.attachments.projects = m.attach_projects;
    search.attachments.columns = m.columns;

    let mut r = client.request(&search).await?;
    println!("=> {:#?}", r);

    while r.cursor.after.is_some() {
        let c = maniphest::search::SearchCursor {
            cursor: &r.cursor,
            search: &search,
        };
        r = client.request(&c).await?;
        println!("=> {:#?}", r);
    }

    Ok(())
}

async fn project_search(client: Client, p: ProjectSearch) -> Result<()> {
    let mut search: project::search::Search = Default::default();
    search.query_key = p.query_key;
    search.constraints.ids = p.ids;
    search.constraints.slugs = p.slugs;
    search.constraints.query = p.query;

    let mut r = client.request(&search).await?;
    println!("=> {:#?}", r);

    while r.cursor.after.is_some() {
        let c = project::search::SearchCursor {
            cursor: &r.cursor,
            search: &search,
        };
        r = client.request(&c).await?;
        println!("=> {:#?}", r);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let opts = Opt::from_args();

    let client = Client::new(
        opts.server.parse().expect("Failed to parse server url"),
        opts.token,
    );

    match opts.command {
        Command::PhidLookup(l) => phid_lookup(client, l).await,
        Command::PhidQuery(l) => phid_query(client, l).await,
        Command::ManiphestInfo(i) => maniphest_info(client, i).await,
        Command::ManiphestSearch(i) => maniphest_search(client, i).await,
        Command::ProjectSearch(i) => project_search(client, i).await,
    }
}

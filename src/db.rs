use std::time::Duration;

use mongodb::bson::{Document, doc};
use mongodb::options::ClientOptions;
use mongodb::sync::Client;

pub const PAGE_SIZE: u64 = 20;

pub fn connect(uri: &str) -> Result<Client, String> {
    let mut options = ClientOptions::parse(uri.trim())
        .run()
        .map_err(|e| format!("invalid uri: {}", e))?;

    options
        .server_selection_timeout
        .get_or_insert(Duration::from_secs(3));
    options
        .connect_timeout
        .get_or_insert(Duration::from_secs(3));

    let client = Client::with_options(options).map_err(|e| format!("connection error: {}", e))?;
    client
        .database("admin")
        .run_command(doc! {"ping": 1})
        .run()
        .map_err(|e| format!("ping failed: {}", e))?;
    Ok(client)
}

pub fn list_databases(client: &Client) -> Vec<String> {
    client.list_database_names().run().unwrap_or_default()
}

pub fn list_collections(client: &Client, db: &str) -> Vec<String> {
    client
        .database(db)
        .list_collection_names()
        .run()
        .unwrap_or_default()
}

pub fn fetch_documents(client: &Client, db: &str, coll: &str, page: u64) -> Vec<Document> {
    client
        .database(db)
        .collection::<Document>(coll)
        .find(doc! {})
        .skip(page * PAGE_SIZE)
        .limit(PAGE_SIZE as i64)
        .run()
        .map(|cursor| cursor.filter_map(Result::ok).collect())
        .unwrap_or_default()
}

pub fn replace_document(
    client: &Client,
    db: &str,
    coll: &str,
    original: &Document,
    edited: Document,
) -> Result<(), String> {
    let id = original.get("_id").ok_or("document has no _id")?.clone();

    client
        .database(db)
        .collection::<Document>(coll)
        .replace_one(doc! {"_id": id}, edited)
        .run()
        .map_err(|e| e.to_string())
        .and_then(|r| {
            if r.matched_count == 0 {
                Err("document no longer exists".to_string())
            } else {
                Ok(())
            }
        })
}

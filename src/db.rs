use std::time::Duration;

use mongodb::bson::{Bson, Document, doc};
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

pub fn parse_filter(text: &str) -> Result<Document, String> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(Document::new());
    }
    serde_json::from_str::<Document>(text).map_err(|e| e.to_string())
}

pub fn fetch_documents(
    client: &Client,
    db: &str,
    coll: &str,
    filter: &Document,
    page: u64,
) -> Result<Vec<Document>, String> {
    let cursor = client
        .database(db)
        .collection::<Document>(coll)
        .find(filter.clone())
        .sort(doc! {"_id": 1})
        .skip(page * PAGE_SIZE)
        .limit(PAGE_SIZE as i64)
        .run()
        .map_err(|e| e.to_string())?;

    Ok(cursor.filter_map(Result::ok).collect())
}

fn count_documents(
    client: &Client,
    db: &str,
    coll: &str,
    filter: &Document,
) -> Result<u64, String> {
    client
        .database(db)
        .collection::<Document>(coll)
        .count_documents(filter.clone())
        .run()
        .map_err(|e| e.to_string())
}

pub fn fetch_count_documents(
    client: &Client,
    db: &str,
    coll: &str,
    filter: &Document,
    page: u64,
) -> Result<(Vec<Document>, u64), String> {
    std::thread::scope(|s| {
        let count_handle = s.spawn(|| count_documents(client, db, coll, filter));
        let fetch_handle = s.spawn(|| fetch_documents(client, db, coll, filter, page));

        let count = count_handle.join().map_err(|_| "count thread panicked")??;
        let docs = fetch_handle.join().map_err(|_| "fetch thread panicked")??;

        Ok((docs, count))
    })
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

pub fn delete_document(client: &Client, db: &str, coll: &str, id: &Bson) -> Result<(), String> {
    let result = client
        .database(db)
        .collection::<Document>(coll)
        .delete_one(doc! {"_id": id.clone()})
        .run()
        .map_err(|e| e.to_string())?;

    if result.deleted_count == 0 {
        Err("document no longer exists".to_string())
    } else {
        Ok(())
    }
}

pub fn drop_database(client: &Client, db: &str) -> Result<(), String> {
    client.database(db).drop().run().map_err(|e| e.to_string())
}

pub fn create_collection(client: &Client, db: &str, name: &str) -> Result<(), String> {
    client
        .database(db)
        .create_collection(name)
        .run()
        .map_err(|e| e.to_string())
}

pub fn drop_collection(client: &Client, db: &str, coll: &str) -> Result<(), String> {
    client
        .database(db)
        .collection::<Document>(coll)
        .drop()
        .run()
        .map_err(|e| e.to_string())
}

pub fn insert_document(
    client: &Client,
    db: &str,
    coll: &str,
    doc: &Document,
) -> Result<Bson, String> {
    client
        .database(db)
        .collection::<Document>(coll)
        .insert_one(doc)
        .run()
        .map_err(|e| e.to_string())
        .map(|r| r.inserted_id)
}

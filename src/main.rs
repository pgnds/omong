use mongodb::bson::doc;

const MONGO_URI: &str = "mongodb://test:test@localhost:27017";

fn main() {
    let client = mongodb::sync::Client::with_uri_str(MONGO_URI).unwrap();
    client.database("admin").run_command(doc! {"ping": 1}).run().unwrap();

    println!("successfully pinged mongodb!");
}


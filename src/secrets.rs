use keyring::Entry;
use uuid::Uuid;

const SERVICE: &str = "omong";

pub fn save_uri(id: Uuid, uri: &str) -> keyring::Result<()> {
    entry(id)?.set_password(uri)
}

pub fn get_uri(id: Uuid) -> keyring::Result<String> {
    entry(id)?.get_password()
}

pub fn delete_uri(id: Uuid) -> keyring::Result<()> {
    match entry(id)?.delete_credential() {
        Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e),
    }
}

fn entry(id: Uuid) -> keyring::Result<Entry> {
    Entry::new(SERVICE, &id.to_string())
}

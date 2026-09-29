use std::io;

fn connections_file_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    std::path::PathBuf::from(home).join(".config/omong/connections")
}

pub fn load_connections() -> Vec<(String, String)> {
    let path = connections_file_path();
    let content = std::fs::read_to_string(path).unwrap_or_default();
    content
        .lines()
        .filter_map(|line| line.split_once('|'))
        .map(|(alias, uri)| (alias.to_string(), uri.to_string()))
        .collect()
}

pub fn save_connection(alias: &str, uri: &str) -> io::Result<()> {
    let path = connections_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    writeln!(file, "{}|{}", alias, uri)
}

pub fn delete_connection(
    connections: &[(String, String)],
    alias: &str,
) -> io::Result<Vec<(String, String)>> {
    let remaining: Vec<(String, String)> = connections
        .iter()
        .filter(|(a, _)| a != alias)
        .cloned()
        .collect();

    let path = connections_file_path();
    let mut content = String::new();
    for (a, u) in &remaining {
        content.push_str(&format!("{}|{}\n", a, u));
    }
    std::fs::write(path, content)?;

    Ok(remaining)
}

pub fn update_connection(
    connections: &[(String, String)],
    old_alias: &str,
    new_alias: &str,
    new_uri: &str,
) -> io::Result<Vec<(String, String)>> {
    let mut updated: Vec<(String, String)> = connections.to_vec();
    for (alias, uri) in updated.iter_mut() {
        if alias == old_alias {
            *alias = new_alias.to_string();
            *uri = new_uri.to_string();
            break;
        }
    }

    let path = connections_file_path();
    let mut content = String::new();
    for (alias, uri) in &updated {
        content.push_str(&format!("{}|{}\n", alias, uri));
    }
    std::fs::write(path, content)?;

    Ok(updated)
}

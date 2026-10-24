use std::io;
use std::os::unix::fs::OpenOptionsExt;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Connection {
    id: Uuid,
    pub alias: String,
}

impl Connection {
    pub fn new(alias: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            alias,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}

fn connections_file_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    std::path::PathBuf::from(home).join(".config/omong/connections")
}

pub fn load_connections() -> Vec<Connection> {
    let path = connections_file_path();
    let content = std::fs::read_to_string(path).unwrap_or_default();
    content
        .lines()
        .filter_map(|line| {
            let (id, alias) = line.split_once('|')?;
            let id: Uuid = Uuid::parse_str(id).ok()?;

            Some(Connection {
                id,
                alias: alias.to_string(),
            })
        })
        .collect()
}

pub fn save_connection(connection: &Connection) -> io::Result<()> {
    let path = connections_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(path)?;

    writeln!(file, "{}|{}", connection.id, connection.alias)
}

pub fn delete_connection(connections: &[Connection], id: Uuid) -> io::Result<Vec<Connection>> {
    let remaining: Vec<Connection> = connections.iter().filter(|c| c.id != id).cloned().collect();

    write_connections(&remaining)?;
    Ok(remaining)
}

pub fn update_connection(
    connections: &[Connection],
    new_connection: &Connection,
) -> io::Result<Vec<Connection>> {
    let mut updated: Vec<Connection> = connections.to_vec();
    if let Some(c) = updated.iter_mut().find(|c| c.id == new_connection.id) {
        *c = new_connection.clone();
    }

    write_connections(&updated)?;
    Ok(updated)
}

fn write_connections(connections: &[Connection]) -> io::Result<()> {
    let path = connections_file_path();
    let mut content = String::new();
    for c in connections {
        content.push_str(&format!("{}|{}\n", c.id, c.alias));
    }
    std::fs::write(path, content)
}

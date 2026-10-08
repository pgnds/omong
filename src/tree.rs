use std::collections::HashSet;

use mongodb::bson::{Bson, Document};

pub struct TreeLine {
    pub path: String,
    pub depth: usize,
    pub label: String,
    pub summary: String,
    pub is_container: bool,
    pub expanded: bool,
}

pub fn flatten(doc: &Document, expanded: &HashSet<String>) -> Vec<TreeLine> {
    let mut out = Vec::new();
    for (k, v) in doc {
        push(&mut out, k.clone(), k.clone(), v, 0, expanded);
    }
    out
}

fn push(
    out: &mut Vec<TreeLine>,
    path: String,
    label: String,
    value: &Bson,
    depth: usize,
    expanded: &HashSet<String>,
) {
    match value {
        Bson::Document(d) => {
            let open = expanded.contains(&path);
            out.push(TreeLine {
                path: path.clone(),
                depth,
                label,
                summary: format!("{{ {} fields }}", d.len()),
                is_container: true,
                expanded: open,
            });
            if open {
                for (k, v) in d {
                    push(
                        out,
                        format!("{}.{}", path, k),
                        k.clone(),
                        v,
                        depth + 1,
                        expanded,
                    );
                }
            }
        }
        Bson::Array(a) => {
            let open = expanded.contains(&path);
            out.push(TreeLine {
                path: path.clone(),
                depth,
                label,
                summary: format!("[ {} items ]", a.len()),
                is_container: true,
                expanded: open,
            });
            if open {
                for (i, v) in a.iter().enumerate() {
                    push(
                        out,
                        format!("{}.{}", path, i),
                        format!("[{}]", i),
                        v,
                        depth + 1,
                        expanded,
                    );
                }
            }
        }
        other => out.push(TreeLine {
            path,
            depth,
            label,
            summary: other.to_string(),
            is_container: false,
            expanded: false,
        }),
    }
}

pub fn all_container_paths(doc: &Document) -> HashSet<String> {
    let mut set = HashSet::new();
    for (k, v) in doc {
        collect(&mut set, k.clone(), v);
    }
    set
}

fn collect(set: &mut HashSet<String>, path: String, value: &Bson) {
    match value {
        Bson::Document(d) => {
            set.insert(path.clone());
            for (k, v) in d {
                collect(set, format!("{}.{}", path, k), v);
            }
        }
        Bson::Array(a) => {
            set.insert(path.clone());
            for (i, v) in a.iter().enumerate() {
                collect(set, format!("{}.{}", path, i), v);
            }
        }
        _ => {}
    }
}

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub target: String,
    pub entry: String,
    pub route: String,
}

impl Project {
    pub fn load(path: &str) -> Result<Project, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("nie moge czytac {}: {}", path, e))?;
        Ok(parse(&text))
    }
}

fn parse(text: &str) -> Project {
    let mut map = HashMap::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }

    Project {
        name: map.get("name").cloned().unwrap_or_else(|| "prog".to_string()),
        target: map.get("target").cloned().unwrap_or_else(|| "x86_64".to_string()),
        entry: map.get("entry").cloned().unwrap_or_else(|| "src/main.xl".to_string()),
        route: map.get("route").cloned().unwrap_or_else(|| "asm".to_string()),
    }
}
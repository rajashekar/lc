use std::collections::HashMap;

#[derive(Clone)]
pub struct McpServerConfig {
    pub name: String,
    pub env: HashMap<String, String>,
}

struct RedactedEnv<'a>(&'a HashMap<String, String>);

impl<'a> std::fmt::Debug for RedactedEnv<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        for k in self.0.keys() {
            map.entry(k, &"[REDACTED]");
        }
        map.finish()
    }
}

impl std::fmt::Debug for McpServerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpServerConfig")
            .field("name", &self.name)
            .field("env", &RedactedEnv(&self.env))
            .finish()
    }
}

fn main() {
    let mut env = HashMap::new();
    env.insert("SECRET_KEY".to_string(), "12345".to_string());
    let config = McpServerConfig {
        name: "test".to_string(),
        env,
    };
    println!("{:?}", config);
}

use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone)]
pub enum SdkMcpTransport {
    Stdio {
        command: String,
        args: Option<Vec<String>>,
        env: Option<HashMap<String, String>>,
        cwd: Option<PathBuf>,
    },
    Sse {
        url: String,
    },
}

struct RedactedEnvOpt<'a>(&'a Option<HashMap<String, String>>);

impl<'a> std::fmt::Debug for RedactedEnvOpt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(env) => {
                let mut map = f.debug_map();
                for k in env.keys() {
                    map.entry(k, &"[REDACTED]");
                }
                map.finish()
            }
            None => write!(f, "None"),
        }
    }
}

impl std::fmt::Debug for SdkMcpTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdio {
                command,
                args,
                env,
                cwd,
            } => {
                f.debug_struct("Stdio")
                    .field("command", command)
                    .field("args", args)
                    .field("env", &RedactedEnvOpt(env))
                    .field("cwd", cwd)
                    .finish()
            }
            Self::Sse { url } => f.debug_struct("Sse").field("url", url).finish(),
        }
    }
}

fn main() {
    let mut env = HashMap::new();
    env.insert("SECRET_KEY".to_string(), "12345".to_string());
    let config = SdkMcpTransport::Stdio {
        command: "test".to_string(),
        args: None,
        env: Some(env),
        cwd: None,
    };
    println!("{:?}", config);
}

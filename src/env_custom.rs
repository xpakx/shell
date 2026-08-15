use std::collections::HashMap;

struct EnvEntry {
    value: String,
    exported: bool,
}

pub struct Env {
    vars: HashMap<String, EnvEntry>,
}

pub fn create_env() -> Env {
    let mut env = Env {
        vars: HashMap::new(),
    };
    for (key, value) in std::env::vars() {
        let entry = EnvEntry {
            value,
            exported: true,
        };
        env.vars.insert(key, entry);
    }
    env
}

impl Env {
    pub fn get(&self, key: &str) -> Option<String> {
        self.vars.get(key)
            .or_else(|| self.vars.get(key))
            .map(|v| String::from(&v.value))
    }

    pub fn set_var(&mut self, key: String, value: &str) {
        self.vars
            .entry(key)
            .and_modify(|entry| {
                entry.value = String::from(value);
            })
        .or_insert_with(|| {
            EnvEntry {
                value: String::from(value),
                exported: false,
            }
        });
    }

    pub fn export(&mut self, key: String) {
        self.vars
            .entry(key)
            .and_modify(|entry| {
                entry.exported = true;
            });
    }

    pub fn home(&self) -> String {
        // TODO: better fallback
        self.get("HOME")
            .unwrap_or(String::from("/"))
    }
}

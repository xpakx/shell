use std::{collections::HashMap, process::Command};

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
        self.vars.get(key).map(|v| String::from(&v.value))
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

    pub fn apply_to_command(&self, cmd: &mut Command) {
        // cmd.env_clear(); // ????

        let exported_vars: Vec<(&String, &String)> = self.vars
            .iter()
            .filter(|(_, entry)| entry.exported)
            .map(|(key, entry)| (key, &entry.value))
            .collect();

        cmd.envs(exported_vars);
    }
}

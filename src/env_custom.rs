use std::collections::HashMap;


pub struct Env {
    pub vars: HashMap<String, String>,
    pub exports: HashMap<String, String>,
}

pub fn create_env() -> Env {
    let mut env = Env {
        vars: HashMap::new(),
        exports: HashMap::new(),
    };
    for (key, value) in std::env::vars() {
        env.exports.insert(key, value);
    }
    env
}

impl Env {
    pub fn get(&self, key: &str) -> Option<String> {
        self.vars.get(key)
            .or_else(|| self.exports.get(key))
            .map(|v| String::from(v))
    }

    pub fn set_var(&mut self, key: String, value: &str) {
        self.vars.insert(key, String::from(value));
    }

    pub fn home(&self) -> String {
        // TODO: better fallback
        self.vars.get("HOME")
            .map(|v| String::from(v))
            .unwrap_or(String::from("/"))
    }
}

use core::result::Result;
use std::{collections::HashMap, path::{Path, PathBuf}, process::Command};

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
        cmd.env_clear(); // TODO

        let exported_vars: Vec<(&String, &String)> = self.vars
            .iter()
            .filter(|(_, entry)| entry.exported)
            .map(|(key, entry)| (key, &entry.value))
            .collect();

        // TODO: we need to use our PWD correctly first
        // if let Some(pwd_entry) = self.vars.get("PWD") {
        //    cmd.current_dir(&pwd_entry.value);
        // }

        cmd.envs(exported_vars);
    }

    pub fn set_current_dir(&mut self, new_path: &Path) -> Result<(), ()> {
        if let Some(current_pwd) = self.get("PWD") {
            self.set_var(String::from("OLDPWD"), &current_pwd);
        }

        let new_pwd_str = new_path.to_string_lossy().to_string();
        self.set_var(String::from("PWD"), &new_pwd_str);

        Ok(())
    }

    pub fn current_dir(&self) -> Option<PathBuf> {
        self.get("PWD").map(|p| PathBuf::from(p))
    }

    pub fn path(&self) -> String {
        self.get("PATH")
            .unwrap_or(String::from(""))
    }

    pub fn update_after_command(&mut self, exit_code: i32) {
        self.set_var(String::from("?"), &exit_code.to_string());
    }

}

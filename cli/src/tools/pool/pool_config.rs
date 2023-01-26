/*
    Copyright 2023 DSR Corporation, Denver, Colorado.
    https://www.dsr-corporation.com
    SPDX-License-Identifier: Apache-2.0
*/
use crate::{
    error::{CliError, CliResult},
    utils::environment::EnvironmentUtils,
};
use std::path::PathBuf;
use std::{
    fs,
    fs::File,
    io,
    io::{Read, Write},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct PoolConfig {
    pub genesis_txn: String,
}

pub struct PoolDirectory {
    pub name: String,
}

impl PoolDirectory {
    pub(crate) fn from(name: &str) -> Self {
        PoolDirectory {
            name: name.to_string(),
        }
    }

    pub(crate) fn store_config(&self, config: &PoolConfig) -> CliResult<()> {
        let mut path = self.path();

        if path.as_path().exists() {
            return Err(CliError::Duplicate(format!(
                "Pool {} already exists!",
                &self.name
            )));
        }

        fs::create_dir_all(path.as_path())?;

        // copy genesis transactions
        {
            path.push(&self.name);
            path.set_extension("txn");

            let mut gt_fin = File::open(&config.genesis_txn)?;
            let mut gt_fout = File::create(path.as_path())?;
            io::copy(&mut gt_fin, &mut gt_fout)?;
        }
        let txn_path = path.to_string_lossy().to_string();

        path.pop();

        // store config file
        {
            path.push("config");
            path.set_extension("json");

            let pool_config = json!({ "genesis_txn": txn_path });

            let mut f: File = File::create(path.as_path())?;
            f.write_all(pool_config.to_string().as_bytes())?;
            f.flush()?;
        }

        Ok(())
    }

    pub(crate) fn read_config(&self) -> CliResult<PoolConfig> {
        let path = EnvironmentUtils::pool_config_path(&self.name);

        let mut config_json = String::new();

        let mut file = File::open(path)?;
        file.read_to_string(&mut config_json)?;

        let config = serde_json::from_str(&config_json)?;
        Ok(config)
    }

    pub(crate) fn delete_config(&self) -> CliResult<()> {
        let path = self.path();
        if !path.as_path().exists() {
            return Err(CliError::NotFound(format!(
                "Pool \"{}\" does not exist.",
                &self.name
            )));
        }
        fs::remove_dir_all(path).map_err(CliError::from)
    }

    pub(crate) fn list_pools() -> CliResult<String> {
        let mut pools = Vec::new();
        let pool_home_path = EnvironmentUtils::pool_home_path();

        if let Ok(entries) = fs::read_dir(pool_home_path) {
            for entry in entries {
                let dir_entry = if let Ok(dir_entry) = entry {
                    dir_entry
                } else {
                    continue;
                };
                if let Some(pool_name) = dir_entry
                    .path()
                    .file_name()
                    .and_then(|os_str| os_str.to_str())
                {
                    let json = json!({ "pool": pool_name.to_owned() });
                    pools.push(json);
                }
            }
        }

        let pools = json!(pools).to_string();
        Ok(pools)
    }

    pub(crate) fn store_pool_transactions(&self, transactions: &Vec<String>) -> CliResult<()> {
        let path = EnvironmentUtils::pool_transactions_path(&self.name);
        let mut f = File::create(path.as_path())?;
        f.write_all(transactions.join("\n").as_bytes())?;
        Ok(())
    }

    fn path(&self) -> PathBuf {
        EnvironmentUtils::pool_path(&self.name)
    }
}

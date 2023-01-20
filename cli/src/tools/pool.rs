/*
    Copyright 2023 DSR Corporation, Denver, Colorado.
    https://www.dsr-corporation.com
    SPDX-License-Identifier: Apache-2.0
*/
use crate::{
    error::{CliError, CliResult},
    utils::{
        futures::block_on,
        pool_directory::{PoolConfig, PoolDirectory},
    },
};
use std::collections::HashMap;

use indy_vdr::{
    config::PoolConfig as OpenPoolConfig,
    pool::{helpers::perform_refresh, LocalPool, Pool as PoolImpl, PoolBuilder, PoolTransactions},
};

pub struct Pool {}

impl Pool {
    pub fn create(name: &str, config: &PoolConfig) -> CliResult<()> {
        PoolDirectory::store_pool_config(name, config).map_err(CliError::from)
    }

    pub fn open(
        name: &str,
        config: OpenPoolConfig,
        pre_ordered_nodes: Option<Vec<&str>>,
    ) -> CliResult<LocalPool> {
        let pool_transactions_file = PoolDirectory::read_pool_config(name)
            .map_err(|_| CliError::NotFound(format!("Pool \"{}\" does not exist.", name)))?
            .genesis_txn;

        let weight_nodes = pre_ordered_nodes.map(|pre_ordered_nodes| {
            pre_ordered_nodes
                .into_iter()
                .map(|node| (node.to_string(), 2.0))
                .collect::<HashMap<String, f32>>()
        });

        let pool_transactions = PoolTransactions::from_json_file(&pool_transactions_file)?;

        PoolBuilder::from(config)
            .transactions(pool_transactions)?
            .node_weights(weight_nodes)
            .into_local()
            .map_err(CliError::from)
    }

    pub fn refresh(name: &str, pool: &LocalPool) -> CliResult<Option<LocalPool>> {
        let (transactions, _) = block_on(async move { perform_refresh(pool).await })?;

        match transactions {
            Some(new_transactions) if new_transactions.len() > 0 => {
                let mut transactions = PoolTransactions::from(pool.get_merkle_tree());
                transactions.extend_from_json(new_transactions)?;

                let pool = PoolBuilder::from(pool.get_config().to_owned())
                    .transactions(transactions)?
                    .into_local()?;

                PoolDirectory::store_pool_transactions(name, &pool.get_json_transactions()?)?;

                Ok(Some(pool))
            }
            _ => Ok(None),
        }
    }

    pub fn list() -> CliResult<String> {
        PoolDirectory::list_pools().map_err(CliError::from)
    }

    pub fn close(_pool: &LocalPool) -> CliResult<()> {
        Ok(())
    }

    pub fn delete(name: &str) -> CliResult<()> {
        PoolDirectory::delete_pool_config(name).map_err(CliError::from)
    }
}

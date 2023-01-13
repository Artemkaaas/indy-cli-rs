use crate::error::{CliError, CliResult};
use crate::utils::pool_config::{Config, PoolConfig};

use aries_askar::future::block_on;
use indy_vdr::{
    config::PoolConfig as OpenPoolConfig,
    pool::{helpers::perform_refresh, LocalPool, Pool as PoolImpl, PoolBuilder, PoolTransactions},
};

pub struct Pool {}

impl Pool {
    pub fn create_config(name: &str, config: &Config) -> CliResult<()> {
        PoolConfig::store(name, config).map_err(CliError::from)
    }

    pub fn open(name: &str, config: OpenPoolConfig) -> CliResult<LocalPool> {
        let pool_transactions_file = PoolConfig::read(name)
            .map_err(|_| CliError::NotFound(format!("Pool \"{}\" does not exist.", name)))?
            .genesis_txn;

        let pool_transactions = PoolTransactions::from_json_file(&pool_transactions_file)?;

        PoolBuilder::from(config)
            .transactions(pool_transactions)?
            .into_local()
            .map_err(CliError::from)
    }

    pub fn refresh(pool: &LocalPool) -> CliResult<Option<LocalPool>> {
        let (transactions, _) = block_on(async move { perform_refresh(pool).await })?;

        match transactions {
            Some(new_transactions) if new_transactions.len() > 0 => {
                let mut transactions = PoolTransactions::from(pool.get_merkle_tree());
                transactions.extend_from_json(new_transactions)?;

                let pool = PoolBuilder::from(pool.get_config().to_owned())
                    .transactions(transactions)?
                    .into_local()?;

                Ok(Some(pool))
            }
            _ => Ok(None),
        }
    }

    pub fn list() -> CliResult<String> {
        PoolConfig::list().map_err(CliError::from)
    }

    pub fn close(_pool: &LocalPool) -> CliResult<()> {
        // TODO: what should we do HERE?
        Ok(())
    }

    pub fn delete(name: &str) -> CliResult<()> {
        PoolConfig::delete(name).map_err(CliError::from)
    }
}

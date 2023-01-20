mod credentials;
mod uri;

use crate::{
    error::{CliError, CliResult},
    tools::did::constants::CATEGORY_DID,
    utils::{
        futures::block_on,
        wallet_backup::WalletBackup,
        wallet_directory::{WalletConfig, WalletDirectory},
    },
};

use self::{
    credentials::WalletCredentials,
    uri::{StorageType, WalletUri},
};

use aries_askar::{any::AnyStore, Error as AskarError, ErrorKind as AskarErrorKind, ManageBackend};
use serde_json::Value as JsonValue;

pub struct Wallet {}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Credentials {
    pub key: String,
    pub key_derivation_method: Option<String>,
    pub rekey: Option<String>,
    pub rekey_derivation_method: Option<String>,
    pub storage_credentials: Option<JsonValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportConfig {
    pub path: String,
    pub key: String,
    pub key_derivation_method: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportConfig {
    pub path: String,
    pub key: String,
    pub key_derivation_method: Option<String>,
}

impl Wallet {
    pub fn create(config: &WalletConfig, credentials: &Credentials) -> CliResult<AnyStore> {
        if WalletDirectory::is_wallet_config_exist(&config.id) {
            return Err(CliError::Duplicate(format!(
                "Wallet \"{}\" already exists",
                config.id
            )));
        }

        let wallet_uri = WalletUri::build(config, credentials, None)?;
        let credentials = WalletCredentials::build(credentials)?;

        WalletDirectory::create(config)?;

        block_on(async move {
            let store = wallet_uri
                .value()
                .provision_backend(
                    credentials.key_method,
                    credentials.key.as_ref(),
                    None,
                    false,
                )
                .await
                .map_err(CliError::from)?;

            // Askar: If there is any opened store when delete the wallet, function returns ok and deletes wallet file successfully
            // But next if we create wallet with the same again it will contain old records
            // So we have to close all store handles
            store.close().await?;

            Ok(store)
        })
    }

    pub fn open(config: &WalletConfig, credentials: &Credentials) -> CliResult<AnyStore> {
        let wallet_uri = WalletUri::build(config, credentials, None)?;
        let credentials = WalletCredentials::build(credentials)?;

        block_on(async move {
            let mut store: AnyStore = wallet_uri
                .value()
                .open_backend(Some(credentials.key_method), credentials.key.as_ref(), None)
                .await
                .map_err(|err: AskarError| match err.kind() {
                    AskarErrorKind::NotFound => CliError::NotFound(format!(
                        "Wallet \"{}\" not found or unavailable.",
                        config.id
                    )),
                    _ => CliError::from(err),
                })?;

            if let (Some(rekey), Some(rekey_method)) = (credentials.rekey, credentials.rekey_method)
            {
                store.rekey(rekey_method, rekey).await?;
            }

            Ok(store)
        })
    }

    pub fn close(store: &AnyStore) -> CliResult<()> {
        block_on(async move { store.close().await.map_err(CliError::from) })
    }

    pub fn delete(config: &WalletConfig, credentials: &Credentials) -> CliResult<bool> {
        let wallet_uri = WalletUri::build(config, credentials, None)?;

        block_on(async move {
            let removed = wallet_uri
                .value()
                .remove_backend()
                .await
                .map_err(CliError::from)?;
            if !removed {
                return Err(CliError::InvalidEntityState(format!(
                    "Unable to delete wallet {}",
                    config.id
                )));
            }
            WalletDirectory::delete(&config)?;
            Ok(removed)
        })
    }

    pub fn list() -> Vec<JsonValue> {
        WalletDirectory::list_wallets()
    }

    pub fn export(store: &AnyStore, export_config: &ExportConfig) -> CliResult<()> {
        let backup_config = WalletConfig {
            id: WalletBackup::get_id(&export_config.path),
            storage_type: StorageType::Sqlite.to_str().to_string(),
            ..WalletConfig::default()
        };
        let backup_credentials = Credentials {
            key: export_config.key.clone(),
            key_derivation_method: export_config.key_derivation_method.clone(),
            ..Credentials::default()
        };

        let backup_uri = WalletUri::build(
            &backup_config,
            &backup_credentials,
            Some(&export_config.path),
        )?;
        let backup_credentials = WalletCredentials::build(&backup_credentials)?;

        WalletBackup::init_directory(&export_config.path)?;

        block_on(async move {
            let backup_store = backup_uri
                .value()
                .provision_backend(
                    backup_credentials.key_method,
                    backup_credentials.key.as_ref(),
                    None,
                    false,
                )
                .await
                .map_err(CliError::from)?;

            Self::copy_records(&store, &backup_store).await?;

            backup_store.close().await?;

            Ok(())
        })
    }

    pub fn import(
        config: &WalletConfig,
        credentials: &Credentials,
        import_config: &ImportConfig,
    ) -> CliResult<()> {
        if !WalletBackup::is_wallet_backup_exist(&import_config.path) {
            return Err(CliError::NotFound(format!(
                "Wallet backup \"{}\" not found",
                import_config.path
            )));
        }

        if WalletDirectory::is_wallet_config_exist(&config.id) {
            return Err(CliError::Duplicate(format!(
                "Wallet \"{}\" already exists",
                config.id
            )));
        }

        let backup_config = WalletConfig {
            id: WalletBackup::get_id(&import_config.path),
            storage_type: StorageType::Sqlite.to_str().to_string(),
            ..WalletConfig::default()
        };
        let backup_credentials = Credentials {
            key: import_config.key.clone(),
            key_derivation_method: import_config.key_derivation_method.clone(),
            ..Credentials::default()
        };

        let backup_wallet_uri = WalletUri::build(
            &backup_config,
            &backup_credentials,
            Some(&import_config.path),
        )?;
        let backup_wallet_credentials = WalletCredentials::build(&backup_credentials)?;

        let new_wallet_uri = WalletUri::build(&config, &credentials, None)?;
        let new_wallet_credentials = WalletCredentials::build(&credentials)?;

        block_on(async move {
            let backup_store: AnyStore = backup_wallet_uri
                .value()
                .open_backend(
                    Some(backup_wallet_credentials.key_method),
                    backup_wallet_credentials.key.as_ref(),
                    None,
                )
                .await
                .map_err(|err: AskarError| match err.kind() {
                    AskarErrorKind::NotFound => CliError::NotFound(err.to_string()),
                    _ => CliError::from(err),
                })?;

            WalletDirectory::create(config)?;

            let new_store = new_wallet_uri
                .value()
                .provision_backend(
                    new_wallet_credentials.key_method,
                    new_wallet_credentials.key.as_ref(),
                    None,
                    false,
                )
                .await
                .map_err(CliError::from)?;

            Self::copy_records(&backup_store, &new_store).await?;

            backup_store.close().await?;
            new_store.close().await?;

            Ok(())
        })
    }

    async fn copy_records(from: &AnyStore, to: &AnyStore) -> CliResult<()> {
        let mut from_session = from.session(None).await?;
        let mut to_session = to.session(None).await?;

        let did_entries = from_session
            .fetch_all(CATEGORY_DID, None, None, false)
            .await?;

        for entry in did_entries {
            to_session
                .insert(
                    &entry.category,
                    &entry.name,
                    &entry.value,
                    Some(&entry.tags),
                    None,
                )
                .await
                .ok();
        }

        let key_entries = from_session
            .fetch_all_keys(None, None, None, None, false)
            .await?;

        for entry in key_entries {
            to_session
                .insert_key(
                    entry.name(),
                    &entry.load_local_key()?,
                    entry.metadata(),
                    None,
                    None,
                )
                .await
                .ok();
        }

        to_session.commit().await?;
        from_session.commit().await?;

        Ok(())
    }
}

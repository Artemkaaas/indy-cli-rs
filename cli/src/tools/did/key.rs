use crate::{
    error::{CliError, CliResult},
    tools::did::seed::Seed,
};

use aries_askar::{
    any::AnyStore,
    kms::{KeyAlg, LocalKey},
};
use indy_utils::base58;

pub struct Key(LocalKey);

impl Key {
    pub async fn create(
        store: &AnyStore,
        seed: Option<&str>,
        metadata: Option<&str>,
    ) -> CliResult<Key> {
        let keypair = match seed {
            Some(seed) => {
                let seed = Seed::from_str(seed)?;
                LocalKey::from_secret_bytes(KeyAlg::Ed25519, seed.value())?
            }
            None => LocalKey::generate(KeyAlg::Ed25519, false)?,
        };

        let key = Key(keypair);

        let verkey = key.verkey()?;

        let mut session = store.session(None).await?;
        session
            .insert_key(&verkey, key.value(), metadata, None, None)
            .await?;

        Ok(key)
    }

    pub fn value(&self) -> &LocalKey {
        &self.0
    }

    pub fn verkey(&self) -> CliResult<String> {
        let public_key = self.0.to_public_bytes()?;
        Ok(base58::encode(public_key))
    }

    pub async fn sign(store: &AnyStore, id: &str, bytes: &[u8]) -> CliResult<Vec<u8>> {
        Self::load(store, id)
            .await?
            .value()
            .sign_message(bytes, None)
            .map_err(CliError::from)
    }

    pub async fn load(store: &AnyStore, id: &str) -> CliResult<Key> {
        let mut session = store.session(None).await?;

        let local_key = session
            .fetch_key(id, false)
            .await?
            .ok_or_else(|| CliError::NotFound(format!("Key {} does not exits in the wallet!", id)))?
            .load_local_key()
            .map_err(CliError::from)?;

        Ok(Key(local_key))
    }
}

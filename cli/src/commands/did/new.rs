/*
    Copyright 2023 DSR Corporation, Denver, Colorado.
    https://www.dsr-corporation.com
    SPDX-License-Identifier: Apache-2.0
*/
use crate::{
    command_executor::{Command, CommandContext, CommandMetadata, CommandParams},
    commands::*,
    tools::did::Did,
};

pub mod new_command {
    use super::*;

    command!(CommandMetadata::build("new", "Create new DID")
        .add_optional_param("did", "Known DID for new wallet instance")
        .add_optional_deferred_param(
            "seed",
            "Seed for creating DID key-pair (UTF-8, base64 or hex)"
        )
        .add_optional_param("method", "Method name to create fully qualified DID")
        .add_optional_param("metadata", "DID metadata")
        .add_example("did new")
        .add_example("did new did=VsKV7grR1BUE29mG2Fm2kX")
        .add_example("did new did=VsKV7grR1BUE29mG2Fm2kX method=indy")
        .add_example("did new did=VsKV7grR1BUE29mG2Fm2kX seed=00000000000000000000000000000My1")
        .add_example("did new seed=00000000000000000000000000000My1 metadata=did_metadata")
        .finalize());

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, secret!(params));

        let store = ensure_opened_wallet(&ctx)?;

        let did = get_opt_str_param("did", params).map_err(error_err!())?;
        let seed = get_opt_str_param("seed", params).map_err(error_err!())?;
        let method = get_opt_str_param("method", params).map_err(error_err!())?;
        let metadata = get_opt_empty_str_param("metadata", params).map_err(error_err!())?;

        let (did, vk) = Did::create(&store, did, seed, metadata, method)
            .map_err(|err| println_err!("{}", err.message(None)))?;

        let vk = Did::abbreviate_verkey(&did, &vk)
            .map_err(|err| println_err!("{}", err.message(None)))?;

        println_succ!("Did \"{}\" has been created with \"{}\" verkey", did, vk);

        // if let Some(metadata) = metadata {
        //     Did::set_metadata(&store, &did, metadata)
        //         .map_err(|err| println_err!("{}", err.message(None)))?;
        // }

        trace!("execute <<");
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    mod did_new {
        use super::*;
        use crate::did::tests::{
            get_did_info, get_dids, DID_TRUSTEE, SEED_TRUSTEE, VERKEY_TRUSTEE,
        };

        #[test]
        pub fn new_works() {
            let ctx = setup_with_wallet();
            {
                let cmd = new_command::new();
                let params = CommandParams::new();
                cmd.execute(&ctx, &params).unwrap();
            }
            let dids = get_dids(&ctx);
            assert_eq!(1, dids.len());

            tear_down_with_wallet(&ctx);
        }

        #[test]
        pub fn new_works_for_did() {
            let ctx = setup_with_wallet();
            {
                let cmd = new_command::new();
                let mut params = CommandParams::new();
                params.insert("did", DID_TRUSTEE.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            let did = get_did_info(&ctx, DID_TRUSTEE);
            assert_eq!(did.did, DID_TRUSTEE);

            tear_down_with_wallet(&ctx);
        }

        #[test]
        pub fn new_works_for_seed() {
            let ctx = setup_with_wallet();
            {
                let cmd = new_command::new();
                let mut params = CommandParams::new();
                params.insert("seed", SEED_TRUSTEE.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            let did = get_did_info(&ctx, DID_TRUSTEE);
            assert_eq!(did.did, DID_TRUSTEE);
            assert_eq!(did.verkey, VERKEY_TRUSTEE);

            tear_down_with_wallet(&ctx);
        }

        #[test]
        pub fn new_works_for_hex_seed() {
            let ctx = setup_with_wallet();
            {
                let cmd = new_command::new();
                let mut params = CommandParams::new();
                params.insert(
                    "seed",
                    "94a823a6387cdd30d8f7687d95710ebab84c6e277b724790a5b221440beb7df6".to_string(),
                );
                cmd.execute(&ctx, &params).unwrap();
            }
            get_did_info(&ctx, "HWvjYf77k1dqQAk6sE4gaS");

            tear_down_with_wallet(&ctx);
        }

        #[test]
        pub fn new_works_for_meta() {
            let ctx = setup_with_wallet();
            let metadata = "metadata";
            {
                let cmd = new_command::new();
                let mut params = CommandParams::new();
                params.insert("metadata", metadata.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            let dids = get_dids(&ctx);
            assert_eq!(1, dids.len());
            assert_eq!(
                dids.get(0)
                    .as_ref()
                    .unwrap()
                    .metadata
                    .as_ref()
                    .unwrap()
                    .to_string(),
                metadata.to_string()
            );

            tear_down_with_wallet(&ctx);
        }

        #[test]
        pub fn new_works_for_no_opened_wallet() {
            let ctx = setup();
            {
                let cmd = new_command::new();
                let params = CommandParams::new();
                cmd.execute(&ctx, &params).unwrap_err();
            }
            tear_down();
        }

        #[test]
        pub fn new_works_for_wrong_seed() {
            let ctx = setup_with_wallet();
            {
                let cmd = new_command::new();
                let mut params = CommandParams::new();
                params.insert("seed", "invalid_base58_string".to_string());
                cmd.execute(&ctx, &params).unwrap_err();
            }
            tear_down_with_wallet(&ctx);
        }

        #[test]
        pub fn new_works_for_method_name() {
            let ctx = setup_with_wallet();
            let method = "sov";
            {
                let cmd = new_command::new();
                let mut params = CommandParams::new();
                params.insert("seed", SEED_TRUSTEE.to_string());
                params.insert("method", method.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            let expected_did = format!("did:{}:{}", method, DID_TRUSTEE);
            let did = get_did_info(&ctx, &expected_did);
            assert_eq!(did.did, expected_did);
            assert_eq!(did.verkey, VERKEY_TRUSTEE.to_string());

            tear_down_with_wallet(&ctx);
        }

        #[test]
        pub fn new_works_for_not_abbreviatable() {
            let ctx = setup_with_wallet();
            let method = "indy";
            {
                let cmd = new_command::new();
                let mut params = CommandParams::new();
                params.insert("seed", SEED_TRUSTEE.to_string());
                params.insert("method", method.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            let expected_did = format!("did:{}:{}", method, DID_TRUSTEE);
            let did = get_did_info(&ctx, &expected_did);
            assert_eq!(did.did, expected_did);
            assert_eq!(did.verkey, VERKEY_TRUSTEE);

            tear_down_with_wallet(&ctx);
        }
    }
}

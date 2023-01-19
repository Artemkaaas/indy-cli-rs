/*
    Copyright 2023 DSR Corporation, Denver, Colorado.
    https://www.dsr-corporation.com
    SPDX-License-Identifier: Apache-2.0
*/
use crate::{
    command_executor::{Command, CommandContext, CommandMetadata, CommandParams},
    commands::*,
    tools::did::Did,
    utils::table::print_list_table,
};

pub mod list_command {
    use super::*;

    command!(
        CommandMetadata::build("list", "List my DIDs stored in the opened wallet.").finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let store = ensure_opened_wallet(&ctx)?;

        let mut dids = Did::list(&store).map_err(|err| println_err!("{}", err.message(None)))?;

        for did_info in dids.iter_mut() {
            did_info.verkey = Did::abbreviate_verkey(&did_info.did, &did_info.verkey)
                .map_err(|err| println_err!("{}", err.message(None)))?;
        }

        print_list_table(
            &dids
                .iter()
                .map(|did| json!(did))
                .collect::<Vec<serde_json::Value>>(),
            &[
                ("did", "Did"),
                ("verkey", "Verkey"),
                ("metadata", "Metadata"),
            ],
            "There are no dids",
        );
        if let Some(cur_did) = get_active_did(ctx)? {
            println_succ!("Current did \"{}\"", cur_did);
        }

        trace!("execute <<");
        Ok(())
    }
}

pub fn did_list(ctx: &CommandContext) -> Vec<String> {
    get_opened_wallet(ctx)
        .and_then(|(store, _)| Did::list(&store).ok())
        .unwrap_or(vec![])
        .into_iter()
        .map(|did| did.did)
        .collect()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    mod did_list {
        use super::*;
        use crate::{
            did::tests::{new_did, SEED_TRUSTEE},
            wallet::tests::close_and_delete_wallet,
        };

        #[test]
        pub fn list_works() {
            let ctx = setup_with_wallet();
            new_did(&ctx, SEED_TRUSTEE);
            {
                let cmd = list_command::new();
                let params = CommandParams::new();
                cmd.execute(&ctx, &params).unwrap();
            }
            tear_down_with_wallet(&ctx);
        }

        #[test]
        pub fn list_works_for_empty_result() {
            let ctx = setup_with_wallet();
            {
                let cmd = list_command::new();
                let params = CommandParams::new();
                cmd.execute(&ctx, &params).unwrap();
            }
            tear_down_with_wallet(&ctx);
        }

        #[test]
        pub fn list_works_for_closed_wallet() {
            let ctx = setup_with_wallet();
            new_did(&ctx, SEED_TRUSTEE);
            close_and_delete_wallet(&ctx);
            {
                let cmd = list_command::new();
                let params = CommandParams::new();
                cmd.execute(&ctx, &params).unwrap_err();
            }
            tear_down();
        }
    }
}

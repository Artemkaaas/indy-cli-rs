/*
    Copyright 2023 DSR Corporation, Denver, Colorado.
    https://www.dsr-corporation.com
    SPDX-License-Identifier: Apache-2.0
*/
use crate::{
    command_executor::{Command, CommandContext, CommandMetadata, CommandParams},
    commands::*,
};

pub mod show_taa_command {
    use super::*;
    use crate::pool::set_transaction_author_agreement;

    command!(CommandMetadata::build(
        "show-taa",
        "Show transaction author agreement set on Ledger."
    )
    .finalize());

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let pool = ensure_connected_pool_handle(&ctx)?;

        match set_transaction_author_agreement(ctx, &pool, false) {
            Err(_) => (),
            Ok(Some(_)) => (),
            Ok(None) => {
                println!("There is no transaction agreement set on the Pool.");
            }
        };

        trace!("execute <<");
        Ok(())
    }
}

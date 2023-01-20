/*
    Copyright 2023 DSR Corporation, Denver, Colorado.
    https://www.dsr-corporation.com
    SPDX-License-Identifier: Apache-2.0
*/
use crate::{
    command_executor::{Command, CommandContext, CommandMetadata, CommandParams},
    commands::*,
    tools::wallet::Wallet,
    utils::table::print_list_table,
};

pub mod list_command {
    use super::*;

    command!(CommandMetadata::build("list", "List attached wallets.").finalize());

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let wallets = Wallet::list();

        print_list_table(
            &wallets,
            &[("id", "Name"), ("storage_type", "Type")],
            "There are no wallets",
        );

        if let Some((_, cur_wallet)) = get_opened_wallet(ctx) {
            println_succ!("Current wallet \"{}\"", cur_wallet);
        }

        trace!("execute << ");
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    mod list {
        use super::*;

        #[test]
        pub fn list_works() {
            let ctx = setup_with_wallet();
            {
                let cmd = list_command::new();
                let params = CommandParams::new();
                cmd.execute(&ctx, &params).unwrap();
            }
            tear_down_with_wallet(&ctx);
        }

        #[test]
        pub fn list_works_for_empty_list() {
            let ctx = setup();
            {
                let cmd = list_command::new();
                let params = CommandParams::new();
                cmd.execute(&ctx, &params).unwrap();
            }
            tear_down();
        }
    }
}

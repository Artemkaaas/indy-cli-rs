/*
    Copyright 2023 DSR Corporation, Denver, Colorado.
    https://www.dsr-corporation.com
    SPDX-License-Identifier: Apache-2.0
*/
use crate::{
    command_executor::{Command, CommandContext, CommandMetadata, CommandParams},
    commands::*,
    tools::pool::Pool,
};

pub mod disconnect_command {
    use super::*;

    command!(CommandMetadata::build("disconnect", "Disconnect from current pool.").finalize());

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let pool = ensure_connected_pool(&ctx)?;
        let name = ensure_connected_pool_name(&ctx)?;

        close_pool(ctx, &pool, &name)?;

        trace!("execute <<");
        Ok(())
    }
}

pub fn close_pool(ctx: &CommandContext, pool: &LocalPool, name: &str) -> Result<(), ()> {
    Pool::close(pool)
        .map(|_| {
            set_connected_pool(ctx, None);
            set_transaction_author_info(ctx, None);
            println_succ!("Pool \"{}\" has been disconnected", name)
        })
        .map_err(|err| println_err!("{}", err.message(Some(&name))))
}

#[cfg(test)]
pub mod tests {
    use super::*;

    mod disconnect {
        use super::*;
        use crate::pool::tests::{create_and_connect_pool, create_pool, delete_pool};

        #[test]
        pub fn disconnect_works_for_not_opened() {
            let ctx = setup();
            create_pool(&ctx);
            {
                let cmd = disconnect_command::new();
                let params = CommandParams::new();
                cmd.execute(&ctx, &params).unwrap_err();
            }
            delete_pool(&ctx);
            tear_down();
        }

        #[test]
        pub fn disconnect_works_for_twice() {
            let ctx = setup();
            create_and_connect_pool(&ctx);
            {
                let cmd = disconnect_command::new();
                let params = CommandParams::new();
                cmd.execute(&ctx, &params).unwrap();
            }
            {
                let cmd = disconnect_command::new();
                let params = CommandParams::new();
                cmd.execute(&ctx, &params).unwrap_err();
            }
            delete_pool(&ctx);
            tear_down();
        }
    }
}

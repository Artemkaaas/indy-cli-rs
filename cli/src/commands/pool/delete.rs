/*
    Copyright 2023 DSR Corporation, Denver, Colorado.
    https://www.dsr-corporation.com
    SPDX-License-Identifier: Apache-2.0
*/
use crate::{
    command_executor::{
        Command, CommandContext, CommandMetadata, CommandParams, DynamicCompletionType,
    },
    commands::*,
    tools::pool::Pool,
};

pub mod delete_command {
    use super::*;
    use crate::pool::close_pool;

    command!(
        CommandMetadata::build("delete", "Delete pool config with specified name")
            .add_main_param_with_dynamic_completion(
                "name",
                "The name of deleted pool config",
                DynamicCompletionType::Pool
            )
            .add_example("pool delete pool1")
            .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let name = get_str_param("name", params).map_err(error_err!())?;

        trace!(r#"Pool::delete try: name {}"#, name);

        if let Some((pool, name)) = get_connected_pool_with_name(ctx) {
            close_pool(ctx, &pool, &name)?;
        }

        Pool::delete(name).map_err(|err| println_err!("{}", err.message(Some(&name))))?;

        println_succ!("Pool \"{}\" has been deleted.", name);

        trace!("execute <<");
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    mod delete {
        use super::*;
        use crate::pool::tests::{create_and_connect_pool, create_pool, get_pools, POOL};

        #[test]
        pub fn delete_works() {
            let ctx = setup();
            create_pool(&ctx);
            {
                let cmd = delete_command::new();
                let mut params = CommandParams::new();
                params.insert("name", POOL.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            let pools = get_pools();
            assert_eq!(0, pools.len());

            tear_down();
        }

        #[test]
        pub fn delete_works_for_not_created() {
            let ctx = setup();

            let cmd = delete_command::new();
            let mut params = CommandParams::new();
            params.insert("name", POOL.to_string());
            cmd.execute(&ctx, &params).unwrap_err();

            tear_down();
        }

        #[test]
        pub fn delete_works_for_connected() {
            let ctx = setup();
            create_and_connect_pool(&ctx);
            {
                let cmd = delete_command::new();
                let mut params = CommandParams::new();
                params.insert("name", POOL.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            tear_down();
        }
    }
}

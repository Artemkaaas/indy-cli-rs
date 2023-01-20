/*
    Copyright 2023 DSR Corporation, Denver, Colorado.
    https://www.dsr-corporation.com
    SPDX-License-Identifier: Apache-2.0
*/
use crate::{
    command_executor::{Command, CommandContext, CommandMetadata, CommandParams},
    commands::*,
    tools::ledger::{Ledger, Response},
};

use indy_vdr::ledger::{
    identifiers::SchemaId,
    requests::schema::{AttributeNames, Schema, SchemaV1},
};
use serde_json::Value as JsonValue;

use super::common::{
    handle_transaction_response, print_transaction_response, set_author_agreement,
};

pub mod schema_command {
    use super::*;

    command!(CommandMetadata::build("schema", r#"Send Schema transaction to the Ledger."#)
                .add_required_param("name", "Schema name")
                .add_required_param("version", "Schema version")
                .add_required_param("attr_names", "Schema attributes split by comma (the number of attributes should be less or equal than 125)")
                .add_optional_param("sign","Sign the request (True by default)")
                .add_optional_param("send","Send the request to the Ledger (True by default). If false then created request will be printed and stored into CLI context.")
                .add_optional_param("endorser","DID of the Endorser that will submit the transaction to the ledger later. \
                    Note that specifying of this parameter implies send=false so the transaction will be prepared to pass to the endorser instead of sending to the ledger.\
                    The created request will be printed and stored into CLI context.")
                .add_example("ledger schema name=gvt version=1.0 attr_names=name,age")
                .add_example("ledger schema name=gvt version=1.0 attr_names=name,age send=false")
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let store = ensure_opened_wallet(&ctx)?;
        let submitter_did = ensure_active_did(&ctx)?;
        let pool = get_connected_pool(&ctx);

        let name = get_str_param("name", params).map_err(error_err!())?;
        let version = get_str_param("version", params).map_err(error_err!())?;
        let attr_names = get_str_array_param("attr_names", params).map_err(error_err!())?;

        let id = SchemaId::new(&submitter_did, name, version);
        let schema = Schema::SchemaV1(SchemaV1 {
            id,
            name: name.to_string(),
            version: version.to_string(),
            attr_names: AttributeNames::from(attr_names.as_slice()),
            seq_no: None,
        });

        let mut request = Ledger::build_schema_request(pool.as_deref(), &submitter_did, schema)
            .map_err(|err| println_err!("{}", err.message(None)))?;

        set_author_agreement(ctx, &mut request)?;

        let (_, response): (String, Response<JsonValue>) = send_write_request!(
            ctx,
            params,
            &mut request,
            &store,
            &wallet_name,
            &submitter_did
        );

        handle_transaction_response(response).map(|result| {
            print_transaction_response(
                result,
                "Schema request has been sent to Ledger.",
                Some("data"),
                &[
                    ("name", "Name"),
                    ("version", "Version"),
                    ("attr_names", "Attributes"),
                ],
                true,
            )
        })?;

        trace!("execute <<");
        Ok(())
    }
}

pub mod get_schema_command {
    use super::*;

    command!(CommandMetadata::build("get-schema", "Get Schema from Ledger.")
                .add_required_param("did", "DID of identity presented in Ledger")
                .add_required_param("name", "Schema name")
                .add_required_param("version", "Schema version")
                .add_optional_param("send","Send the request to the Ledger (True by default). If false then created request will be printed and stored into CLI context.")
                .add_example("ledger get-schema did=VsKV7grR1BUE29mG2Fm2kX name=gvt version=1.0")
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let submitter_did = get_active_did(&ctx)?;
        let pool = get_connected_pool(&ctx);

        let target_did = get_did_param("did", params).map_err(error_err!())?;
        let name = get_str_param("name", params).map_err(error_err!())?;
        let version = get_str_param("version", params).map_err(error_err!())?;

        let id = SchemaId::new(&target_did, name, version);

        let request =
            Ledger::build_get_schema_request(pool.as_deref(), submitter_did.as_ref(), &id)
                .map_err(|err| println_err!("{}", err.message(None)))?;

        let (_, response) = send_read_request!(&ctx, params, &request, submitter_did.as_ref());

        if let Some(result) = response.result.as_ref() {
            if !result["seqNo"].is_i64() {
                println_err!("Schema not found");
                return Err(());
            }
        };

        handle_transaction_response(response).map(|result| {
            print_transaction_response(
                result,
                "Following Schema has been received.",
                Some("data"),
                &[
                    ("name", "Name"),
                    ("version", "Version"),
                    ("attr_names", "Attributes"),
                ],
                true,
            )
        })?;

        trace!("execute <<");
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        commands::{
            did::tests::{new_did, use_did, DID_MY3, DID_TRUSTEE, SEED_MY3},
            wallet::tests::{close_wallet, open_wallet},
        },
        ledger::{
            endorse_transaction_command,
            tests::{create_new_did, ensure_schema_added, send_nym, use_new_identity, use_trustee},
        },
    };

    mod schema {
        use super::*;

        #[test]
        pub fn schema_works() {
            let ctx = setup_with_wallet_and_pool();
            let (did, _) = use_new_identity(&ctx);
            {
                let cmd = schema_command::new();
                let mut params = CommandParams::new();
                params.insert("name", "gvt".to_string());
                params.insert("version", "1.0".to_string());
                params.insert("attr_names", "name,age".to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            assert!(ensure_schema_added(&ctx, &did).is_ok());
            tear_down_with_wallet_and_pool(&ctx);
        }

        #[test]
        pub fn schema_works_for_missed_required_params() {
            let ctx = setup_with_wallet_and_pool();
            use_trustee(&ctx);
            {
                let cmd = schema_command::new();
                let mut params = CommandParams::new();
                params.insert("name", "gvt".to_string());
                cmd.execute(&ctx, &params).unwrap_err();
            }
            tear_down_with_wallet_and_pool(&ctx);
        }

        #[test]
        pub fn schema_works_unknown_submitter() {
            let ctx = setup_with_wallet_and_pool();
            new_did(&ctx, SEED_MY3);
            use_did(&ctx, DID_MY3);
            {
                let cmd = schema_command::new();
                let mut params = CommandParams::new();
                params.insert("name", "gvt".to_string());
                params.insert("version", "1.0".to_string());
                params.insert("attr_names", "name,age".to_string());
                cmd.execute(&ctx, &params).unwrap_err();
            }
            tear_down_with_wallet_and_pool(&ctx);
        }

        #[test]
        pub fn schema_works_for_no_active_did() {
            let ctx = setup_with_wallet_and_pool();
            {
                let cmd = schema_command::new();
                let mut params = CommandParams::new();
                params.insert("name", "gvt".to_string());
                params.insert("version", "1.0".to_string());
                params.insert("attr_names", "name,age".to_string());
                cmd.execute(&ctx, &params).unwrap_err();
            }
            tear_down_with_wallet_and_pool(&ctx);
        }

        #[test]
        pub fn schema_works_without_sending() {
            let ctx = setup_with_wallet_and_pool();
            let (did, _) = use_new_identity(&ctx);
            {
                let cmd = schema_command::new();
                let mut params = CommandParams::new();
                params.insert("name", "gvt".to_string());
                params.insert("version", "1.0".to_string());
                params.insert("attr_names", "name,age".to_string());
                params.insert("send", "false".to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            assert!(ensure_schema_added(&ctx, &did).is_err());
            assert!(get_context_transaction(&ctx).is_some());
            tear_down_with_wallet_and_pool(&ctx);
        }

        #[test]
        pub fn schema_works_without_signing() {
            let ctx = setup_with_wallet_and_pool();
            use_new_identity(&ctx);
            {
                let cmd = schema_command::new();
                let mut params = CommandParams::new();
                params.insert("name", "gvt".to_string());
                params.insert("version", "1.0".to_string());
                params.insert("attr_names", "name,age".to_string());
                params.insert("sign", "false".to_string());
                params.insert("send", "false".to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            let transaction = get_context_transaction(&ctx).unwrap();
            let transaction: JsonValue = serde_json::from_str(&transaction).unwrap();
            assert!(transaction["signature"].is_null());
            tear_down_with_wallet_and_pool(&ctx);
        }

        #[test]
        pub fn schema_works_for_endorser() {
            let ctx = setup_with_wallet_and_pool();
            let (endorser_did, _) = use_new_identity(&ctx);

            // Publish new NYM without any role
            let (did, verkey) = create_new_did(&ctx);
            send_nym(&ctx, &did, &verkey, None);
            use_did(&ctx, &did);

            {
                let cmd = schema_command::new();
                let mut params = CommandParams::new();
                params.insert("name", "gvt".to_string());
                params.insert("version", "1.0".to_string());
                params.insert("attr_names", "name,age".to_string());
                params.insert("endorser", endorser_did.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            use_did(&ctx, &endorser_did);
            {
                let cmd = endorse_transaction_command::new();
                let params = CommandParams::new();
                cmd.execute(&ctx, &params).unwrap();
            }
            assert!(ensure_schema_added(&ctx, &did).is_ok());
            tear_down_with_wallet_and_pool(&ctx);
        }
    }

    mod get_schema {
        use super::*;

        #[test]
        pub fn get_schema_works() {
            let ctx = setup_with_wallet_and_pool();
            let (did, _) = use_new_identity(&ctx);
            {
                let cmd = schema_command::new();
                let mut params = CommandParams::new();
                params.insert("name", "gvt".to_string());
                params.insert("version", "1.0".to_string());
                params.insert("attr_names", "name,age".to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            assert!(ensure_schema_added(&ctx, &did).is_ok());
            {
                let cmd = get_schema_command::new();
                let mut params = CommandParams::new();
                params.insert("did", did);
                params.insert("name", "gvt".to_string());
                params.insert("version", "1.0".to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            tear_down_with_wallet_and_pool(&ctx);
        }

        #[test]
        pub fn schema_works_for_unknown_schema() {
            let ctx = setup_with_wallet_and_pool();
            use_trustee(&ctx);
            {
                let cmd = get_schema_command::new();
                let mut params = CommandParams::new();
                params.insert("did", DID_TRUSTEE.to_string());
                params.insert("name", "unknown_schema_name".to_string());
                params.insert("version", "1.0".to_string());
                cmd.execute(&ctx, &params).unwrap_err();
            }
            tear_down_with_wallet_and_pool(&ctx);
        }

        #[test]
        pub fn schema_works_for_unknown_submitter() {
            let ctx = setup_with_wallet_and_pool();
            new_did(&ctx, SEED_MY3);
            use_did(&ctx, DID_MY3);
            {
                let cmd = get_schema_command::new();
                let mut params = CommandParams::new();
                params.insert("did", DID_MY3.to_string());
                params.insert("name", "gvt".to_string());
                params.insert("version", "1.0".to_string());
                cmd.execute(&ctx, &params).unwrap_err();
            }
            tear_down_with_wallet_and_pool(&ctx);
        }

        #[test]
        pub fn schema_works_for_no_active_did() {
            let ctx = setup_with_wallet_and_pool();
            let (did, _) = use_new_identity(&ctx);
            {
                let cmd = schema_command::new();
                let mut params = CommandParams::new();
                params.insert("name", "gvt".to_string());
                params.insert("version", "1.0".to_string());
                params.insert("attr_names", "name,age".to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            assert!(ensure_schema_added(&ctx, &did).is_ok());

            // to reset active did
            close_wallet(&ctx);
            open_wallet(&ctx);

            {
                let cmd = get_schema_command::new();
                let mut params = CommandParams::new();
                params.insert("did", did);
                params.insert("name", "gvt".to_string());
                params.insert("version", "1.0".to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            tear_down_with_wallet_and_pool(&ctx);
        }
    }
}

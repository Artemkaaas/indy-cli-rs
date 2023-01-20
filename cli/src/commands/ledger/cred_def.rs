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
    identifiers::{CredentialDefinitionId, SchemaId},
    requests::cred_def::{
        CredentialDefinition, CredentialDefinitionData, CredentialDefinitionV1, SignatureType,
    },
};
use serde_json::Value as JsonValue;

use super::common::{
    handle_transaction_response, print_transaction_response, set_author_agreement,
};

pub mod cred_def_command {
    use super::*;

    command!(CommandMetadata::build("cred-def", r#"Send Cred Def transaction to the Ledger."#)
                .add_required_param("schema_id", "Sequence number of schema")
                .add_required_param("signature_type", "Signature type (only CL supported now)")
                .add_optional_param("tag", "Allows to distinct between credential definitions for the same issuer and schema. Note that it is mandatory for indy-node version 1.4.x and higher")
                .add_required_param("primary", "Primary key in json format")
                .add_optional_param("revocation", "Revocation key in json format")
                .add_optional_param("sign","Sign the request (True by default)")
                .add_optional_param("send","Send the request to the Ledger (True by default). If false then created request will be printed and stored into CLI context.")
                .add_optional_param("endorser","DID of the Endorser that will submit the transaction to the ledger later. \
                    Note that specifying of this parameter implies send=false so the transaction will be prepared to pass to the endorser instead of sending to the ledger.\
                    The created request will be printed and stored into CLI context.")
                .add_example(r#"ledger cred-def schema_id=1 signature_type=CL tag=1 primary={"n":"1","s":"2","rms":"3","r":{"age":"4","name":"5"},"rctxt":"6","z":"7"}"#)
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let store = ensure_opened_wallet(&ctx)?;
        let submitter_did = ensure_active_did(&ctx)?;
        let pool = get_connected_pool(&ctx);

        let schema_id = get_str_param("schema_id", params).map_err(error_err!())?;
        let signature_type = get_str_param("signature_type", params).map_err(error_err!())?;
        let tag = get_opt_str_param("tag", params)
            .map_err(error_err!())?
            .unwrap_or("");

        let primary = get_object_param("primary", params).map_err(error_err!())?;
        let revocation = get_opt_object_param("revocation", params).map_err(error_err!())?;

        let schema_id = SchemaId::from(schema_id.to_string());
        let id = CredentialDefinitionId::new(&submitter_did, &schema_id, signature_type, tag);

        let signature_type = SignatureType::from_str(signature_type)
            .map_err(|_| println_err!("Unsupported signature_type {}", signature_type))?;

        let cred_def = CredentialDefinition::CredentialDefinitionV1(CredentialDefinitionV1 {
            id,
            schema_id,
            signature_type,
            tag: tag.to_string(),
            value: CredentialDefinitionData {
                primary,
                revocation,
            },
        });

        let mut request = Ledger::build_cred_def_request(pool.as_deref(), &submitter_did, cred_def)
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
                "NodeConfig request has been sent to Ledger.",
                Some("data"),
                &[("primary", "Primary Key"), ("revocation", "Revocation Key")],
                true,
            )
        })?;

        trace!("execute <<");
        Ok(())
    }
}

pub mod get_cred_def_command {
    use super::*;

    command!(CommandMetadata::build("get-cred-def", "Get Cred Definition from Ledger.")
                .add_required_param("schema_id", "Sequence number of schema")
                .add_required_param("signature_type", "Signature type (only CL supported now)")
                .add_optional_param("tag", "Allows to distinct between credential definitions for the same issuer and schema. Note that it is mandatory for indy-node version 1.4.x and higher")
                .add_required_param("origin", "Credential definition owner DID")
                .add_optional_param("send","Send the request to the Ledger (True by default). If false then created request will be printed and stored into CLI context.")
                .add_example("ledger get-cred-def schema_id=1 signature_type=CL tag=1 origin=VsKV7grR1BUE29mG2Fm2kX")
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let submitter_did = get_active_did(&ctx)?;
        let pool = get_connected_pool(&ctx);

        let schema_id = get_str_param("schema_id", params).map_err(error_err!())?;
        let signature_type = get_str_param("signature_type", params).map_err(error_err!())?;
        let tag = get_opt_str_param("tag", params)
            .map_err(error_err!())?
            .unwrap_or("");
        let origin = get_did_param("origin", params).map_err(error_err!())?;

        let schema_id = SchemaId::from(schema_id.to_string());
        let id = CredentialDefinitionId::new(&origin, &schema_id, signature_type, tag);

        let request =
            Ledger::build_get_cred_def_request(pool.as_deref(), submitter_did.as_ref(), &id)
                .map_err(|err| println_err!("{}", err.message(None)))?;

        let (_, response) = send_read_request!(&ctx, params, &request, submitter_did.as_ref());

        if let Some(result) = response.result.as_ref() {
            if !result["seqNo"].is_i64() {
                println_err!("Credential Definition not found");
                return Err(());
            }
        };

        handle_transaction_response(response).map(|result| {
            print_transaction_response(
                result,
                "Following Credential Definition has been received.",
                Some("data"),
                &[("primary", "Primary Key"), ("revocation", "Revocation Key")],
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
            did::tests::{new_did, use_did, DID_MY3, SEED_MY3},
            wallet::tests::{close_wallet, open_wallet},
        },
        ledger::tests::{
            ensure_cred_def_added, send_schema, use_new_identity, use_trustee, CRED_DEF_DATA,
        },
    };

    mod cred_def {
        use super::*;

        #[test]
        pub fn cred_def_works() {
            let ctx = setup_with_wallet_and_pool();
            let (did, _) = use_new_identity(&ctx);
            let schema_id = send_schema(&ctx, &did);
            {
                let cmd = cred_def_command::new();
                let mut params = CommandParams::new();
                params.insert("schema_id", schema_id.clone());
                params.insert("signature_type", "CL".to_string());
                params.insert("tag", "TAG".to_string());
                params.insert("primary", CRED_DEF_DATA.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            assert!(ensure_cred_def_added(&ctx, &did, &schema_id).is_ok());
            tear_down_with_wallet_and_pool(&ctx);
        }

        #[test]
        pub fn cred_def_works_for_missed_required_params() {
            let ctx = setup_with_wallet_and_pool();
            use_trustee(&ctx);
            {
                let cmd = cred_def_command::new();
                let mut params = CommandParams::new();
                params.insert("schema_id", "1".to_string());
                cmd.execute(&ctx, &params).unwrap_err();
            }
            tear_down_with_wallet_and_pool(&ctx);
        }

        #[test]
        pub fn cred_def_works_for_unknown_submitter() {
            let ctx = setup_with_wallet_and_pool();
            new_did(&ctx, SEED_MY3);
            use_did(&ctx, DID_MY3);
            {
                let cmd = cred_def_command::new();
                let mut params = CommandParams::new();
                params.insert("schema_id", "1".to_string());
                params.insert("signature_type", "CL".to_string());
                params.insert("tag", "TAG".to_string());
                params.insert("primary", CRED_DEF_DATA.to_string());
                cmd.execute(&ctx, &params).unwrap_err();
            }
            tear_down_with_wallet_and_pool(&ctx);
        }

        #[test]
        pub fn cred_def_works_for_no_active_did() {
            let ctx = setup_with_wallet_and_pool();
            {
                let cmd = cred_def_command::new();
                let mut params = CommandParams::new();
                params.insert("schema_id", "1".to_string());
                params.insert("signature_type", "CL".to_string());
                params.insert("tag", "TAG".to_string());
                params.insert("primary", CRED_DEF_DATA.to_string());
                cmd.execute(&ctx, &params).unwrap_err();
            }
            tear_down_with_wallet_and_pool(&ctx);
        }

        #[test]
        pub fn cred_def_works_without_sending() {
            let ctx = setup_with_wallet_and_pool();
            let (did, _) = use_new_identity(&ctx);
            let schema_id = send_schema(&ctx, &did);
            {
                let cmd = cred_def_command::new();
                let mut params = CommandParams::new();
                params.insert("schema_id", schema_id.clone());
                params.insert("signature_type", "CL".to_string());
                params.insert("tag", "TAG".to_string());
                params.insert("primary", CRED_DEF_DATA.to_string());
                params.insert("send", "false".to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            assert!(ensure_cred_def_added(&ctx, &did, &schema_id).is_err());
            assert!(get_context_transaction(&ctx).is_some());
            tear_down_with_wallet_and_pool(&ctx);
        }
    }

    mod get_cred_def {
        use super::*;

        #[test]
        pub fn get_cred_def_works() {
            let ctx = setup_with_wallet_and_pool();
            let (did, _) = use_new_identity(&ctx);
            let schema_id = send_schema(&ctx, &did);
            {
                let cmd = cred_def_command::new();
                let mut params = CommandParams::new();
                params.insert("schema_id", schema_id.clone());
                params.insert("signature_type", "CL".to_string());
                params.insert("tag", "TAG".to_string());
                params.insert("primary", CRED_DEF_DATA.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            assert!(ensure_cred_def_added(&ctx, &did, &schema_id).is_ok());
            {
                let cmd = get_cred_def_command::new();
                let mut params = CommandParams::new();
                params.insert("schema_id", schema_id);
                params.insert("signature_type", "CL".to_string());
                params.insert("tag", "TAG".to_string());
                params.insert("origin", did.clone());
                cmd.execute(&ctx, &params).unwrap();
            }
            tear_down_with_wallet_and_pool(&ctx);
        }

        #[test]
        pub fn get_cred_def_works_for_unknown_cred_def() {
            let ctx = setup_with_wallet_and_pool();
            use_trustee(&ctx);
            {
                let cmd = get_cred_def_command::new();
                let mut params = CommandParams::new();
                params.insert("schema_id", "2".to_string());
                params.insert("signature_type", "CL".to_string());
                params.insert("tag", "TAG".to_string());
                params.insert("origin", DID_MY3.to_string());
                cmd.execute(&ctx, &params).unwrap_err();
            }
            tear_down_with_wallet_and_pool(&ctx);
        }

        #[test]
        pub fn get_cred_def_works_for_no_active_did() {
            let ctx = setup_with_wallet_and_pool();
            let (did, _) = use_new_identity(&ctx);
            let schema_id = send_schema(&ctx, &did);
            {
                let cmd = cred_def_command::new();
                let mut params = CommandParams::new();
                params.insert("schema_id", schema_id.clone());
                params.insert("signature_type", "CL".to_string());
                params.insert("tag", "TAG".to_string());
                params.insert("primary", CRED_DEF_DATA.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            assert!(ensure_cred_def_added(&ctx, &did, &schema_id).is_ok());

            // to reset active did
            close_wallet(&ctx);
            open_wallet(&ctx);

            {
                let cmd = get_cred_def_command::new();
                let mut params = CommandParams::new();
                params.insert("schema_id", schema_id);
                params.insert("signature_type", "CL".to_string());
                params.insert("tag", "TAG".to_string());
                params.insert("origin", did.clone());
                cmd.execute(&ctx, &params).unwrap();
            }
            tear_down_with_wallet_and_pool(&ctx);
        }
    }
}

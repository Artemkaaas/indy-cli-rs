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

use indy_vdr::ledger::requests::node::{NodeOperationData, Services};
use serde_json::Value as JsonValue;

use super::common::{handle_transaction_response, print_transaction_response};

pub mod node_command {
    use super::*;

    command!(CommandMetadata::build("node", "Send Node transaction to the Ledger.")
                .add_required_param("target", "Node identifier")
                .add_required_param("alias", "Node alias (can't be changed in case of update)")
                .add_optional_param("node_ip", "Node Ip. Note that it is mandatory for adding node case")
                .add_optional_param("node_port", "Node port. Note that it is mandatory for adding node case")
                .add_optional_param("client_ip", "Client Ip. Note that it is mandatory for adding node case")
                .add_optional_param("client_port","Client port. Note that it is mandatory for adding node case")
                .add_optional_param("blskey",  "Node BLS key")
                .add_optional_param("blskey_pop",  "Node BLS key proof of possession. Note that it is mandatory if blskey specified")
                .add_optional_param("services", "Node type. One of: VALIDATOR, OBSERVER or empty in case of blacklisting node")
                .add_optional_param("sign","Sign the request (True by default)")
                .add_optional_param("send","Send the request to the Ledger (True by default). If false then created request will be printed and stored into CLI context.")
                .add_example("ledger node target=A5iWQVT3k8Zo9nXj4otmeqaUziPQPCiDqcydXkAJBk1Y node_ip=127.0.0.1 node_port=9710 client_ip=127.0.0.1 client_port=9711 alias=Node5 services=VALIDATOR blskey=2zN3bHM1m4rLz54MJHYSwvqzPchYp8jkHswveCLAEJVcX6Mm1wHQD1SkPYMzUDTZvWvhuE6VNAkK3KxVeEmsanSmvjVkReDeBEMxeDaayjcZjFGPydyey1qxBHmTvAnBKoPydvuTAqx5f7YNNRAdeLmUi99gERUU7TD8KfAa6MpQ9bw blskey_pop=RPLagxaR5xdimFzwmzYnz4ZhWtYQEj8iR5ZU53T2gitPCyCHQneUn2Huc4oeLd2B2HzkGnjAff4hWTJT6C7qHYB1Mv2wU5iHHGFWkhnTX9WsEAbunJCV2qcaXScKj4tTfvdDKfLiVuU2av6hbsMztirRze7LvYBkRHV3tGwyCptsrP")
                .add_example("ledger node target=A5iWQVT3k8Zo9nXj4otmeqaUziPQPCiDqcydXkAJBk1Y node_ip=127.0.0.1 node_port=9710 client_ip=127.0.0.1 client_port=9711 alias=Node5 services=VALIDATOR")
                .add_example("ledger node target=A5iWQVT3k8Zo9nXj4otmeqaUziPQPCiDqcydXkAJBk1Y alias=Node5 services=VALIDATOR")
                .add_example("ledger node target=A5iWQVT3k8Zo9nXj4otmeqaUziPQPCiDqcydXkAJBk1Y alias=Node5 services=")
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let store = ensure_opened_wallet(&ctx)?;
        let submitter_did = ensure_active_did(&ctx)?;
        let pool = get_connected_pool(&ctx);

        let target_did = get_did_param("target", params).map_err(error_err!())?;
        let alias = get_str_param("alias", params).map_err(error_err!())?;
        let node_ip = get_opt_str_param("node_ip", params).map_err(error_err!())?;
        let node_port = get_opt_number_param::<i32>("node_port", params).map_err(error_err!())?;
        let client_ip = get_opt_str_param("client_ip", params).map_err(error_err!())?;
        let client_port =
            get_opt_number_param::<i32>("client_port", params).map_err(error_err!())?;
        let blskey = get_opt_str_param("blskey", params).map_err(error_err!())?;
        let blskey_pop = get_opt_str_param("blskey_pop", params).map_err(error_err!())?;
        let services = get_opt_str_array_param("services", params).map_err(error_err!())?;

        let services = match services {
            Some(services) => Some(
                services
                    .into_iter()
                    .map(|service| match service {
                        "VALIDATOR" => Ok(Services::VALIDATOR),
                        "OBSERVER" => Ok(Services::OBSERVER),
                        service => {
                            println_err!("Unsupported service \"{}\"!", service);
                            Err(())
                        }
                    })
                    .collect::<Result<Vec<Services>, ()>>()?,
            ),
            None => None,
        };

        let node_data = NodeOperationData {
            node_ip: node_ip.map(String::from),
            node_port,
            client_ip: client_ip.map(String::from),
            client_port,
            alias: alias.to_string(),
            services,
            blskey: blskey.map(String::from),
            blskey_pop: blskey_pop.map(String::from),
        };

        let mut request =
            Ledger::build_node_request(pool.as_deref(), &submitter_did, &target_did, node_data)
                .map_err(|err| println_err!("{}", err.message(None)))?;

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
                &[
                    ("alias", "Alias"),
                    ("node_ip", "Node Ip"),
                    ("node_port", "Node Port"),
                    ("client_ip", "Client Ip"),
                    ("client_port", "Client Port"),
                    ("services", "Services"),
                    ("blskey", "Blskey"),
                    ("blskey_pop", "Blskey Proof of Possession"),
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
        commands::did::tests::use_did,
        ledger::tests::{create_new_did, send_nym, use_trustee},
    };

    mod node {
        use super::*;

        #[test]
        #[ignore] //TODO: FIXME currently unstable pool behaviour after new non-existing node was added
        pub fn node_works() {
            let ctx = setup_with_wallet_and_pool();
            use_trustee(&ctx);
            let (_did, my_verkey) = create_new_did(&ctx);
            send_nym(&ctx, &_did, &my_verkey, Some("STEWARD"));
            use_did(&ctx, &_did);
            {
                let cmd = node_command::new();
                let mut params = CommandParams::new();
                params.insert(
                    "target",
                    "A5iWQVT3k8Zo9nXj4otmeqaUziPQPCiDqcydXkAJBk1Y".to_string(),
                );
                params.insert("node_ip", "127.0.0.1".to_string());
                params.insert("node_port", "9710".to_string());
                params.insert("client_ip", "127.0.0.2".to_string());
                params.insert("client_port", "9711".to_string());
                params.insert("alias", "Node5".to_string());
                params.insert("blskey", "2zN3bHM1m4rLz54MJHYSwvqzPchYp8jkHswveCLAEJVcX6Mm1wHQD1SkPYMzUDTZvWvhuE6VNAkK3KxVeEmsanSmvjVkReDeBEMxeDaayjcZjFGPydyey1qxBHmTvAnBKoPydvuTAqx5f7YNNRAdeLmUi99gERUU7TD8KfAa6MpQ9bw".to_string());
                params.insert("blskey_pop", "RPLagxaR5xdimFzwmzYnz4ZhWtYQEj8iR5ZU53T2gitPCyCHQneUn2Huc4oeLd2B2HzkGnjAff4hWTJT6C7qHYB1Mv2wU5iHHGFWkhnTX9WsEAbunJCV2qcaXScKj4tTfvdDKfLiVuU2av6hbsMztirRze7LvYBkRHV3tGwyCptsrP".to_string());
                params.insert("services", "VALIDATOR".to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            tear_down_with_wallet_and_pool(&ctx);
        }
    }
}

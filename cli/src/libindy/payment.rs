use super::ErrorCode;

use libc::c_char;
use std::ffi::CString;
use std::ptr::null;

pub struct Payment {}

impl Payment {
    pub fn create_payment_address(wallet_handle: i32, payment_method: &str, config: &str) -> Result<String, ErrorCode> {
        let (receiver, command_handle, cb) = super::callbacks::_closure_to_cb_ec_string();

        let payment_method = CString::new(payment_method).unwrap();
        let config = CString::new(config).unwrap();

        let err = unsafe {
            indy_create_payment_address(command_handle,
                                        wallet_handle,
                                        payment_method.as_ptr(),
                                        config.as_ptr(),
                                        cb)
        };

        super::results::result_to_string(err, receiver)
    }

    pub fn list_addresses(wallet_handle: i32) -> Result<String, ErrorCode> {
        let (receiver, command_handle, cb) = super::callbacks::_closure_to_cb_ec_string();

        let err = unsafe {
            indy_list_addresses(command_handle,
                                wallet_handle,
                                cb)
        };

        super::results::result_to_string(err, receiver)
    }

    pub fn build_get_utxo_request(payment_address: &str) -> Result<(String, String), ErrorCode> {
        let (receiver, command_handle, cb) =
            super::callbacks::_closure_to_cb_ec_string_string();

        let payment_address = CString::new(payment_address).unwrap();

        let err = unsafe {
            indy_build_get_utxo_request(command_handle,
                                        payment_address.as_ptr(),
                                        cb)
        };

        super::results::result_to_string_string(err, receiver)
    }


    pub fn parse_get_utxo_response(payment_method: &str, resp_json: &str) -> Result<String, ErrorCode> {
        let (receiver, command_handle, cb) =
            super::callbacks::_closure_to_cb_ec_string();

        let payment_method = CString::new(payment_method).unwrap();
        let resp_json = CString::new(resp_json).unwrap();

        let err = unsafe {
            indy_parse_get_utxo_response(command_handle,
                                         payment_method.as_ptr(),
                                         resp_json.as_ptr(),
                                         cb)
        };

        super::results::result_to_string(err, receiver)
    }

    pub fn build_mint_req(outputs_json: &str) -> Result<(String, String), ErrorCode> {
        let (receiver, command_handle, cb) =
            super::callbacks::_closure_to_cb_ec_string_string();

        let outputs_json = CString::new(outputs_json).unwrap();

        let err = unsafe {
            indy_build_mint_req(command_handle,
                                outputs_json.as_ptr(),
                                cb)
        };

        super::results::result_to_string_string(err, receiver)
    }

    pub fn build_set_txn_fees_req(payment_method: &str, fees_json: &str) -> Result<String, ErrorCode> {
        let (receiver, command_handle, cb) =
            super::callbacks::_closure_to_cb_ec_string();

        let payment_method = CString::new(payment_method).unwrap();
        let fees_json = CString::new(fees_json).unwrap();

        let err = unsafe {
            indy_build_set_txn_fees_req(command_handle,
                                        payment_method.as_ptr(),
                                        fees_json.as_ptr(),
                                        cb)
        };

        super::results::result_to_string(err, receiver)
    }

    pub fn build_get_txn_fees_req(payment_method: &str) -> Result<String, ErrorCode> {
        let (receiver, command_handle, cb) =
            super::callbacks::_closure_to_cb_ec_string();

        let payment_method = CString::new(payment_method).unwrap();

        let err = unsafe {
            indy_build_get_txn_fees_req(command_handle,
                                        payment_method.as_ptr(),
                                        cb)
        };

        super::results::result_to_string(err, receiver)
    }
}

extern {
    #[no_mangle]
    fn indy_create_payment_address(command_handle: i32,
                                   wallet_handle: i32,
                                   payment_method: *const c_char,
                                   config: *const c_char,
                                   cb: Option<extern fn(command_handle_: i32,
                                                        err: ErrorCode,
                                                        payment_address: *const c_char)>) -> ErrorCode;

    #[no_mangle]
    fn indy_list_addresses(command_handle: i32,
                           wallet_handle: i32,
                           cb: Option<extern fn(command_handle_: i32,
                                                err: ErrorCode,
                                                payment_addresses_json: *const c_char)>) -> ErrorCode;

    #[no_mangle]
    fn indy_build_get_utxo_request(command_handle: i32,
                                   payment_address: *const c_char,
                                   cb: Option<extern fn(command_handle_: i32,
                                                        err: ErrorCode,
                                                        get_utxo_txn_json: *const c_char,
                                                        payment_method: *const c_char)>) -> ErrorCode;

    #[no_mangle]
    fn indy_parse_get_utxo_response(command_handle: i32,
                                    payment_method: *const c_char,
                                    resp_json: *const c_char,
                                    cb: Option<extern fn(command_handle_: i32,
                                                         err: ErrorCode,
                                                         utxo_json: *const c_char)>) -> ErrorCode;

    #[no_mangle]
    fn indy_build_payment_req(command_handle: i32,
                              inputs_json: *const c_char,
                              outputs_json: *const c_char,
                              cb: Option<extern fn(command_handle_: i32,
                                                   err: ErrorCode,
                                                   payment_req_json: *const c_char,
                                                   payment_method: *const c_char)>) -> ErrorCode;

    #[no_mangle]
    fn indy_build_mint_req(command_handle: i32,
                           outputs_json: *const c_char,
                           cb: Option<extern fn(command_handle_: i32,
                                                err: ErrorCode,
                                                mint_req_json: *const c_char,
                                                payment_method: *const c_char)>) -> ErrorCode;

    #[no_mangle]
    fn indy_build_set_txn_fees_req(command_handle: i32,
                                   payment_method: *const c_char,
                                   fees_json: *const c_char,
                                   cb: Option<extern fn(command_handle_: i32,
                                                        err: ErrorCode,
                                                        set_txn_fees_json: *const c_char)>) -> ErrorCode;

    #[no_mangle]
    fn indy_build_get_txn_fees_req(command_handle: i32,
                                   payment_method: *const c_char,
                                   cb: Option<extern fn(command_handle_: i32,
                                                        err: ErrorCode,
                                                        get_txn_fees_json: *const c_char)>) -> ErrorCode;
}

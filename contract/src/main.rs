#![no_main]
#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_snake_case)]

extern crate alloc;

use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
};
use core::convert::TryInto;

use contract::{
    contract_api::runtime::revert,
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};

use types::{
    account::AccountHash,
    bytesrepr::{FromBytes, ToBytes},
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, NamedKeys},
    runtime_args, ApiError, CLType, CLTyped, CLValue, Group, Parameter, RuntimeArgs, URef, U256,
};

#[repr(u16)]
enum FailureCode {
    Zero = 0, // 65,536 as an ApiError::User
    One,      // 65,537 as an ApiError::User
    Two,      // 65,538 as an ApiError::User
}

impl From<FailureCode> for ApiError {
    fn from(code: FailureCode) -> Self {
        ApiError::User(code as u16)
    }
}

#[no_mangle]
pub extern "C" fn name() {
    let val: String = get_key("name");
    ret(val)
}

#[no_mangle]
pub extern "C" fn symbol() {
    let val: String = get_key("symbol");
    ret(val)
}

#[no_mangle]
pub extern "C" fn total_supply() {
    let val: U256 = get_key("total_supply");
    ret(val)
}
#[no_mangle]
pub extern "C" fn contract_owner() {
    let val: AccountHash = get_key("owner");
    ret(val)
}
#[no_mangle]
pub extern "C" fn is_owner() {
    let val: AccountHash = get_key("owner");
    let owner: AccountHash = runtime::get_caller();
    let result: bool = (val == owner);
    ret(result)
}
#[no_mangle]
pub extern "C" fn balance_of() {
    let account: AccountHash = runtime::get_named_arg("account");
    let val: U256 = get_key(&balance_key(&account));
    ret(val)
}
#[no_mangle]
pub extern "C" fn owner_of() {
    let id: U256 = runtime::get_named_arg("id");
    let val: AccountHash = get_key(&owner_key(id));
    ret(val)
}
#[no_mangle]
pub extern "C" fn tokenURI() {
    let val: String = get_key("base_uri");
    let id: U256 = runtime::get_named_arg("id");
    ret(uri_formatter(val, id))
}
#[no_mangle]
pub extern "C" fn approval() {
    let id: U256 = runtime::get_named_arg("id");
    let val: AccountHash = get_key(&approval_key(&id));
    ret(val)
}
#[no_mangle]
pub extern "C" fn approval_for_all() {
    let owner: AccountHash = runtime::get_named_arg("owner");
    let spender: AccountHash = runtime::get_named_arg("spender");
    let val: U256 = get_key(&approval_all_key(&owner, &spender));
    ret(val)
}
#[no_mangle]
pub extern "C" fn approve() {
    let spender: AccountHash = runtime::get_named_arg("spender");
    let id: U256 = runtime::get_named_arg("id");
    let caller: AccountHash = runtime::get_caller();
    let owner: AccountHash = get_key(&owner_key(id));
    if (owner == caller) {
        _approve(spender, id);
    } else {
        revert(ApiError::User(1));
    }
}

#[no_mangle]
pub extern "C" fn approve_all() {
    let spender: AccountHash = runtime::get_named_arg("spender");

    _approve_all(runtime::get_caller(), spender);
}
#[no_mangle]
pub extern "C" fn add_minter() {
    let minter: AccountHash = runtime::get_named_arg("minter");
    let is_minter: bool = get_key(&minter_key(&runtime::get_caller()));
    if (is_minter) {
        _add_minter(minter);
    } else {
        revert(ApiError::User(1));
    }
}

#[no_mangle]
pub extern "C" fn transfer() {
    let recipient: AccountHash = runtime::get_named_arg("recipient");
    let id: U256 = runtime::get_named_arg("id");
    _transfer(runtime::get_caller(), recipient, id);
}
#[no_mangle]
pub extern "C" fn mint() {
    let recipient: AccountHash = runtime::get_named_arg("recipient");
    let id: U256 = runtime::get_named_arg("id");
    let caller: AccountHash = runtime::get_caller();
    _mint(id, recipient, caller);
}

#[no_mangle]
pub extern "C" fn transfer_from() {
    let owner: AccountHash = runtime::get_named_arg("owner");
    let recipient: AccountHash = runtime::get_named_arg("recipient");
    let id: U256 = runtime::get_named_arg("id");
   
    _transfer_from(owner, recipient, id);
}

#[no_mangle]
pub extern "C" fn call() {
    let token_name: String = runtime::get_named_arg("token_name");
    let token_symbol: String = runtime::get_named_arg("token_symbol");
    let base_uri: String = runtime::get_named_arg("base_uri");
    let token_total_supply: U256 = U256::from(0);
    let owner: AccountHash = runtime::get_named_arg("owner");

    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(endpoint("name", vec![], CLType::String));
    entry_points.add_entry_point(endpoint("symbol", vec![], CLType::String));
    entry_points.add_entry_point(endpoint("symbol", vec![], CLType::String));
    entry_points.add_entry_point(endpoint("total_supply", vec![], CLType::U32));
    entry_points.add_entry_point(endpoint(
        "transfer",
        vec![
            Parameter::new("recipient", AccountHash::cl_type()),
            Parameter::new("id", CLType::U256),
        ],
        CLType::Unit,
    ));
    entry_points.add_entry_point(endpoint(
        "balance_of",
        vec![Parameter::new("account", AccountHash::cl_type())],
        CLType::U256,
    ));
    entry_points.add_entry_point(endpoint("is_owner", vec![], CLType::Bool));
    entry_points.add_entry_point(endpoint(
        "approve_all",
        vec![Parameter::new("spender", AccountHash::cl_type())],
        CLType::Unit,
    ));
    entry_points.add_entry_point(endpoint(
        "approval",
        vec![Parameter::new("id", CLType::U256)],
        AccountHash::cl_type(),
    ));
    entry_points.add_entry_point(endpoint(
        "approval_for_all",
        vec![
            Parameter::new("owner", AccountHash::cl_type()),
            Parameter::new("spender", AccountHash::cl_type()),
        ],
        CLType::Bool,
    ));
    entry_points.add_entry_point(endpoint(
        "approve",
        vec![Parameter::new("id", CLType::U256)],
        CLType::Unit,
    ));
    entry_points.add_entry_point(endpoint(
        "mint",
        vec![
            Parameter::new("to", AccountHash::cl_type()),
            Parameter::new("id", CLType::U256),
        ],
        CLType::Unit,
    ));
    entry_points.add_entry_point(endpoint(
        "add_minter",
        vec![Parameter::new("minter", AccountHash::cl_type())],
        CLType::Unit,
    ));
    entry_points.add_entry_point(endpoint(
        "transfer_from",
        vec![
            Parameter::new("owner", AccountHash::cl_type()),
            Parameter::new("recipient", AccountHash::cl_type()),
            Parameter::new("id", CLType::U256),
        ],
        CLType::Unit,
    ));

    let mut named_keys = NamedKeys::new();
    named_keys.insert("name".to_string(), storage::new_uref(token_name).into());
    named_keys.insert("symbol".to_string(), storage::new_uref(token_symbol).into());
    named_keys.insert("base_uri".to_string(), storage::new_uref(base_uri).into());
    named_keys.insert(
        "contract_owner".to_string(),
        storage::new_uref(owner).into(),
    );
    named_keys.insert(
        "total_supply".to_string(),
        storage::new_uref(token_total_supply).into(),
    );
    named_keys.insert(
        balance_key(&runtime::get_caller()),
        storage::new_uref(token_total_supply).into(),
    );
    named_keys.insert(
        minter_key(&runtime::get_caller()),
        storage::new_uref(true).into(),
    );
    named_keys.insert(
        contract_owner_key(&runtime::get_caller()),
        storage::new_uref(true).into(),
    );
    let (contract_hash, _) =
        storage::new_locked_contract(entry_points, Some(named_keys), None, None);
    runtime::put_key("ERC721", contract_hash.into());
    runtime::put_key("ERC721_hash", storage::new_uref(contract_hash).into());
}

fn _transfer(sender: AccountHash, recipient: AccountHash, id: U256) {
    let sender_key = balance_key(&sender);
    let recipient_key = balance_key(&recipient);
    let owner_key = owner_key(id);
    let account_default: AccountHash = Default::default();
    if (get_key::<AccountHash>(&owner_key) != sender) {
        revert(ApiError::User(2));
    }
    let new_sender_balance: U256 = (get_key::<U256>(&sender_key)) - 1;
    set_key(&sender_key, new_sender_balance);
    let new_recipient_balance: U256 = (get_key::<U256>(&recipient_key)) + 1;
    set_key(&recipient_key, new_recipient_balance);
    set_key(&owner_key, recipient);
    set_key(&approval_key(&id), account_default);
}

fn _transfer_from(owner: AccountHash, recipient: AccountHash, id: U256) {
    let sender = runtime::get_caller();
    
    let approval_key = approval_key(&id);
    let approve_all = approval_all_key(&owner, &recipient);

    let approved_account = get_key::<AccountHash>(&approval_key);

    let all_true: bool = get_key::<bool>(&approve_all);
    if all_true || approved_account == sender || sender==owner  {
        _transfer(owner, recipient, id);
    }else{
        revert(ApiError::User(1))
    }
    
}
fn _mint(id: U256, receiver: AccountHash, _caller: AccountHash) {
    let owner_key = owner_key(id);
    let token_owner: AccountHash = get_key(&owner_key);
    let contract_owner: AccountHash = get_key("owner");
    let account_default: AccountHash = Default::default();
    let is_minter: bool = get_key(&minter_key(&_caller));

    if (token_owner == account_default && is_minter == true) {
        let supply: U256 = get_key("total_supply");
        let balance_key = balance_key(&receiver);

        let new_recipient_balance: U256 = (get_key::<U256>(&balance_key) + 1);
        set_key(&balance_key, new_recipient_balance);
        set_key(&owner_key, receiver);
        set_key("total_supply", supply + 1);
    } else {
        revert(ApiError::User(1))
    }
}

fn _approve(spender: AccountHash, id: U256) {
    set_key(&approval_key(&id), spender);
}
fn _approve_all(owner: AccountHash, spender: AccountHash) {
    set_key(&approval_all_key(&owner, &spender), true);
}
fn _add_minter(minter: AccountHash) {
    set_key(&minter_key(&minter), true);
}
fn _transfer_ownership(current:AccountHash,newowner: AccountHash) {
    set_key(&contract_owner_key(&current), false);
    set_key(&contract_owner_key(&newowner), true);
}
fn _remove_minter(minter: AccountHash) {
    set_key(&minter_key(&minter), false);
}
fn ret<T: CLTyped + ToBytes>(value: T) {
    runtime::ret(CLValue::from_t(value).unwrap_or_revert())
}

fn get_key<T: FromBytes + CLTyped + Default>(name: &str) -> T {
    match runtime::get_key(name) {
        None => Default::default(),
        Some(value) => {
            let key = value.try_into().unwrap_or_revert();
            storage::read(key).unwrap_or_revert().unwrap_or_revert()
        }
    }
}

fn set_key<T: ToBytes + CLTyped>(name: &str, value: T) {
    match runtime::get_key(name) {
        Some(key) => {
            let key_ref = key.try_into().unwrap_or_revert();
            storage::write(key_ref, value);
        }
        None => {
            let key = storage::new_uref(value).into();
            runtime::put_key(name, key);
        }
    }
}

fn balance_key(account: &AccountHash) -> String {
    format!("balances_{}", account)
}
fn minter_key(account: &AccountHash) -> String {
    format!("minter_{}", account)
}
fn contract_owner_key(account: &AccountHash) -> String {
    format!("minter_{}", account)
}
fn owner_key(id: U256) -> String {
    format!("owner_{}", id)
}

fn uri_formatter(base: String, id: U256) -> String {
    format!("{}{}", base, id)
}
fn approval_key(id: &U256) -> String {
    format!("approval_{}", id)
}
fn approval_all_key(owner: &AccountHash, spender: &AccountHash) -> String {
    format!("approval_all_{}_{}", owner, spender)
}

fn endpoint(name: &str, param: Vec<Parameter>, ret: CLType) -> EntryPoint {
    EntryPoint::new(
        String::from(name),
        param,
        ret,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

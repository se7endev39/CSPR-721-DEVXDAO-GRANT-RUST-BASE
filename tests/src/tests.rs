use crate::erc721::{token_cfg, Sender, Token};
use casper_types::{U256, U512};
#[test]
fn test_erc721_deploy() {
    let t = Token::deployed();
    assert_eq!(t.name(), token_cfg::NAME);
    assert_eq!(t.symbol(), token_cfg::SYMBOL);

    println!("{}", t.owner());
    assert_eq!(t.balance_of(t.bob), 0.into());
}

#[test]
fn test_erc721_transfer() {
    let id: U256 = 10.into();
    let mut t = Token::deployed();
    println!("{}", t.is_minter(t.ali));
    println!("{}", t.is_minter(t.bob));
    t.mint(t.bob, Sender(t.ali), id);

    //t.transfer_from(t.ali,t.bob, id, Sender(t.ali));
    println!("{}", t.balance_of(t.bob));
    //println!("{}",t.owner()==t.ali);

    assert_eq!(t.balance_of(t.bob), 1.into());
    t.transfer(t.ali, id, Sender(t.bob));
    assert_eq!(t.balance_of(t.ali), 1.into());
    assert_eq!(t.balance_of(t.bob), 0.into());
}
#[test]
fn test_erc721_approve_transfer() {
    let id = 10.into();
    let id1 = 11.into();
    let mut t = Token::deployed();
    t.mint(t.bob, Sender(t.ali), id);
    t.mint(t.bob, Sender(t.ali), id1);
    t.approve(t.ali, id, Sender(t.bob));
    t.transfer_from(t.bob, t.ali, id, Sender(t.ali));
    t.transfer_from(t.bob, t.ali, id1, Sender(t.bob));
    assert_eq!(t.balance_of(t.ali), 2.into());
    println!("{}", t.default);
    assert_eq!(t.approval(id), t.default);
}
#[test]
fn test_erc721_approve_all() {
    let id = 10.into();
    let id1 = 11.into();
    let id2 = 17.into();
    let mut t = Token::deployed();
    t.mint(t.bob, Sender(t.ali), id);
    t.mint(t.bob, Sender(t.ali), id1);
    t.mint(t.bob, Sender(t.ali), id2);
    t.approve_all(t.ali, Sender(t.bob));
    t.transfer_from(t.bob, t.ali, id, Sender(t.ali));
    t.transfer_from(t.bob, t.ali, id1, Sender(t.bob));
    t.transfer_from(t.bob, t.ali, id2, Sender(t.ali));
    assert_eq!(t.balance_of(t.ali), 3.into());
    assert_eq!(t.owner_of(id), t.ali);
    assert_eq!(t.owner_of(id1), t.ali);
    assert_eq!(t.owner_of(id2), t.ali);
    assert_eq!(t.approval_all(t.bob, t.ali), true);
}
#[test]
fn test_erc721_test_uri() {
    let id = 123.into();
    let mut t = Token::deployed();
    t.mint(t.bob, Sender(t.ali), id);
    let uri = t.token_uri(id);
    assert_eq!(uri, "test.io/123");
}

#[test]
#[should_panic]
fn test_erc721_transfer_too_much() {
    let id = 1.into();
    let mut t = Token::deployed();
    t.mint(t.bob, Sender(t.ali), id);

    t.transfer(t.ali, id, Sender(t.bob));
    t.transfer(t.ali, id, Sender(t.bob));
}

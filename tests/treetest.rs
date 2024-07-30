extern crate chunkfs;

use chunkfs::lsmtree::LsmTree;

#[test]
fn test_insert() {
    let mut tree = LsmTree::new();
    tree.insert(2, "22");
    tree.insert(3, "33");
    tree.insert(6, "66");

    assert_eq!(tree.get(2), "22");
    assert_eq!(tree.get(3), "33");
    assert_eq!(tree.get(6), "66");
    assert_eq!(tree.get(1), "");
}

#[test]
fn test_big_right_rotate() {
    let mut tree = LsmTree::new();
    tree.insert(6, "66");
    tree.insert(7, "77");
    tree.insert(3, "33");
    tree.insert(1, "11");
    tree.insert(4, "44");
    tree.insert(5, "55");

    assert_eq!(tree.get_balance_factor(4), Some(0));
    assert_eq!(tree.get_balance_factor(5), Some(0));
    assert_eq!(tree.get_balance_factor(6), Some(0));
    assert_eq!(tree.get_balance_factor(7), Some(0));
    assert_eq!(tree.get_balance_factor(1), Some(0));
    assert_eq!(tree.get_balance_factor(3), Some(-1));
}

#[test]
fn test_big_left_rotate() {
    let mut tree = LsmTree::new();
    tree.insert(3, "33");
    tree.insert(2, "22");
    tree.insert(6, "66");
    tree.insert(5, "55");
    tree.insert(7, "77");
    tree.insert(4, "44");
    tree.insert(1, "11");
    tree.insert(0, "00");
    tree.insert(9, "99");
    tree.insert(8, "88");

    assert_eq!(tree.get_balance_factor(0), Some(0));
    assert_eq!(tree.get_balance_factor(1), Some(0));
    assert_eq!(tree.get_balance_factor(2), Some(0));
    assert_eq!(tree.get_balance_factor(3), Some(-1));
    assert_eq!(tree.get_balance_factor(4), Some(0));
    assert_eq!(tree.get_balance_factor(5), Some(0));
    assert_eq!(tree.get_balance_factor(6), Some(0));
    assert_eq!(tree.get_balance_factor(7), Some(1));
    assert_eq!(tree.get_balance_factor(8), Some(0));
    assert_eq!(tree.get_balance_factor(9), Some(-1));
}

#[test]
fn test_left_rotate() {
    let mut tree = LsmTree::new();
    tree.insert(2, "22");
    tree.insert(3, "33");
    tree.insert(9, "99");

    assert_eq!(tree.get(2), "22");
    assert_eq!(tree.get(3), "33");
    assert_eq!(tree.get(9), "99");
}

#[test]
fn test_right_rotate() {
    let mut tree = LsmTree::new();
    tree.insert(6, "66");
    tree.insert(7, "77");
    tree.insert(4, "44");
    tree.insert(5, "55");
    tree.insert(3, "33");
    tree.insert(1, "11");

    assert_eq!(tree.get_balance_factor(4), Some(0));
    assert_eq!(tree.get_balance_factor(5), Some(0));
    assert_eq!(tree.get_balance_factor(6), Some(0));
    assert_eq!(tree.get_balance_factor(7), Some(0));
    assert_eq!(tree.get_balance_factor(1), Some(0));
    assert_eq!(tree.get_balance_factor(3), Some(-1));
}

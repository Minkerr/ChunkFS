extern crate chunkfs;

use chunkfs::lsmtree::LsmTree;

#[test]
fn test_insert() {
    let mut tree = LsmTree::new();
    tree.insert(2, "22");
    tree.insert(6, "66");
    tree.insert(3, "33");
    tree.insert(1, "11");
    tree.insert(0, "00");
    tree.insert(9, "99");
    tree.insert(4, "44");
    tree.insert(5, "55");
    tree.insert(7, "77");
    tree.insert(8, "88");

    tree.print();
    println!("==============================");

    // assert_eq!(tree.get(1), "11");
    // assert_eq!(tree.get(4), "44");
    // assert_eq!(tree.get(7), "77");
    // assert_eq!(tree.get(10), "");
}

#[test]
fn test_big_right_rotate() {
    let mut tree = LsmTree::new();
    tree.insert(6, "66");
    tree.insert(1, "11");
    tree.insert(7, "77");
    tree.insert(8, "88");
    tree.insert(9, "99");
    tree.insert(0, "00");
    tree.insert(4, "44");
    tree.insert(3, "33");
    tree.insert(5, "55");
    tree.insert(2, "22");

    tree.print();
    println!("==============================");

    // assert_eq!(tree.get(1), "11");
    // assert_eq!(tree.get(4), "44");
    // assert_eq!(tree.get(7), "77");
    // assert_eq!(tree.get(10), "");
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

    tree.print();
    println!("==============================");

    // assert_eq!(tree.get(1), "11");
    // assert_eq!(tree.get(4), "44");
    // assert_eq!(tree.get(7), "77");
    // assert_eq!(tree.get(10), "");
}

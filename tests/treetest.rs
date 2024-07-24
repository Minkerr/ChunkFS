extern crate chunkfs;

use chunkfs::lsmtree::AvlTree;

#[test]
fn test_insert() {
    let mut tree = AvlTree::new();
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

    tree.print(&mut |key, _, balance_factor| {
        println!("{}({})", key, balance_factor);
    });

    assert_eq!(tree.get(1), Some("11"));
    assert_eq!(tree.get(2), Some("22"));
    assert_eq!(tree.get(3), Some("33"));
    assert_eq!(tree.get(10), None);
}

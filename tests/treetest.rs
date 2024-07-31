extern crate chunkfs;

use chunkfs::lsmtree::LsmTree;

#[test]
fn test_insert() {
    let mut tree = LsmTree::new(2, |v| v, |v| v);
    tree.insert(2, "22".to_string());
    tree.insert(1, "11".to_string());
    tree.insert(3, "33".to_string());
    tree.insert(4, "44".to_string());
    tree.insert(6, "66".to_string());

    assert_eq!(tree.get(2).unwrap(), "22");
    assert_eq!(tree.get(3).unwrap(), "33");
    assert_eq!(tree.get(6).unwrap(), "66");
    assert_eq!(tree.get(0), None);
}

#[test]
fn test_big_right_rotate() {
    let mut tree = LsmTree::new(4, |v| v, |v| v);
    tree.insert(6, "66".to_string());
    tree.insert(7, "77".to_string());
    tree.insert(3, "33".to_string());
    tree.insert(1, "11".to_string());
    tree.insert(4, "44".to_string());
    tree.insert(5, "55".to_string());

    assert_eq!(tree.get_balance_factor(4), Some(0));
    assert_eq!(tree.get_balance_factor(5), Some(0));
    assert_eq!(tree.get_balance_factor(6), Some(0));
    assert_eq!(tree.get_balance_factor(7), Some(0));
    assert_eq!(tree.get_balance_factor(1), Some(0));
    assert_eq!(tree.get_balance_factor(3), Some(-1));
}

#[test]
fn test_big_left_rotate() {
    let mut tree = LsmTree::new(4, |v| v, |v| v);
    tree.insert(3, "33".to_string().to_string());
    tree.insert(2, "22".to_string().to_string());
    tree.insert(6, "66".to_string().to_string());
    tree.insert(5, "55".to_string().to_string());
    tree.insert(7, "77".to_string().to_string());
    tree.insert(4, "44".to_string().to_string());
    tree.insert(1, "11".to_string().to_string());
    tree.insert(0, "00".to_string().to_string());
    tree.insert(9, "99".to_string().to_string());
    tree.insert(8, "88".to_string().to_string());
    tree.print();
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
    let mut tree = LsmTree::new(4, |v| v, |v| v);
    tree.insert(2, "22".to_string());
    tree.insert(3, "33".to_string());
    tree.insert(9, "99".to_string());

    assert_eq!(tree.get_balance_factor(2), Some(0));
    assert_eq!(tree.get_balance_factor(3), Some(0));
    assert_eq!(tree.get_balance_factor(9), Some(0));
}

#[test]
fn test_right_rotate() {
    let mut tree = LsmTree::new(4, |v| v, |v| v);
    tree.insert(6, "66".to_string());
    tree.insert(7, "77".to_string());
    tree.insert(4, "44".to_string());
    tree.insert(5, "55".to_string());
    tree.insert(3, "33".to_string());
    tree.insert(1, "11".to_string());

    assert_eq!(tree.get_balance_factor(4), Some(0));
    assert_eq!(tree.get_balance_factor(5), Some(0));
    assert_eq!(tree.get_balance_factor(6), Some(0));
    assert_eq!(tree.get_balance_factor(7), Some(0));
    assert_eq!(tree.get_balance_factor(1), Some(0));
    assert_eq!(tree.get_balance_factor(3), Some(-1));
}

#[test]
fn test_tree_with_array() {
    fn boxer(input: Vec<u8>) -> String {
        input.iter()
            .map(|&x| x.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }

    fn unboxer(input: String) -> Vec<u8> {
        let res = input.split(',')
            .map(|s| s.trim().parse::<u8>())
            .collect();
        match res {
            Ok(v) => v,
            Err(e) => panic!("{}", e)
        }
    }

    let mut tree = LsmTree::new(2, boxer, unboxer);
    tree.insert(1, Vec::from([1, 2, 3]));
    tree.insert(2, Vec::from([2, 3, 9]));
    tree.insert(3, Vec::from([6, 6, 6, 6, 6, 6, 6, 6]));
    tree.insert(4, Vec::from([1]));
    tree.insert(5, Vec::from([7, 9, 4]));
    assert_eq!(tree.get(2), Some(Vec::from([2, 3, 9])));
    assert_eq!(tree.get(4), Some(Vec::from([1])));
    assert_eq!(tree.get(5), Some(Vec::from([7, 9, 4])));
}

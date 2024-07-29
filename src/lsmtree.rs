use std::cmp::Ordering;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

#[derive(Debug)]
pub struct LsmTree<K, V> {
    root: Node<K, V>,
    size: u32,
    number_of_unloads: u8,
    unload_bias: u32,
}

#[derive(Debug)]
#[derive(Clone)]
enum Node<K, V> {
    Leaf,
    Branch {
        key: K,
        value: Option<V>,
        sstable_number: u8,
        left: Box<Node<K, V>>,
        right: Box<Node<K, V>>,
        balance_factor: i8,
    },
}


impl<K: Ord + Clone + ToString + Display, V: Clone + ToString + PartialEq + Display> LsmTree<K, V> {
    pub fn new() -> Self {
        LsmTree {
            root: Node::Leaf,
            size: 0,
            number_of_unloads: 0,
            unload_bias: 4,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let mut climb = Box::new(true);
        self.root = Node::insert(&mut self.root, key, value, &mut climb);
        self.size += 1;
        if self.size % self.unload_bias == 0 {
            self.unload();
        }
    }

    pub fn get(&self, target_key: K) -> String {
        let (result_value, sstable_number) = Node::get(&self.root, target_key.clone());
        match result_value {
            None => {
                if sstable_number != 0 {
                    Self::get_from_table(target_key, sstable_number)
                } else {
                    String::from("")
                }
            }
            Some(value) => { value.to_string() }
        }
    }

    fn get_from_table(target_key: K, num: u8) -> String { // returns string for tests
        let file = File::open(format!("storage/sstable{}", num));
        let reader = BufReader::new(file.unwrap());
        for line in reader.lines() {
            let line = line.unwrap();
            let (key, value) = line.split_once(':')
                .map(|(a, b)| (a.to_string(), b.to_string())).unwrap();
            if key == target_key.to_string() {
                return value;
            }
        }
        return String::from("");
    }

    pub fn print(&self) {
        Node::print(&self.root, 0)
    }

    pub fn unload(&mut self) {
        self.number_of_unloads += 1;
        let mut file = File::create(format!("storage/sstable{}", self.number_of_unloads))
            .unwrap();
        Node::unload_to_file(&mut self.root, &mut file, self.number_of_unloads);
    }
}

impl<K: Ord + Clone + ToString + Display, V: Clone + ToString + PartialEq + Display> Node<K, V> {
    fn insert(&mut self, new_key: K, new_value: V, should_climb: &mut bool) -> Node<K, V> {
        match self {
            Node::Leaf => Node::Branch {
                key: new_key,
                value: Some(new_value),
                sstable_number: 0,
                left: Box::new(Node::Leaf),
                right: Box::new(Node::Leaf),
                balance_factor: 0,
            },
            Node::Branch {
                ref mut key,
                ref mut left,
                ref mut right,
                ref mut balance_factor,
                ..
            } => {
                let comparison_result = new_key.cmp(&key);
                match comparison_result {
                    Ordering::Greater => {
                        *right = Box::new(right.insert(new_key, new_value, should_climb));
                        *balance_factor = (*balance_factor).clone() + 1;
                    }
                    Ordering::Less => {
                        *left = Box::new(left.insert(new_key, new_value, should_climb));
                        *balance_factor = (*balance_factor).clone() - 1;
                    }
                    Ordering::Equal => {
                        *should_climb = false;
                        return self.clone();
                    }
                }
                if !*should_climb {
                    if comparison_result == Ordering::Less {
                        *balance_factor += 1;
                    } else {
                        *balance_factor -= 1;
                    }
                    return self.clone();
                }
                let bf = (*balance_factor).clone();
                if bf == -2 || bf == 0 || bf == 2 {
                    *should_climb = false;
                }

                Node::balance(self)
            }
        }
    }

    fn get(node: &Node<K, V>, target_key: K) -> (Option<V>, u8) {
        match node {
            Node::Leaf => (None, 0),
            Node::Branch {
                ref key,
                ref value,
                ref sstable_number,
                ref left,
                ref right,
                ..
            } => match target_key.cmp(&key) {
                Ordering::Greater => Node::get(right, target_key),
                Ordering::Less => Node::get(left, target_key),
                Ordering::Equal => {
                    if *sstable_number == 0 {
                        (value.clone(), 0)
                    } else {
                        (None, *sstable_number)
                    }
                }
            },
        }
    }

    fn rotate_left(node: &mut Node<K, V>) -> Node<K, V> {
        let mut right_node = node.get_right_child();
        let left_right_node = right_node.get_left_child();

        let new_left_right_node = match *right_node {
            Node::Branch { ref mut left, .. } => {
                *left = Box::new(node.clone());
                left
            }
            Node::Leaf => unreachable!()
        };

        match **new_left_right_node {
            Node::Branch { ref mut right, .. } => { *right = Box::new(*left_right_node) }
            Node::Leaf => unreachable!()
        }

        match *right_node {
            Node::Branch { ref mut left, ref mut balance_factor, .. } => {
                let node_balance = match **left {
                    Node::Branch { ref mut balance_factor, .. } => { balance_factor }
                    Node::Leaf => unreachable!()
                };
                if (*balance_factor).clone() != 0 {
                    *node_balance = 0;
                    *balance_factor = 0;
                } else {
                    *node_balance = 1;
                    *balance_factor = -1;
                }
            }
            Node::Leaf => unreachable!()
        }

        *right_node
    }

    fn rotate_right(node: &mut Node<K, V>) -> Node<K, V> {
        let mut left_node = node.get_left_child();
        let right_left_node = left_node.get_right_child();

        let new_right_left_node = match *left_node {
            Node::Branch { ref mut right, .. } => {
                *right = Box::new(node.clone());
                right
            }
            Node::Leaf => { unreachable!() }
        };

        match **new_right_left_node {
            Node::Branch { ref mut left, .. } => { *left = Box::new(*right_left_node) }
            Node::Leaf => {}
        }

        match *left_node {
            Node::Branch { ref mut right, ref mut balance_factor, .. } => {
                let node_balance = match **right {
                    Node::Branch { ref mut balance_factor, .. } => { balance_factor }
                    Node::Leaf => unreachable!()
                };
                if (*balance_factor).clone() != 0 {
                    *node_balance = 0;
                    *balance_factor = 0;
                } else {
                    *node_balance = -1;
                    *balance_factor = 1;
                }
            }
            Node::Leaf => unreachable!()
        }

        *left_node
    }

    fn big_rotate_left(node: &mut Node<K, V>) -> Node<K, V> {
        let mut right_node = node.get_right_child();
        let mut left_right_node = right_node.get_left_child();
        let left_left_right_node = left_right_node.get_left_child();
        let right_left_right_node = left_right_node.get_right_child();

        let (new_left_left_right_node, new_right_left_right_node)
            = match *left_right_node {
            Node::Branch { ref mut left, ref mut right, .. } => {
                *left = Box::new(node.clone());
                *right = Box::new((*right_node).clone());
                (left, right)
            }
            Node::Leaf => { unreachable!() }
        };

        match **new_left_left_right_node {
            Node::Branch { ref mut right, .. }
            => { *right = Box::new(*left_left_right_node) }
            Node::Leaf => unreachable!()
        }

        match **new_right_left_right_node {
            Node::Branch { ref mut left, .. }
            => { *left = Box::new(*right_left_right_node) }
            Node::Leaf => unreachable!()
        }

        match *left_right_node {
            Node::Branch {
                ref mut left,
                ref mut right,
                ref mut balance_factor,
                ..
            } => {
                let node_balance = match **left {
                    Node::Branch { ref mut balance_factor, .. } => { balance_factor }
                    Node::Leaf => unreachable!()
                };
                let right_node_balance = match **right {
                    Node::Branch { ref mut balance_factor, .. } => { balance_factor }
                    Node::Leaf => unreachable!()
                };
                match balance_factor {
                    -1 => {
                        *node_balance = 0;
                        *right_node_balance = 1;
                    }
                    0 => {
                        *node_balance = 0;
                        *right_node_balance = 0;
                    }
                    1 => {
                        *node_balance = -1;
                        *right_node_balance = 0;
                    }
                    _ => {}
                }
                *balance_factor = 0;
            }
            Node::Leaf => unreachable!()
        }

        *left_right_node
    }

    fn big_rotate_right(node: &mut Node<K, V>) -> Node<K, V> {
        let mut left_node = node.get_left_child();
        let mut right_left_node = left_node.get_right_child();
        let left_right_left_node = right_left_node.get_left_child();
        let right_right_left_node = right_left_node.get_right_child();

        let (new_left_right_left_node, new_right_right_left_node)
            = match *right_left_node {
            Node::Branch { ref mut left, ref mut right, .. } => {
                *left = Box::new((*left_node).clone());
                *right = Box::new(node.clone());
                (left, right)
            }
            Node::Leaf => { unreachable!() }
        };

        match **new_left_right_left_node {
            Node::Branch { ref mut right, .. }
            => { *right = Box::new(*left_right_left_node) }
            Node::Leaf => unreachable!()
        }

        match **new_right_right_left_node {
            Node::Branch { ref mut left, .. }
            => { *left = Box::new(*right_right_left_node) }
            Node::Leaf => unreachable!()
        }

        match *right_left_node {
            Node::Branch {
                ref mut left,
                ref mut right,
                ref mut balance_factor,
                ..
            } => {
                let left_node_balance = match **left {
                    Node::Branch { ref mut balance_factor, .. } => { balance_factor }
                    Node::Leaf => unreachable!()
                };
                let node_balance = match **right {
                    Node::Branch { ref mut balance_factor, .. } => { balance_factor }
                    Node::Leaf => unreachable!()
                };
                match balance_factor {
                    -1 => {
                        *node_balance = 1;
                        *left_node_balance = 0;
                    }
                    0 => {
                        *node_balance = 0;
                        *left_node_balance = 0;
                    }
                    1 => {
                        *node_balance = 0;
                        *left_node_balance = -1;
                    }
                    _ => {}
                }
                *balance_factor = 0;
            }
            Node::Leaf => unreachable!()
        }

        *right_left_node
    }

    fn get_right_child(&mut self) -> Box<Node<K, V>> {
        match *self {
            Node::Branch { ref mut right, .. } => {
                (*right).clone()
            }
            Node::Leaf => { Box::new(Node::Leaf) }
        }
    }

    fn get_left_child(&mut self) -> Box<Node<K, V>> {
        match *self {
            Node::Branch { ref mut left, .. } => {
                (*left).clone()
            }
            Node::Leaf => { Box::new(Node::Leaf) }
        }
    }

    fn balance(node: &mut Node<K, V>) -> Node<K, V> {
        match *node {
            Node::Leaf => { node.clone() }
            Node::Branch {
                ref left,
                ref right,
                ref balance_factor,
                ..
            } => {
                match *balance_factor {
                    2 => {
                        match **right {
                            Node::Branch { ref balance_factor, .. }
                            => if *balance_factor >= 0 {
                                Node::rotate_left(node)
                            } else {
                                Node::big_rotate_left(node)
                            },
                            _ => unreachable!()
                        }
                    }
                    -2 => {
                        match **left {
                            Node::Branch { ref balance_factor, .. }
                            => if *balance_factor <= 0 {
                                Node::rotate_right(node)
                            } else {
                                Node::big_rotate_right(node)
                            },
                            _ => unreachable!()
                        }
                    }
                    _ => { node.clone() }
                }
            }
        }
    }

    pub fn print(&self, d: i32) {
        match self {
            Node::Leaf => {}
            Node::Branch { key, value, left, right, balance_factor, .. } => {
                left.print(d.clone() + 1);
                for _i in 0..d {
                    print!("    ");
                }
                match value {
                    None => { println!("{}:({})", key, balance_factor) }
                    Some(v) => { println!("{}:{}({})", key, v, balance_factor) }
                }
                right.print(d.clone() + 1);
            }
        }
    }

    pub fn unload_to_file(&mut self, file: &mut File, number_of_unloads: u8) {
        match self {
            Node::Leaf => {}
            Node::Branch {
                key,
                value,
                left,
                right,
                sstable_number,
                ..
            } => {
                left.unload_to_file(file, number_of_unloads);
                let text = match value {
                    Some(v) => { format!("{}:{}\n", key, v) }
                    None => format!("{}:\n", key)
                };
                if *value != None {
                    let _ = file.write_all(text.as_bytes());
                    *value = None;
                    *sstable_number = number_of_unloads;
                }
                right.unload_to_file(file, number_of_unloads);
            }
        }
    }
}




use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json;
use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LsmTree<K, V>
where
    K: Clone,
    V: Clone,
{
    root: Node<K, V>,
    stack: Vec<Node<K, V>>,
    size: u32,
    number_of_unloads: u8,
    unload_bias: u32,
    identifier: String,
}

#[derive(Debug, Clone)]
enum Node<K, V>
where
    K: Clone,
    V: Clone,
{
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

impl<'b, K, V> LsmTree<K, V>
where
    K: Ord + Clone + Serialize + for<'a> Deserialize<'a>,
    V: Clone + Serialize + for<'a> Deserialize<'a>,
{
    pub fn new(bias: u32) -> Self {
        LsmTree {
            root: Node::Leaf,
            stack: Vec::new(),
            size: 0,
            number_of_unloads: 0,
            unload_bias: bias,
            identifier: Self::generate_random_string(16),
        }
    }

    fn generate_random_string(length: usize) -> String {
        let rng = thread_rng();
        let random_string: String = rng
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect();
        random_string
    }

    pub fn insert(&mut self, key: K, value: V) {
        let mut climb = Box::new(true);
        self.root = Node::insert(&mut self.root, key, value, &mut climb);
        self.size += 1;
        if self.size % self.unload_bias == 0 {
            self.unload();
        }
    }

    pub fn get_balance_factor(&self, target_key: K) -> Option<i8> {
        self.root.get_balance_factor(target_key.clone())
    }

    pub fn get(&self, target_key: K) -> Option<V> {
        let (result_value, sstable_number) = Node::get(&self.root, target_key.clone());
        match result_value {
            None => {
                if sstable_number != 0 {
                    Self::get_from_table(target_key, sstable_number, self.identifier.clone())
                } else {
                    None
                }
            }
            Some(value) => Some(value),
        }
    }

    fn get_from_table(target_key: K, num: u8, id: String) -> Option<V> {
        let file = File::open(format!("storage/tree{}/sstable{}", id, num));
        let reader = BufReader::new(file.unwrap());
        for line in reader.lines() {
            let line = line.unwrap();
            let (key, value) = line
                .split_once(':')
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .unwrap();
            if key == serde_json::to_string(&target_key).unwrap() {
                return Some(serde_json::from_str(&value).unwrap());
            }
        }
        return None;
    }

    pub fn print(&self) {
        Node::print(&self.root, 0)
    }

    fn unload(&mut self) {
        self.number_of_unloads += 1;
        if !Path::new("storage").exists() {
            let _ = std::fs::create_dir("storage");
        }
        let tree_folder = format!("storage/tree{}", self.identifier);
        if !Path::new(&tree_folder).exists() {
            let _ = std::fs::create_dir(tree_folder);
        }

        let mut file = File::create(format!(
            "storage/tree{}/sstable{}",
            self.identifier, self.number_of_unloads
        ))
        .unwrap();
        Node::unload_to_file(&mut self.root, &mut file, self.number_of_unloads);
    }

    pub fn iter(&self) -> LsmTreeIterator<K, V> {
        LsmTreeIterator::new(self.root.clone(), self.identifier.clone())
    }
}

impl<K, V> Node<K, V>
where
    K: Ord + Clone + Serialize + for<'a> Deserialize<'a>,
    V: Clone + Serialize + for<'a> Deserialize<'a>,
{
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

                self.balance()
            }
        }
    }

    fn get_balance_factor(&self, target_key: K) -> Option<i8> {
        match self {
            Node::Leaf => None,
            Node::Branch {
                ref key,
                ref left,
                ref right,
                balance_factor,
                ..
            } => match target_key.cmp(&key) {
                Ordering::Greater => right.get_balance_factor(target_key),
                Ordering::Less => left.get_balance_factor(target_key),
                Ordering::Equal => Some(*balance_factor),
            },
        }
    }

    fn get(&self, target_key: K) -> (Option<V>, u8) {
        match self {
            Node::Leaf => (None, 0),
            Node::Branch {
                ref key,
                ref value,
                ref sstable_number,
                ref left,
                ref right,
                ..
            } => match target_key.cmp(&key) {
                Ordering::Greater => right.get(target_key),
                Ordering::Less => left.get(target_key),
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

    fn rotate_left(&mut self) -> Node<K, V> {
        let mut right_node = self.get_right_child();
        let left_right_node = right_node.get_left_child();

        let new_left_right_node = match *right_node {
            Node::Branch { ref mut left, .. } => {
                *left = Box::new(self.clone());
                left
            }
            Node::Leaf => unreachable!(),
        };

        new_left_right_node.set_right_child(left_right_node);

        match *right_node {
            Node::Branch {
                ref mut left,
                ref mut balance_factor,
                ..
            } => {
                let node_balance = match **left {
                    Node::Branch {
                        ref mut balance_factor,
                        ..
                    } => balance_factor,
                    Node::Leaf => unreachable!(),
                };
                if (*balance_factor).clone() != 0 {
                    *node_balance = 0;
                    *balance_factor = 0;
                } else {
                    *node_balance = 1;
                    *balance_factor = -1;
                }
            }
            Node::Leaf => unreachable!(),
        }

        *right_node
    }

    fn rotate_right(&mut self) -> Node<K, V> {
        let mut left_node = self.get_left_child();
        let right_left_node = left_node.get_right_child();

        let new_right_left_node = match *left_node {
            Node::Branch { ref mut right, .. } => {
                *right = Box::new(self.clone());
                right
            }
            Node::Leaf => {
                unreachable!()
            }
        };

        new_right_left_node.set_left_child(right_left_node);

        match *left_node {
            Node::Branch {
                ref mut right,
                ref mut balance_factor,
                ..
            } => {
                let node_balance = match **right {
                    Node::Branch {
                        ref mut balance_factor,
                        ..
                    } => balance_factor,
                    Node::Leaf => unreachable!(),
                };
                if (*balance_factor).clone() != 0 {
                    *node_balance = 0;
                    *balance_factor = 0;
                } else {
                    *node_balance = -1;
                    *balance_factor = 1;
                }
            }
            Node::Leaf => unreachable!(),
        }

        *left_node
    }

    fn big_rotate_left(&mut self) -> Node<K, V> {
        let mut right_node = self.get_right_child();
        let mut left_right_node = right_node.get_left_child();
        let left_left_right_node = left_right_node.get_left_child();
        let right_left_right_node = left_right_node.get_right_child();

        let (new_left_left_right_node, new_right_left_right_node) = match *left_right_node {
            Node::Branch {
                ref mut left,
                ref mut right,
                ..
            } => {
                *left = Box::new(self.clone());
                *right = Box::new((*right_node).clone());
                (left, right)
            }
            Node::Leaf => {
                unreachable!()
            }
        };

        new_left_left_right_node.set_right_child(left_left_right_node);
        new_right_left_right_node.set_left_child(right_left_right_node);

        match *left_right_node {
            Node::Branch {
                ref mut left,
                ref mut right,
                ref mut balance_factor,
                ..
            } => {
                let node_balance = match **left {
                    Node::Branch {
                        ref mut balance_factor,
                        ..
                    } => balance_factor,
                    Node::Leaf => unreachable!(),
                };
                let right_node_balance = match **right {
                    Node::Branch {
                        ref mut balance_factor,
                        ..
                    } => balance_factor,
                    Node::Leaf => unreachable!(),
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
            Node::Leaf => unreachable!(),
        }

        *left_right_node
    }

    fn big_rotate_right(&mut self) -> Node<K, V> {
        let mut left_node = self.get_left_child();
        let mut right_left_node = left_node.get_right_child();
        let left_right_left_node = right_left_node.get_left_child();
        let right_right_left_node = right_left_node.get_right_child();

        let (new_left_right_left_node, new_right_right_left_node) = match *right_left_node {
            Node::Branch {
                ref mut left,
                ref mut right,
                ..
            } => {
                *left = Box::new((*left_node).clone());
                *right = Box::new(self.clone());
                (left, right)
            }
            Node::Leaf => {
                unreachable!()
            }
        };

        new_left_right_left_node.set_right_child(left_right_left_node);
        new_right_right_left_node.set_left_child(right_right_left_node);

        match *right_left_node {
            Node::Branch {
                ref mut left,
                ref mut right,
                ref mut balance_factor,
                ..
            } => {
                let left_node_balance = match **left {
                    Node::Branch {
                        ref mut balance_factor,
                        ..
                    } => balance_factor,
                    Node::Leaf => unreachable!(),
                };
                let node_balance = match **right {
                    Node::Branch {
                        ref mut balance_factor,
                        ..
                    } => balance_factor,
                    Node::Leaf => unreachable!(),
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
            Node::Leaf => unreachable!(),
        }

        *right_left_node
    }

    fn set_right_child(&mut self, node: Box<Node<K, V>>) {
        match self {
            Node::Branch { ref mut right, .. } => *right = Box::new(*node),
            Node::Leaf => unreachable!(),
        }
    }

    fn set_left_child(&mut self, node: Box<Node<K, V>>) {
        match self {
            Node::Branch { ref mut left, .. } => *left = Box::new(*node),
            Node::Leaf => unreachable!(),
        }
    }

    fn get_right_child(&mut self) -> Box<Node<K, V>> {
        match *self {
            Node::Branch { ref mut right, .. } => (*right).clone(),
            Node::Leaf => Box::new(Node::Leaf),
        }
    }

    fn get_left_child(&mut self) -> Box<Node<K, V>> {
        match *self {
            Node::Branch { ref mut left, .. } => (*left).clone(),
            Node::Leaf => Box::new(Node::Leaf),
        }
    }

    fn balance(&mut self) -> Node<K, V> {
        match *self {
            Node::Leaf => self.clone(),
            Node::Branch {
                ref left,
                ref right,
                ref balance_factor,
                ..
            } => match *balance_factor {
                2 => match **right {
                    Node::Branch {
                        ref balance_factor, ..
                    } => {
                        if *balance_factor >= 0 {
                            self.rotate_left()
                        } else {
                            self.big_rotate_left()
                        }
                    }
                    _ => unreachable!(),
                },
                -2 => match **left {
                    Node::Branch {
                        ref balance_factor, ..
                    } => {
                        if *balance_factor <= 0 {
                            self.rotate_right()
                        } else {
                            self.big_rotate_right()
                        }
                    }
                    _ => unreachable!(),
                },
                _ => self.clone(),
            },
        }
    }

    pub fn print(&self, d: i32) {
        match self {
            Node::Leaf => {}
            Node::Branch {
                key,
                value,
                left,
                right,
                balance_factor,
                ..
            } => {
                left.print(d.clone() + 1);
                for _i in 0..d {
                    print!("    ");
                }
                match value {
                    None => {
                        println!(
                            "{}:({})",
                            serde_json::to_string(&key).unwrap(),
                            balance_factor
                        )
                    }
                    Some(v) => {
                        println!(
                            "{}:{}({})",
                            serde_json::to_string(&key).unwrap(),
                            serde_json::to_string(v).unwrap(),
                            balance_factor
                        )
                    }
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
                    Some(v) => {
                        format!(
                            "{}:{}\n",
                            serde_json::to_string(&key).unwrap(),
                            serde_json::to_string(&v).unwrap()
                        )
                    }
                    None => format!("{}:\n", serde_json::to_string(&key).unwrap()),
                };
                if (*value).is_some() {
                    let _ = file.write_all(text.as_bytes());
                    *value = None;
                    *sstable_number = number_of_unloads;
                }
                right.unload_to_file(file, number_of_unloads);
            }
        }
    }
}

pub struct LsmTreeIterator<K: Clone, V: Clone> {
    stack: Vec<Node<K, V>>,
    id: String,
}

impl<K, V> LsmTreeIterator<K, V>
where
    K: Clone,
    V: Clone,
{
    fn new(root: Node<K, V>, id: String) -> Self {
        let mut stack = Vec::new();
        Self::push_left(&mut stack, root);
        LsmTreeIterator { stack, id }
    }

    fn push_left(stack: &mut Vec<Node<K, V>>, node: Node<K, V>) {
        let mut current = &node;
        while let Node::Branch { left, .. } = current {
            stack.push((*current).clone());
            current = left;
        }
    }
}

impl<'a, K, V> Iterator for LsmTreeIterator<K, V>
where
    K: Ord + Clone + Serialize + for<'b> Deserialize<'b>,
    V: Clone + Serialize + for<'b> Deserialize<'b>,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = &self.stack.pop() {
            if let Node::Branch {
                key,
                value,
                right,
                sstable_number,
                ..
            } = node
            {
                let new_value = match value.clone() {
                    None => {
                        if *sstable_number != 0 {
                            LsmTree::get_from_table(key.clone(), *sstable_number, self.id.clone())
                                .unwrap()
                        } else {
                            unreachable!()
                        }
                    }
                    Some(value) => value,
                };
                let result = Some(((*key).clone(), new_value));
                Self::push_left(&mut self.stack, (**right).clone());
                result
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<K, V> IntoIterator for LsmTree<K, V>
where
    K: Ord + Clone + Serialize + for<'b> Deserialize<'b>,
    V: Clone + Serialize + for<'b> Deserialize<'b>,
{
    type Item = (K, V);
    type IntoIter = LsmTreeIterator<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        LsmTreeIterator::new(self.root.clone(), self.identifier.clone())
    }
}

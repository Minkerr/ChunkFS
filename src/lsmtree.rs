use std::cmp::Ordering;

#[derive(Debug)]
pub struct AvlTree<K, V> {
    root: Node<K, V>,
}

#[derive(Debug)]
#[derive(Clone)]
enum Node<K, V> {
    Leaf,
    Branch {
        key: K,
        value: V,
        left: Box<Node<K, V>>,
        right: Box<Node<K, V>>,
        balance_factor: i8,
    },
}


impl<K: Ord + Clone, V: Clone> AvlTree<K, V> {
    pub fn new() -> Self {
        AvlTree { root: Node::Leaf }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let mut climb = Box::new(true);
        self.root = Node::insert(&mut self.root, key, value, &mut climb);
    }

    pub fn get(&self, key: K) -> Option<V> {
        Node::get(&self.root, key)
    }

    pub fn traversal(&self, f: &mut dyn FnMut(&K, &V)) {
        Node::traversal(&self.root, f)
    }

    pub fn print(&self, f: &mut dyn FnMut(&K, &V, &i8)) {
        Node::print(&self.root, f, 0)
    }
}

impl<K: Ord + Clone, V: Clone> Node<K, V> {
    fn insert(node: &mut Node<K, V>, new_key: K, new_value: V, should_climb: &mut Box<bool>) -> Node<K, V> {
        match node {
            Node::Leaf => Node::Branch {
                key: new_key,
                value: new_value,
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
                        *right = Box::new(Node::insert(right, new_key, new_value, should_climb));
                        *balance_factor = (*balance_factor).clone() + 1;
                    }
                    Ordering::Less => {
                        *left = Box::new(Node::insert(left, new_key, new_value, should_climb));
                        *balance_factor = (*balance_factor).clone() - 1;
                    }
                    Ordering::Equal => {
                        **should_climb = false;
                        return node.clone();
                    }
                }
                if !**should_climb {
                    if comparison_result == Ordering::Less {
                        *balance_factor += 1;
                    } else {
                        *balance_factor -= 1;
                    }
                    return node.clone();
                }
                let bf = (*balance_factor).clone();
                if bf == -2 || bf == 0 || bf == 2 {
                    **should_climb = false;
                }

                Node::balance(node)
            }
        }
    }

    fn get(node: &Node<K, V>, target_key: K) -> Option<V> {
        match node {
            Node::Leaf => None,
            Node::Branch {
                ref key,
                ref value,
                ref left,
                ref right,
                ..
            } => match target_key.cmp(&key) {
                Ordering::Greater => Node::get(right, target_key),
                Ordering::Less => Node::get(left, target_key),
                Ordering::Equal => Some(value.clone()),
            },
        }
    }

    fn rotate_left(node: &mut Node<K, V>) -> Node<K, V> {
        let mut right_node = match *node {
            Node::Branch { ref mut right, .. } => {
                (*right).clone()
            }
            Node::Leaf => unreachable!()
        };
        let left_right_node = match *right_node {
            Node::Branch { ref mut left, .. } => {
                (*left).clone()
            }
            Node::Leaf => { Box::new(Node::Leaf) }
        };

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
        let mut left_node = match *node {
            Node::Branch { ref mut left, .. } => {
                (*left).clone()
            }
            Node::Leaf => unreachable!()
        };
        let right_left_node = match *left_node {
            Node::Branch { ref mut right, .. } => {
                (*right).clone()
            }
            Node::Leaf => { Box::new(Node::Leaf) }
        };

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
        let mut right_node = match *node {
            Node::Branch { ref mut right, .. } => {
                (*right).clone()
            }
            Node::Leaf => unreachable!()
        };
        let mut  left_right_node = match *right_node {
            Node::Branch { ref mut left, .. } => {
                (*left).clone()
            }
            Node::Leaf => unreachable!()
        };
        let left_left_right_node = match *left_right_node {
            Node::Branch { ref mut left, .. } => {
                (*left).clone()
            }
            Node::Leaf => { Box::new(Node::Leaf) }
        };
        let right_left_right_node = match *left_right_node {
            Node::Branch { ref mut right, .. } => {
                (*right).clone()
            }
            Node::Leaf => { Box::new(Node::Leaf) }
        };

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
                    0  => {
                        *node_balance = 0;
                        *right_node_balance = 0;
                    }
                    1  => {
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
        let mut left_node = match *node {
            Node::Branch { ref mut left, .. } => {
                (*left).clone()
            }
            Node::Leaf => unreachable!()
        };
        let mut right_left_node = match *left_node {
            Node::Branch { ref mut right, .. } => {
                (*right).clone()
            }
            Node::Leaf => unreachable!()
        };
        let left_right_left_node = match *right_left_node {
            Node::Branch { ref mut left, .. } => {
                (*left).clone()
            }
            Node::Leaf => { Box::new(Node::Leaf) }
        };
        let right_right_left_node = match *right_left_node {
            Node::Branch { ref mut right, .. } => {
                (*right).clone()
            }
            Node::Leaf => { Box::new(Node::Leaf) }
        };

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
                    0  => {
                        *node_balance = 0;
                        *left_node_balance = 0;
                    }
                    1  => {
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
                                return Node::rotate_left(node);
                            },
                            _ => unreachable!()
                        }
                        return Node::big_rotate_left(node);
                    }
                    -2 => {
                        match **left {
                            Node::Branch { ref balance_factor, .. }
                            => if *balance_factor <= 0 {
                                return Node::rotate_right(node);
                            },
                            _ => {}
                        }
                        return Node::big_rotate_right(node);
                    }
                    _ => { node.clone() }
                }
            }
        }
    }

    pub fn traversal(&self, f: &mut dyn FnMut(&K, &V)) {
        match self {
            Node::Leaf => {}
            Node::Branch { key, value, left, right, .. } => {
                f(key, value);
                left.traversal(f);
                right.traversal(f);
            }
        }
    }

    pub fn print(&self, f: &mut dyn FnMut(&K, &V, &i8), d: i32) {
        match self {
            Node::Leaf => {}
            Node::Branch { key, value, left, right, balance_factor } => {
                left.print(f, d.clone() + 1);
                for _i in 0..d {
                    print!("    ");
                }
                f(key, value, balance_factor);
                right.print(f, d.clone() + 1);
            }
        }
    }
}


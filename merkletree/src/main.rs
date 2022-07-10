use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn main() {
    let names = ["kari", "ammar", "ammarif", "alamo", "snarky"];
    let mut nodes: Vec<Box<Node>> = Vec::new();

    for name in names.iter() {
        let new_node = Node::new_from(calculate_hash(name));
        nodes.push(Box::new(new_node));
    }

    while nodes.len() > 1 {
        let mut parents: Vec<Box<Node>> = Vec::new();
        let mut n = 0;

        while n < nodes.len() {
            let mut new_node = Node::new();
            new_node.left = Some(nodes[n].clone());

            if n + 1 < nodes.len() {
                new_node.right = Some(nodes[n + 1].clone());
            } else {
                new_node.right = new_node.left.clone();
            }

            new_node.compute_hash();
            parents.push(Box::new(new_node));

            n += 2;
        }

        nodes = parents;
    }

    println!("{:?}", nodes[0].hash());
}

#[allow(unused)]
fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[derive(Debug, Clone)]
struct Node {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
    hash: Option<u64>,
}

#[allow(unused)]
impl Node {
    fn new() -> Node {
        Node {
            left: None,
            right: None,
            hash: None,
        }
    }

    fn new_from(hash: u64) -> Node {
        Node {
            left: None,
            right: None,
            hash: Some(hash),
        }
    }

    fn compute_hash(&mut self) {
        match (&self.left, &self.right) {
            (Some(left), Some(right)) => {
                let mut s = DefaultHasher::new();

                if let (Some(lh), Some(rh)) = (&left.hash, &right.hash) {
                    lh.hash(&mut s);
                    rh.hash(&mut s);

                    self.hash = Some(s.finish());
                } else {
                    self.hash = None;
                }
            }

            _ => self.hash = None,
        }
    }

    fn hash(&self) -> u64 {
        match self.hash {
            Some(h) => h,

            None => 0,
        }
    }
}

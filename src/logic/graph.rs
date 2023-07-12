use core::cmp::PartialEq;
use core::hash::Hash;
use std::collections::hash_map::{DefaultHasher, Keys};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::vec::Vec;

#[derive(Clone, Debug)]
pub struct GraphLoop(Vec<usize>);

impl GraphLoop {
    pub fn new() -> Self {
        GraphLoop(vec![])
    }
}

impl Deref for GraphLoop {
    type Target = Vec<usize>;
    fn deref(&self) -> &Vec<usize> {
        &self.0
    }
}

impl DerefMut for GraphLoop {
    fn deref_mut(&mut self) -> &mut Vec<usize> {
        &mut self.0
    }
}

impl From<Vec<usize>> for GraphLoop {
    fn from(vec: Vec<usize>) -> Self {
        return GraphLoop(vec);
    }
}

impl PartialEq for GraphLoop {
    fn eq(&self, other: &Self) -> bool {
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        return self.0.hash(&mut hasher1) == other.hash(&mut hasher2);
    }
}

impl Eq for GraphLoop {}

// TODO: make this more efficient through caching/
// make this class a proper wrapper and only calculate when the vector is modified
// This is massively over-complicated
impl Hash for GraphLoop {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let (mut i, n) = (0i8..)
            .zip(self.0.iter())
            .min_by(|(_, a), (_, b)| a.cmp(b))
            .unwrap();
        let dir: i8;
        // ensure consistent equality in either direction by hashing in the 'upwards' direction around the min point
        if self.0.len() > 2 {
            let right = self.0[((i + 1) % (self.0.len() as i8)) as usize];
            let left: usize;
            if i - 1 < 0 {
                left = *self.0.last().unwrap();
            } else {
                left = self.0[i as usize - 1];
            }
            if right > left {
                dir = 1
            } else {
                dir = -1;
            }
        } else {
            dir = 1;
        }
        let mut j = 0;
        loop {
            state.write_usize(self.0[i as usize]);
            i = i + 1 * dir;
            if i < 0 {
                i = self.0.len() as i8 - 1;
            } else if i > (self.0.len() as i8) - 1 {
                i = 0;
            }
            j += 1;
            if j == self.0.len() {
                break;
            }
        }
    }
}

#[derive(Debug)]
pub struct Graph {
    adjacency_list: HashMap<usize, Vec<usize>>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            adjacency_list: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, connections: Vec<usize>) -> &usize {
        let id = self.adjacency_list.len();
        for conn in &connections {
            self.add_edge(id, *conn);
        }
        self.adjacency_list.insert(id, connections);
        return self.adjacency_list.get_key_value(&id).unwrap().0;
    }

    pub fn add_edge(&mut self, a: usize, b: usize) {
        if let Some(a_adjacent) = self.adjacency_list.get_mut(&a) {
            a_adjacent.push(b);
        }
        if let Some(b_adjacent) = self.adjacency_list.get_mut(&b) {
            b_adjacent.push(a);
        }
    }

    pub fn neighbours(&self, node: usize) -> &Vec<usize> {
        return self.adjacency_list.get(&node).unwrap();
    }

    pub fn edge_count(&self, node: usize) -> usize {
        return self.adjacency_list.get(&node).unwrap().len();
    }

    pub fn iter_nodes(&self) -> Keys<'_, usize, Vec<usize>> {
        return self.adjacency_list.keys();
    }

    pub fn has_node(&self, node: usize) -> bool {
        return self.adjacency_list.contains_key(&node);
    }
}

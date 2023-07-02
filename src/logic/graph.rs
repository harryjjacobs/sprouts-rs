use std::collections::HashMap;
use std::collections::hash_map::Keys;
use std::vec::Vec;

pub struct Graph {
    adjacency_list: HashMap<usize, Vec<usize>>,
}

impl Graph { 
    pub fn new() -> Graph {
        Graph { adjacency_list: HashMap::new() }
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

use std::collections::HashSet;

use crate::logic::graph::{Graph, GraphLoop};

use log::warn;
use sdl2::rect::Rect;

pub struct Player {
    id: u8,
    name: String,
}

impl Player {
    pub fn new(id: u8, name: String) -> Player {
        Player { id, name }
    }
}

pub enum GameState {
    ACTIVE,
    END,
}

pub struct Game {
    graph: Graph,
    player1: Player,
    player2: Player,
    turn: u8,
    pub state: GameState,
    winner: u8,
}

impl Game {
    pub fn new(player1: Player, player2: Player, n: usize) -> Game {
        let mut graph = Graph::new();

        for _i in 0..n {
            graph.add_node(vec![]);
        }

        let player1_id = player1.id.clone();
        let game = Game {
            graph,
            player1,
            player2,
            turn: player1_id,
            state: GameState::ACTIVE,
            winner: 0,
        };

        return game;
    }

    pub fn get_nodes(&self) -> Vec<usize> {
        return Vec::from_iter(self.graph.iter_nodes().cloned());
    }

    pub fn do_turn(&mut self, a: usize, b: usize) -> Option<usize> {
        // check if valid turn
        if !self.can_connect_nodes(a, b) {
            return None;
        }
        return Some(self.connect_nodes(a, b));
    }

    pub fn end_turn(
        &mut self,
        point_in_polygon: impl Fn(usize, Vec<usize>) -> bool,
        loop_size: impl Fn(Vec<usize>) -> Rect,
    ) {
        if self.check_endstate(point_in_polygon, loop_size) {
            self.winner = self.turn;
            self.state = GameState::END;
            println!("Game Over. Player {} won", self.winner);
        } else {
            if self.turn == self.player1.id {
                self.turn = self.player2.id;
            } else {
                self.turn = self.player1.id;
            }
        }
    }

    pub fn is_node_alive(&self, node: usize) -> bool {
        return self.graph.edge_count(node) < 3;
    }

    fn can_connect_nodes(&self, a: usize, b: usize) -> bool {
        if !self.graph.has_node(a) || !self.graph.has_node(b) {
            warn!("Invalid node passed to can_connect");
            return false;
        }
        // can only connect to self if have 0 or 1 edges
        if a == b && self.graph.edge_count(a) >= 2 {
            return false;
        }
        if !self.is_node_alive(a) || !self.is_node_alive(b) {
            return false;
        }
        return true;
    }

    fn connect_nodes(&mut self, a: usize, b: usize) -> usize {
        // add an edge between two nodes and put a node in the middle
        return *self.graph.add_node(vec![a, b]);
    }

    fn check_endstate(
        &self,
        point_in_polygon: impl Fn(usize, Vec<usize>) -> bool,
        loop_size: impl Fn(Vec<usize>) -> Rect,
    ) -> bool {
        // are there any nodes with 0 or 1 edges? These can at least connect to themselves
        for n in self.graph.iter_nodes() {
            let edge_count = self.graph.edge_count(*n);
            if edge_count < 2 {
                return false; // node could be connected to itself
            }
        }

        let polygons = self.get_loops();

        let smallest_enclosing_polygon = |n: usize| {
            let mut smallest_polygon: Option<&GraphLoop> = None;
            let mut smallest_size: u32 = u32::MAX;
            for polygon in polygons.iter() {
                if polygon.contains(&n) {
                    continue;
                }
                // TODO: the .to_vec()s below aren't great
                println!("Point in polygon? {}", n);
                if point_in_polygon(n, polygon.to_vec()) {
                    let rect = loop_size(polygon.to_vec());
                    let size = rect.width() * rect.height();
                    if size < smallest_size {
                        smallest_polygon = Some(polygon);
                        smallest_size = size;
                    }
                }
            }
            println!("{:?}", smallest_polygon);
            return smallest_polygon;
        };

        let active_nodes = Vec::from_iter(
            self.graph
                .iter_nodes()
                .filter(|&&n| self.graph.edge_count(n) < 3),
        );
        for active_node in active_nodes.iter() {
            println!("Active node: {}", active_node);
            let sep = smallest_enclosing_polygon(**active_node);
            println!("{:?}", sep);
            if sep.is_some() {
                for n in sep.unwrap().to_vec() {
                    if self.graph.edge_count(n) < 3 {
                        // node on smallest enclosing polygon is active therefore connectable
                        return false;
                    }
                }
            } else if active_nodes
                .iter()
                .filter(|&n| **n != **active_node)
                .any(|&n| smallest_enclosing_polygon(*n).is_none())
            {
                // at least 2 active nodes on a non-enclosed polygon (on the outside)
                return false;
            }
        }

        return true;
    }

    fn get_loops(&self) -> Vec<GraphLoop> {
        let mut graph_loops: Vec<GraphLoop> = vec![];

        // iterate over each node and find loops
        for n in self.get_nodes().iter() {
            println!("{}'s neighbours: {:?}", n, self.graph.neighbours(*n));
            self.get_loops_recursive(
                &mut graph_loops,
                &mut GraphLoop::from(vec![*n]),
                &mut HashSet::new(),
            );
        }

        // remove duplicates
        let unique_graph_loops: HashSet<GraphLoop> = HashSet::from_iter(graph_loops);

        // TODO: just for debugging. remove me
        for p in unique_graph_loops.iter() {
            println!("{:?}", p);
        }

        return unique_graph_loops.into_iter().collect::<Vec<GraphLoop>>();
    }

    fn get_loops_recursive(
        &self,
        loops: &mut Vec<GraphLoop>,
        current_loop: &mut GraphLoop,
        visited: &mut HashSet<usize>,
    ) {
        let current = *current_loop.last().unwrap();
        println!("Visiting node {}", current);
        visited.insert(current);
        for neighbour in self.graph.neighbours(current) {
            println!("Found neighbour {}", neighbour);
            if !visited.contains(neighbour) {
                let mut new_loop = current_loop.clone();
                new_loop.push(*neighbour);
                self.get_loops_recursive(loops, &mut new_loop, visited);
            } else if *current_loop.first().unwrap() == *neighbour && current_loop.len() >= 3 {
                // loop complete.
                loops.push(current_loop.clone());
                println!("Loop found: {:?}", current_loop);
                break;
            } else {
                println!("Ignoring neighbour (already seen)");
                continue;
            }
        }
    }
}

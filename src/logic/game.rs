use crate::logic::graph::Graph;

use log::{warn};

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
        let game = Game { graph, player1, player2, turn: player1_id, state: GameState::ACTIVE, winner: 0, };
    
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
        let result = self.connect_nodes(a, b);
        self.end_turn();
        return Some(result);
    }
    
    fn end_turn(&mut self) {
        if self.check_endstate() {
            self.winner = self.turn;
            self.state = GameState::END;
            println!("Game Over. Player {} won", self.winner)
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

    fn check_endstate(&self) -> bool {
        // are there any nodes with 0 or 1 edges? These can at least connect to themselves
        for n in self.graph.iter_nodes() {
            let edge_count = self.graph.edge_count(*n);
            if edge_count < 2 {
                return false; // node could be connected to itself
            }
            // TODO: implement logic for detecting end of game
        }
        return true;
    }
}

use std::f32::consts::PI;
use std::ops::{Mul, Div};
use std::vec::Vec;
use std::cmp::min;
use std::collections::HashMap;
use sdl2::{mouse::MouseButton, gfx::primitives::DrawRenderer};
use sdl2::video::Window;
use sdl2::render::Canvas;
use sdl2::rect::Point;
use sdl2::event::Event;
use sdl2::pixels::Color;
use log::{warn};
use crate::logic::game::Game;

const NODE_RADIUS: i32 = 15;

struct UINode {
    focused: bool,
    pos: Point,
}

impl UINode {
    pub fn new() -> UINode {
        UINode { focused: false, pos: Point::new(0, 0) }
    }

    pub fn at_position(pos: Point) -> UINode {
        UINode { focused: false, pos, }
    }
}

pub struct UI {
    canvas: Canvas<Window>,
    nodes: HashMap<usize, UINode>,
    edges: Vec<Vec<Point>>,
    drawing: bool,
    drawing_start: usize,
    drawing_edge: Vec<Point>,
    mouse_pos: Point,
}

impl UI {
    pub fn new(window: Window, nodes: Vec<usize>) -> UI {
        let mut canvas = window.into_canvas().build().unwrap();
        let res = canvas.set_logical_size(800, 600);
        if !res.is_ok() {
            warn!("{}", res.unwrap_err());
        }
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();
        canvas.present();

        let nodes_map = nodes.iter().map(|n| (n.clone(), UINode::new()));
        let mut ui = UI { 
            canvas, 
            nodes: nodes_map.collect::<HashMap<_, _>>(), 
            drawing: false, 
            drawing_start: 0, 
            drawing_edge: Vec::new(), 
            mouse_pos: Point::new(0, 0), 
            edges: Vec::new(), 
        };
        ui.auto_layout_nodes();
        return ui;
    }

    pub fn process(&mut self, event: Event, game: &mut Game) {
        match event {
            Event::MouseButtonDown { x, y, mouse_btn, .. } => {
                if mouse_btn == MouseButton::Left {
                    if let Some(node) = self.find_node_at(x, y) {
                        if !self.drawing {
                            self.drawing = true;
                            self.drawing_start = node;
                            self.drawing_edge.push(Point::new(x, y));
                        }
                    } else {
                        self.drawing = false;
                    }
                }
            },
            Event::MouseButtonUp { mouse_btn, x, y, .. } => {
                if mouse_btn == MouseButton::Left {
                    if let Some(node) = self.find_node_at(x, y) {
                        if self.drawing {
                            if let Some(new_node) = game.do_turn(self.drawing_start, node) {
                                self.edges.push(self.drawing_edge.clone());
                                let new_pos = self.bisect_pos(&self.drawing_edge);
                                self.nodes.insert(new_node, UINode::at_position(new_pos));
                            }
                        }
                    }
                    self.drawing = false;
                    self.drawing_edge.clear();
                }
            },
            Event::MouseMotion { x, y, .. } => {
                self.mouse_pos.x = x;
                self.mouse_pos.y = y;
                // unhighlight all nodes
                for node in self.nodes.values_mut() {
                   node.focused = false; 
                }
                // highlight node if alive
                if let Some(node) = self.find_node_at(x, y) {
                    if game.is_node_alive(node) {
                        self.nodes.get_mut(&node).unwrap().focused = true;
                    }                   
                }
                // update edge path
                if self.drawing {
                    self.drawing_edge.push(self.mouse_pos);
                    Self::refine_edge(&mut self.drawing_edge);
                }
            },
            _ => {}
        }
    }

    pub fn render(&mut self) {
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        self.canvas.clear();
        
        for node in self.nodes.values() {
            let color;
            if node.focused {
                color = Color::RGB(50, 50, 50);
            } else {
                color = Color::RGB(0, 0, 0);
            }
            let _ = self.canvas.filled_circle(node.pos.x as i16, node.pos.y as i16, NODE_RADIUS as i16, color);
        }

        let draw_edge = |line: &Vec<Point>| {
             for i in 0..line.len()-1 {
                let start_pos = &line[i];
                let end_pos = &line[i + 1];
                let _ = self.canvas.thick_line(start_pos.x as i16, start_pos.y as i16, end_pos.x as i16, end_pos.y as i16, 8, Color::RGB(0, 0, 0));
            }
        };

        if self.drawing {
            draw_edge(&self.drawing_edge);
        }

        for edge in &self.edges {
            draw_edge(&edge);
        }

        self.canvas.present();
    }

    fn find_node_at(&self, x: i32, y: i32) -> Option<usize> {
        for (id, node) in self.nodes.iter() {
            let radius = NODE_RADIUS;
            if x > node.pos.x - radius && x < node.pos.x + radius {
                if y > node.pos.y - radius && y < node.pos.y + radius {
                    return Some(id.clone());
                }
            }
        }
        return None;
    }

    fn auto_layout_nodes(&mut self) {
        let (width, height) = self.canvas.logical_size();
        let center = Point::new(width as i32 / 2, height as i32 / 2);
        let margin = 150.0;
        let radius = min(width, height) as f32 / 2.0 - margin; 
        let count = self.nodes.len();
        for (i, node) in self.nodes.values_mut().enumerate() {
            let t = 2.0 * PI * i as f32 / (count) as f32;
            let x = center.x + (t.sin() * radius) as i32;
            let y = center.y + (t.cos() * radius) as i32;
            node.pos = Point::new(x, y);
        }
    }

    fn bisect_pos(&self, edge: &Vec<Point>) -> Point {
        // TODO: edges won't always be straight lines
        return edge[edge.len() / 2];
        
    }

    fn refine_edge(edge: &mut Vec<Point>) {
        // reduce line resolution by resampling at a fixed distance
        const MIN_DISTANCE: i32 = 1000;
        let mut i = 0;
        loop {
            if i >= edge.len() - 2 {
                break;
            }
            let j = i + 1;
            loop {
                if j >= edge.len() - 2 {
                    break;
                }
                let dist = Self::point_distance(&edge[i], &edge[j]);
                if dist < MIN_DISTANCE {
                    edge.remove(j);
                } else {
                    break;
                }
            }
            i = j;
        }
        // smoothing
        // let iter = 3;
        // for _n in 0..iter {
        //     let mut i = 1;
        //     loop {
        //         if i >= edge.len() - 2 {
        //             break;
        //         }
        //         // take two new points at 1/4 and 3/4 of each segment between i and i+1
        //         // and replace i and i+1 with them.
        //         let q = edge[i].mul(3).div(4) + edge[i + 1].div(4);
        //         let p = edge[i].div(4) + edge[i + 1].mul(3).div(4);
        //         println!("({},{}, ({},{})", q.x, q.y, p.x, p.y);
        //         edge.insert(i + 1, q);
        //         edge.insert(i + 2, p);
        //         if i > 4 {
        //             // remove old points from previous refinement
        //             edge.remove(i - 3);
        //         }
        //         i += 3;
        //     }
        // }
    }

    fn point_distance(a: &Point, b: &Point) -> i32 {
        return (a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y);
    }
}

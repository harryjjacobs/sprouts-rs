use crate::logic::game::{Game, GameState};
use log::warn;
use once_cell::sync::Lazy;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, TextureQuery};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::Window;
use sdl2::{gfx::primitives::DrawRenderer, mouse::MouseButton};
use std::cmp::min;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::vec::Vec;

const NODE_RADIUS: i32 = 15;

static TTF_CONTEXT: Lazy<Sdl2TtfContext> =
    Lazy::new(|| return sdl2::ttf::init().map_err(|e| e.to_string()).unwrap());

struct UINode {
    focused: bool,
    pos: Point,
}

impl UINode {
    pub fn new() -> UINode {
        UINode {
            focused: false,
            pos: Point::new(0, 0),
        }
    }

    pub fn at_position(pos: Point) -> UINode {
        UINode {
            focused: false,
            pos,
        }
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
    font: sdl2::ttf::Font<'static, 'static>,
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
            font: TTF_CONTEXT
                .load_font("./assets/UbuntuNerdFont-Medium.ttf", 25)
                .unwrap(),
        };
        ui.auto_layout_nodes();
        return ui;
    }

    pub fn process(&mut self, event: Event, game: &mut Game) {
        if !matches!(game.state, GameState::ACTIVE) {
            return;
        }

        match event {
            Event::MouseButtonDown {
                x, y, mouse_btn, ..
            } => {
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
            }
            Event::MouseButtonUp {
                mouse_btn, x, y, ..
            } => {
                if mouse_btn == MouseButton::Left {
                    if let Some(node) = self.find_node_at(x, y) {
                        if self.drawing {
                            if let Some(new_node) = game.do_turn(self.drawing_start, node) {
                                self.edges.push(self.drawing_edge.clone());
                                let new_pos = self.bisect_pos(&self.drawing_edge);
                                self.nodes.insert(new_node, UINode::at_position(new_pos));
                                // end turn
                                game.end_turn(
                                    |node, polygon| self.point_in_polygon(&node, &polygon),
                                    |nodes| self.point_bounds(&nodes),
                                )
                            }
                        }
                    }
                    self.drawing = false;
                    self.drawing_edge.clear();
                }
            }
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
            }
            _ => {}
        }
    }

    pub fn render(&mut self) {
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        self.canvas.clear();

        for (id, node) in self.nodes.iter() {
            let color;
            if node.focused {
                color = Color::RGB(50, 50, 50);
            } else {
                color = Color::RGB(0, 0, 0);
            }
            let _ = self.canvas.filled_circle(
                node.pos.x as i16,
                node.pos.y as i16,
                NODE_RADIUS as i16,
                color,
            );
            self.canvas.render_text(
                &self.font,
                format!("{}", id),
                node.pos.offset(NODE_RADIUS, NODE_RADIUS),
                Color::RGB(0, 255, 0),
            );
        }

        let draw_edge = |line: &Vec<Point>| {
            for i in 0..line.len() - 1 {
                let start_pos = &line[i];
                let end_pos = &line[i + 1];
                let _ = self.canvas.thick_line(
                    start_pos.x as i16,
                    start_pos.y as i16,
                    end_pos.x as i16,
                    end_pos.y as i16,
                    8,
                    Color::RGB(0, 0, 0),
                );
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

    // https://web.archive.org/web/20130126163405/http://geomalgorithms.com/a03-_inclusion.html
    fn point_in_polygon(&self, point: &usize, polygon: &Vec<usize>) -> bool {
        // winding number method
        let mut wn = 0;
        let point = &self.nodes[point].pos;
        let polygon = polygon
            .iter()
            .map(|id| &self.nodes[id].pos)
            .collect::<Vec<&Point>>();
        for i in 0..polygon.len() - 1 {
            let a = &polygon[i];
            let b = &polygon[i + 1];
            if a.y <= point.y {
                if b.y > point.y {
                    // upward crossing
                    if Self::is_left(a, b, point) > 0 {
                        wn += 1;
                    }
                }
            } else {
                if b.y <= point.y {
                    // downward crossing
                    if Self::is_left(a, b, point) < 0 {
                        wn -= 1;
                    }
                }
            }
        }
        return wn != 0;
    }

    /// returns true if c is left of the line from a to b
    /// See isLeft(): https://web.archive.org/web/20130406084141/http://geomalgorithms.com/a01-_area.html
    fn is_left(a: &Point, b: &Point, c: &Point) -> i32 {
        return (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y);
    }

    /// determines the bounding box of these points
    /// this is different to the polygon length, as it doesn't account
    /// for edges, just the smalest box that can fit around the points
    /// Its used to check whether a loop in the graph could fit inside
    /// another one, when finding the smallest enclosing polygon of a point
    fn point_bounds(&self, nodes: &Vec<usize>) -> Rect {
        let points_vec = nodes
            .iter()
            .map(|n| self.nodes.get(n).unwrap().pos)
            .collect::<Vec<Point>>();
        let points = points_vec.as_slice().try_into().unwrap();
        return Rect::from_enclose_points(points, None).unwrap();
    }
}

trait TextRendering {
    fn render_text(&mut self, font: &Font, text: String, position: Point, color: Color);
}

impl TextRendering for Canvas<Window> {
    fn render_text(&mut self, font: &Font, text: String, position: Point, color: Color) {
        // render a surface, and convert it to a texture bound to the canvas
        let surface = font
            .render(&text)
            .blended(Color::RGBA(255, 0, 0, 255))
            .map_err(|e| e.to_string())
            .unwrap();
        let texture_creator = self.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())
            .unwrap();

        self.set_draw_color(color);

        let TextureQuery { width, height, .. } = texture.query();

        let target = Rect::from_center(position, width, height);

        self.copy(&texture, None, Some(target))
            .map_err(|e| e.to_string())
            .unwrap();
    }
}

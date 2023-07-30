use std::cmp::{max, min};

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, TextureQuery};
use sdl2::ttf::Font;
use sdl2::video::Window;

pub trait TextRendering {
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

/// checks whether edge ab intersects with edge cd
pub fn edges_intersect(a: &Point, b: &Point, c: &Point, d: &Point) -> bool {
    const BOUNDS_FUDGE_FACTOR: i32 = 2;

    let m_ab = (a.y - b.y) as f32 / (a.x - b.x) as f32;
    let m_cd = (c.y - d.y) as f32 / (c.x - d.x) as f32;

    // parallel
    if m_ab == m_cd {
        return false;
    }

    let p: Point;

    // Equation of a straight line won't work with infinite gradient.
    // I could use the cross product for line segment intersection, but wanted
    // to do it from first principles
    if m_ab.is_infinite() {
        let c_cd = c.y as f32 - m_cd * c.x as f32;
        p = Point::new(a.x, (m_cd * a.x as f32 + c_cd) as i32);
    } else if m_cd.is_infinite() {
        let c_ab = a.y as f32 - m_ab * a.x as f32;
        p = Point::new(c.x, (m_ab * c.x as f32 + c_ab) as i32);
    } else {
        let c_ab = a.y as f32 - m_ab * a.x as f32;
        let c_cd = c.y as f32 - m_cd * c.x as f32;

        // intersection point
        p = Point::new(
            ((c_cd - c_ab) / (m_ab - m_cd)) as i32,
            ((m_cd * c_ab - m_ab * c_cd) / (m_cd - m_ab)) as i32,
        );
    }

    if p.x > min(a.x, b.x) - BOUNDS_FUDGE_FACTOR
        && p.x < max(a.x, b.x) + BOUNDS_FUDGE_FACTOR
        && p.x > min(c.x, d.x) - BOUNDS_FUDGE_FACTOR
        && p.x < max(c.x, d.x) + BOUNDS_FUDGE_FACTOR
        && p.y > min(a.y, b.y) - BOUNDS_FUDGE_FACTOR
        && p.y < max(a.y, b.y) + BOUNDS_FUDGE_FACTOR
        && p.y > min(c.y, d.y) - BOUNDS_FUDGE_FACTOR
        && p.y < max(c.y, d.y) + BOUNDS_FUDGE_FACTOR
    {
        return true;
    }

    return false;
}

#[cfg(test)]
mod tests {
    use super::edges_intersect;
    use sdl2::rect::Point;

    #[test]
    fn test_intersect() {
        assert_eq!(
            edges_intersect(
                &Point::new(100, 100),
                &Point::new(500, 400),
                &Point::new(400, 0),
                &Point::new(200, 700),
            ),
            true
        );
        // with vertical line
        assert_eq!(
            edges_intersect(
                &Point::new(20, 10),
                &Point::new(20, 40),
                &Point::new(10, 20),
                &Point::new(30, 30),
            ),
            true
        );
    }
}

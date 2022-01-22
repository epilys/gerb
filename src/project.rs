/*
 * gerb
 *
 * Copyright 2022 - Manos Pitsidianakis
 *
 * This file is part of gerb.
 *
 * gerb is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * gerb is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with gerb. If not, see <http://www.gnu.org/licenses/>.
 */

use gtk::cairo::{Context, FontSlant, FontWeight};
use std::collections::HashMap;
use std::path::PathBuf;
pub type Point = (i64, i64);

#[derive(Debug, Clone)]
pub struct Bezier {
    pub points: Vec<Point>,
}

impl Bezier {
    fn new(points: Vec<Point>) -> Self {
        Bezier { points }
    }

    fn get_point(&self, t: f64) -> Option<Point> {
        draw_curve_point(&self.points, t)
    }
}

fn draw_curve_point(points: &[Point], t: f64) -> Option<Point> {
    if points.is_empty() {
        return None;
    }
    if points.len() == 1 {
        //std::dbg!(points[0]);
        return Some(points[0]);
    }
    let mut new_points = Vec::with_capacity(points.len() - 1);
    for chunk in points.windows(2) {
        let p1 = chunk[0];
        let p2 = chunk[1];
        let x = (1. - t) * (p1.0 as f64) + t * (p2.0 as f64);
        let y = (1. - t) * (p1.1 as f64) + t * (p2.1 as f64);
        new_points.push((x as i64, y as i64));
    }
    assert_eq!(new_points.len(), points.len() - 1);
    draw_curve_point(&new_points, t)
}

#[derive(Debug, Clone)]
pub struct Glyph {
    pub name: &'static str,
    pub char: char,
    pub curves: Vec<Bezier>,
}

impl Default for Glyph {
    fn default() -> Self {
        let curves = vec![
            Bezier::new(vec![(54, 72), (55, 298)]),
            Bezier::new(vec![(27, 328), (61, 333), (55, 299)]),
            Bezier::new(vec![(26, 328), (27, 338)]),
            Bezier::new(vec![(27, 339), (124, 339)]),
            Bezier::new(vec![(98, 306), (97, 209)]),
            Bezier::new(vec![(97, 301), (98, 334), (123, 330)]),
            Bezier::new(vec![(123, 330), (124, 337)]),
            Bezier::new(vec![(12, 53), (54, 55), (53, 72)]),
            Bezier::new(vec![(11, 52), (174, 53)]),
            Bezier::new(vec![(174, 55), (251, 63), (266, 124)]),
            Bezier::new(vec![(183, 192), (265, 182), (266, 127)]),
            Bezier::new(vec![(100, 180), (101, 78)]),
            Bezier::new(vec![(100, 79), (125, 78)]),
            Bezier::new(vec![(126, 79), (209, 67), (216, 120)]),
            Bezier::new(vec![(136, 177), (217, 178), (218, 122)]),
            Bezier::new(vec![(105, 176), (135, 176)]),
            Bezier::new(vec![(96, 209), (138, 209)]),
            Bezier::new(vec![(140, 210), (183, 201), (203, 243)]),
            Bezier::new(vec![(205, 245), (215, 296), (241, 327)]),
            Bezier::new(vec![(187, 192), (244, 197), (252, 237)]),
            Bezier::new(vec![(253, 241), (263, 304), (290, 317)]),
            Bezier::new(vec![(241, 327), (287, 359), (339, 301)]),
            Bezier::new(vec![(292, 317), (316, 318), (332, 294)]),
            Bezier::new(vec![(335, 295), (339, 303)]),
        ];

        Glyph {
            name: "R",
            char: 'R',
            curves,
        }
    }
}

impl Glyph {
    pub fn draw(&self, drar: &gtk::DrawingArea, cr: &gtk::cairo::Context) {
        if self.curves.is_empty() {
            cr.set_source_rgba(0.2, 0.2, 0.2, 0.6);
            cr.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
            cr.set_font_size(0.35);

            cr.move_to(0.04, 0.53);
            cr.show_text(&format!("{}", self.char))
                .expect("Invalid cairo surface state");
        }
        for c in &self.curves {
            cr.set_source_rgba(0.2, 0.2, 0.2, 0.6);
            cr.set_line_width(0.003);
            let mut prev_point = c.points[0];
            let mut sample = 0;
            for t in (0..100).step_by(1) {
                let t = (t as f64) / 100.;
                if let Some(new_point) = c.get_point(t) {
                    if sample == 0 {
                        //println!("{:?} {:?}", prev_point, new_point);
                        cr.move_to(prev_point.0 as f64 / 500., prev_point.1 as f64 / 500.);
                        cr.line_to(new_point.0 as f64 / 500., new_point.1 as f64 / 500.);
                        cr.stroke().expect("Invalid cairo surface state");

                        sample = 5;
                        prev_point = new_point;
                    }
                    sample -= 1;
                }
            }
            let new_point = *c.points.last().unwrap();
            cr.move_to(prev_point.0 as f64 / 500., prev_point.1 as f64 / 500.);
            cr.line_to(new_point.0 as f64 / 500., new_point.1 as f64 / 500.);
            cr.stroke().expect("Invalid cairo surface state");
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_line_width(0.005);
            for &(x, y) in &c.points {
                cr.rectangle(x as f64 / 500., y as f64 / 500., 0.001, 0.001);
                cr.stroke_preserve().expect("Invalid cairo surface state");
            }
        }
    }
}

#[derive(Debug)]
pub struct Project {
    pub name: String,
    pub modified: bool,
    pub last_saved: Option<u64>,
    pub glyphs: HashMap<u32, Glyph>,
    pub path: Option<PathBuf>,
}

impl Default for Project {
    fn default() -> Self {
        Project {
            name: "test project".to_string(),
            modified: false,
            last_saved: None,
            glyphs: [('R' as u32, Glyph::default())]
                .into_iter()
                .collect::<HashMap<u32, Glyph>>(),
            path: None,
        }
    }
}

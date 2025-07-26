mod utils;

use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, ImageData};

#[wasm_bindgen]
unsafe extern "C" {
    unsafe fn alert(s: &str);
}

#[wasm_bindgen]
pub struct Scene {
    ctx: CanvasRenderingContext2d,
    world: Box<[Color]>,
    width: u32,
    height: u32,
    start: (u32, u32),
    end: (u32, u32),
    neighbours: HashMap<(u32, u32), Cell>,
    reached: bool,
    g_cost_multiplier: u32,
    h_cost_multiplier: u32,
}

#[wasm_bindgen]
impl Scene {
    #[wasm_bindgen(constructor)]
    pub fn new(
        start_x: u32,
        start_y: u32,
        end_x: u32,
        end_y: u32,
        g_cost_multiplier: u32,
        h_cost_multiplier: u32,
        ctx: CanvasRenderingContext2d,
    ) -> Self {
        let canvas = ctx.canvas().unwrap();
        let width = canvas.width();
        let height = canvas.height();
        let image_data = ctx
            .get_image_data(0.0, 0.0, width as f64, height as f64)
            .unwrap();
        let world = unsafe {
            image_data
                .data()
                .0
                .align_to::<Color>()
                .1
                .to_vec()
                .into_boxed_slice()
        };

        Self {
            ctx,
            world,
            width,
            height,
            start: (start_x, start_y),
            end: (end_x, end_y),
            neighbours: HashMap::new(),
            reached: false,
            g_cost_multiplier,
            h_cost_multiplier,
        }
    }

    fn get_pixel(&mut self, x: u32, y: u32) -> &Color {
        &self.world[(y * self.width + x) as usize]
    }

    fn get_pixel_mut(&mut self, x: u32, y: u32) -> &mut Color {
        &mut self.world[(y * self.width + x) as usize]
    }

    pub fn init(&mut self) {
        self.neighbours.insert(
            self.start,
            Cell {
                g_cost: 0,
                h_cost: self.calculate_h_cost(self.start.0, self.start.1) * self.h_cost_multiplier,
                from: (0, 0),
                visited: false,
            },
        );
        for pixel in self.world.iter_mut() {
            if *pixel != Color::BLACK {
                *pixel = Color::WHITE;
            }
        }
        *self.get_pixel_mut(self.start.0, self.start.1) = Color::BLUE;

        *self.get_pixel_mut(self.end.0, self.end.1) = Color::BLUE;
    }

    fn run_step(&mut self) {
        let next_cell = match self
            .neighbours
            .iter()
            .filter(|(_, cell)| !cell.visited)
            .min_by(|(_, a), (_, b)| {
                let ord = a.f_cost().cmp(&b.f_cost());
                if ord == std::cmp::Ordering::Equal {
                    a.h_cost.cmp(&b.h_cost)
                } else {
                    ord
                }
            }) {
            Some((pos, _)) => *pos,
            None => {
                self.reached = true;
                return;
            }
        };
        self.add_neighbours(next_cell);
        self.neighbours.get_mut(&next_cell).unwrap().visited = true;
        *self.get_pixel_mut(next_cell.0, next_cell.1) = Color::RED;
        if next_cell == self.end {
            self.reached = true;
            let mut previous_cell = next_cell;
            while previous_cell != self.start {
                *self.get_pixel_mut(previous_cell.0, previous_cell.1) = Color::BLUE;
                previous_cell = self.neighbours[&previous_cell].from;
            }
        }
    }

    fn add_neighbours(&mut self, position: (u32, u32)) {
        let start_g_cost = self.neighbours[&position].g_cost;
        #[rustfmt::skip]
        let neighbours_positions = [
            (position.0.wrapping_add(1), position.1,                 10),
            (position.0.wrapping_add(1), position.1.wrapping_add(1), 14),
            (position.0,                 position.1.wrapping_add(1), 10),
            (position.0.wrapping_sub(1), position.1.wrapping_add(1), 14),
            (position.0.wrapping_sub(1), position.1,                 10),
            (position.0.wrapping_sub(1), position.1.wrapping_sub(1), 14),
            (position.0,                 position.1.wrapping_sub(1), 10),
            (position.0.wrapping_add(1), position.1.wrapping_sub(1), 14),
        ];

        for (x, y, g_cost) in neighbours_positions {
            if x < self.width && y < self.height && *self.get_pixel(x, y) != Color::BLACK {
                let g_cost = start_g_cost + g_cost * self.g_cost_multiplier;
                let mut visited = false;
                if let Some(old_cell) = self.neighbours.get(&(x, y)) {
                    if old_cell.g_cost < g_cost {
                        continue;
                    } else {
                        visited = old_cell.visited;
                    }
                }
                let h_cost = self.calculate_h_cost(x, y) * self.h_cost_multiplier;
                self.neighbours.insert(
                    (x, y),
                    Cell {
                        g_cost,
                        h_cost,
                        from: position,
                        visited,
                    },
                );
                *self.get_pixel_mut(x, y) = if visited { Color::RED } else { Color::GREEN }
            }
        }
    }

    fn calculate_h_cost(&self, x: u32, y: u32) -> u32 {
        let distance = (self.end.0.abs_diff(x), self.end.1.abs_diff(y));
        let min = u32::min(distance.0, distance.1);
        let max = u32::max(distance.0, distance.1);
        min * 14 + (max - min) * 10
    }

    fn draw(&mut self) -> Result<(), JsValue> {
        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(unsafe { self.world.align_to().1 }),
            self.width,
            self.height,
        )
        .unwrap();
        self.ctx.put_image_data(&image_data, 0.0, 0.0)
    }

    pub fn update(&mut self, fast: bool) -> Result<(), JsValue> {
        if fast {
            while !self.reached {
                self.run_step();
            }
        } else if !self.reached {
            self.run_step();
        }
        self.draw()
    }
}

#[wasm_bindgen]
#[derive(Clone, PartialEq)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    const BLACK: Color = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    const GREEN: Color = Self {
        r: 47,
        g: 214,
        b: 72,
        a: 255,
    };
    const BLUE: Color = Self {
        r: 47,
        g: 97,
        b: 214,
        a: 255,
    };
    const RED: Color = Self {
        r: 194,
        g: 31,
        b: 31,
        a: 255,
    };
    const WHITE: Color = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
}

struct Cell {
    g_cost: u32,
    h_cost: u32,
    from: (u32, u32),
    visited: bool,
}

impl Cell {
    fn f_cost(&self) -> u32 {
        self.g_cost + self.h_cost
    }
}

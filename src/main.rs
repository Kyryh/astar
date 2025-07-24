use std::{
    collections::HashMap,
    io::{self, Write as _},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crossterm::{QueueableCommand, cursor, event, style, terminal};

fn main() -> std::io::Result<()> {
    let mut scene = Scene::new((11, 13), (4, 1))?;
    scene.run()
}

struct Scene {
    stdout: io::Stdout,
    world: Box<[u8]>,
    width: u16,
    height: u16,
    start: (u16, u16),
    end: (u16, u16),
    neighbours: HashMap<(u16, u16), Cell>,
    should_update: bool,
    reached: bool,
}

impl Scene {
    pub fn new(start: (u16, u16), end: (u16, u16)) -> io::Result<Self> {
        let mut stdout = io::stdout();
        stdout
            // .queue(terminal::EnterAlternateScreen)?
            .queue(event::EnableMouseCapture)?
            // .queue(event::DisableBracketedPaste)?
            .queue(event::DisableFocusChange)?
            .flush()?;
        let (width, height) = terminal::size()?;
        let world = vec![0; (width * height) as usize].into_boxed_slice();
        Ok(Self {
            stdout,
            world,
            width,
            height,
            start,
            end,
            neighbours: HashMap::new(),
            should_update: true,
            reached: false,
        })
    }

    fn get_cell(&self, x: u16, y: u16) -> &u8 {
        &self.world[(y * self.width + x) as usize]
    }

    fn get_cell_mut(&mut self, x: u16, y: u16) -> &mut u8 {
        &mut self.world[(y * self.width + x) as usize]
    }

    fn update(&mut self, input: &mpsc::Receiver<Input>) -> io::Result<()> {
        for event in input.try_iter() {
            self.should_update = true;
            match event {
                Input::Mouse {
                    x,
                    y,
                    button: event::MouseButton::Left,
                } => *self.get_cell_mut(x, y) = 1,
                Input::Mouse {
                    x,
                    y,
                    button: event::MouseButton::Middle,
                } => *self.get_cell_mut(x, y) = 0,
                _ => {}
            }
        }
        if self.should_update {
            self.should_update = false;
            self.reached = false;
            self.neighbours.clear();
            self.neighbours.insert(
                self.start,
                Cell {
                    g_cost: 0,
                    h_cost: self.calculate_h_cost(self.start.0, self.start.1),
                    from: (0, 0),
                    visited: false,
                },
            );
            for cell in self.world.iter_mut() {
                if *cell == 2 {
                    *cell = 0;
                }
            }
        }
        #[cfg(feature = "fast")]
        while !self.reached {
            self.run_step();
        }
        #[cfg(not(feature = "fast"))]
        if !self.reached {
            self.run_step();
        }

        Ok(())
    }

    fn run_step(&mut self) {
        let next_cell = *self
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
            })
            .unwrap()
            .0;
        self.add_neighbours(next_cell);
        self.neighbours.get_mut(&next_cell).unwrap().visited = true;
        if next_cell == self.end {
            self.reached = true;
            let mut previous_cell = next_cell;
            while previous_cell != self.start {
                *self.get_cell_mut(previous_cell.0, previous_cell.1) = 2;
                previous_cell = self.neighbours[&previous_cell].from;
            }
        }
    }

    fn add_neighbours(&mut self, position: (u16, u16)) {
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
            if x < self.width && y < self.height && *self.get_cell(x, y) == 0 {
                let g_cost = start_g_cost + g_cost;
                let mut visited = false;
                if let Some(old_cell) = self.neighbours.get(&(x, y)) {
                    if old_cell.g_cost < g_cost {
                        continue;
                    } else {
                        visited = old_cell.visited;
                    }
                }
                let h_cost = self.calculate_h_cost(x, y);
                self.neighbours.insert(
                    (x, y),
                    Cell {
                        g_cost,
                        h_cost,
                        from: position,
                        visited,
                    },
                );
            }
        }
    }

    fn calculate_h_cost(&self, x: u16, y: u16) -> u16 {
        let distance = (self.end.0.abs_diff(x), self.end.1.abs_diff(y));
        let min = u16::min(distance.0, distance.1);
        let max = u16::max(distance.0, distance.1);
        min * 14 + (max - min) * 10
    }

    fn draw(&mut self) -> io::Result<()> {
        for x in 0..self.width {
            for y in 0..self.height {
                let color = match *self.get_cell(x, y) {
                    1 => style::Color::Black,
                    2 => style::Color::Blue,
                    _ => {
                        if (x, y) == self.start || (x, y) == self.end {
                            style::Color::Blue
                        } else if let Some(cell) = self.neighbours.get(&(x, y)) {
                            if cell.visited {
                                style::Color::Red
                            } else {
                                style::Color::Green
                            }
                        } else {
                            style::Color::White
                        }
                    }
                };
                self.stdout
                    .queue(cursor::MoveTo(x, y))?
                    .queue(style::SetForegroundColor(color))?
                    .queue(style::Print("█"))?;
            }
        }
        self.stdout.flush()?;
        Ok(())
    }

    pub fn run(&mut self) -> io::Result<()> {
        let (send, recv) = mpsc::channel();
        thread::spawn(|| Self::input_loop(send));
        loop {
            let now = Instant::now();
            self.update(&recv)?;
            self.draw()?;
            spin_sleep::sleep_until(now + Duration::from_millis(50));
        }
    }

    fn input_loop(send: mpsc::Sender<Input>) -> io::Result<()> {
        loop {
            match event::read()? {
                event::Event::Mouse(event::MouseEvent {
                    kind: event::MouseEventKind::Down(button) | event::MouseEventKind::Drag(button),
                    column: x,
                    row: y,
                    ..
                }) => {
                    send.send(Input::Mouse { x, y, button }).unwrap();
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug)]
struct Cell {
    g_cost: u16,
    h_cost: u16,
    from: (u16, u16),
    visited: bool,
}

impl Cell {
    fn f_cost(&self) -> u16 {
        self.g_cost + self.h_cost
    }
}

enum Input {
    Mouse {
        x: u16,
        y: u16,
        button: event::MouseButton,
    },
}

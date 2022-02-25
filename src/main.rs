extern crate glutin_window;
extern crate graphics;
extern crate input;
extern crate opengl_graphics;
extern crate piston;
extern crate rand; // 0.6.5

use glutin_window::GlutinWindow as Window;
use input::*;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateEvent};
use piston::window::WindowSettings;
use rand::Rng;

// Pixel size
const PS: f64 = 30.0;
// Window size (this'll be multiplied by pixel size)
const WS: u32 = 20;

// Background colour
const BC: [f32; 4] = [0.1, 0.05, 0.1, 1.0];
// Snake colour
const SC: [f32; 4] = [0.1, 0.7, 0.2, 1.0];
// Fruit colour
const FC: [f32; 4] = [0.8, 0.2, 0.1, 1.0];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_eat() {
        let snake = Snake::new(1, 1);
        let fruits = vec![Fruit { x: 1, y: 1 }];
        assert_eq!(snake.is_eating(&fruits), true);
    }

    #[test]
    fn can_not_eat() {
        let snake = Snake::new(1, 1);
        let fruits = vec![Fruit { x: 1, y: 2 }];
        assert_ne!(snake.is_eating(&fruits), true);
    }

    #[test]
    fn can_go() {
        let mut snake = Snake::new(1, 1);
        let fruits: Vec<Fruit> = Vec::new();
        snake.go(&fruits);
        assert_eq!(snake.x, 2);
    }

    #[test]
    fn can_grow() {
        let mut snake = Snake::new(1, 1);
        let fruits = vec![Fruit { x: 2, y: 1 }];
        snake.go(&fruits);
        assert_eq!(snake.parts.len(), 2);
    }
}

struct App {
    gl: GlGraphics,
}

impl App {
    fn render(&mut self, args: RenderArgs, snake: &Snake, fruits: &Vec<Fruit>) {
        use graphics::*;

        self.gl.draw(args.viewport(), |c, gl| {
            clear(BC, gl);

            for i in fruits {
                rectangle(
                    FC,
                    rectangle::square(i.x as f64 * PS, i.y as f64 * PS, PS - 1.0),
                    c.transform,
                    gl,
                );
            }

            for i in &snake.parts {
                rectangle(
                    SC,
                    rectangle::square(i.x as f64 * PS, i.y as f64 * PS, PS - 1.0),
                    c.transform,
                    gl,
                );
            }
        });
    }
}

struct Fruit {
    x: i16,
    y: i16,
}

#[derive(Debug)]
struct SnakePart {
    x: i16,
    y: i16,
}

struct Snake {
    parts: Vec<SnakePart>,
    going_horizontally: bool,
    speed: i16,
    x: i16,
    y: i16,
}

impl Snake {
    fn new(x: i16, y: i16) -> Snake {
        Snake {
            parts: vec![SnakePart { x, y }],
            going_horizontally: true,
            speed: 1,
            x,
            y,
        }
    }

    fn go(&mut self, fruits: &Vec<Fruit>) {
        self.parts.reverse();
        if self.going_horizontally {
            self.parts.push(SnakePart {
                x: (self.parts[self.parts.len() - 1].x + self.speed),
                y: (self.parts[self.parts.len() - 1].y),
            });
        } else {
            self.parts.push(SnakePart {
                x: (self.parts[self.parts.len() - 1].x),
                y: (self.parts[self.parts.len() - 1].y + self.speed),
            });
        }
        self.parts.reverse();

        // Update snake's coordinates to coordinates of new head
        self.x = self.parts[0].x;
        self.y = self.parts[0].y;

        if !self.is_eating(fruits) {
            self.parts.pop();
        }
    }

    fn is_eating(&self, fruits: &Vec<Fruit>) -> bool {
        let fruits: Vec<&Fruit> = fruits
            .iter()
            .filter(|fruit| fruit.x == self.x && fruit.y == self.y)
            .collect();
        if fruits.len() > 0 {
            return true;
        }
        false
    }

    fn touched_by(&self, x: i16, y: i16) -> bool {
        let mut result: bool = false;
        for i in &self.parts {
            if i.x == x || i.y == y {
                result = true;
                break;
            };
        }
        result
    }

    fn touches_self(&self) -> bool {
        let mut count = 0;
        for part in &self.parts {
            if self.x == part.x && self.y == part.y {
                count += 1;
            }
        }
        if count > 1 {
            return true;
        }
        false
    }
}

fn randomize_fruits(number: u8) -> Vec<Fruit> {
    let mut fruits: Vec<Fruit> = Vec::new();
    for _ in 0..number {
        let (x, y) = (
            rand::thread_rng().gen_range(0, WS),
            rand::thread_rng().gen_range(0, WS),
        );
        fruits.push(Fruit {
            x: x as i16,
            y: y as i16,
        });
    }
    fruits
}

fn new_random_fruit(fruits: &mut Vec<Fruit>, snake: &Snake) {
    let mut coordinates: Vec<[i16; 2]> = Vec::new();
    for i in 0..WS as i16 {
        for j in 0..WS as i16 {
            coordinates.push([i, j]);
        }
    }
    let coordinates: Vec<[i16; 2]> = coordinates
        .into_iter()
        .filter(|coordinate| {
            for part in &snake.parts {
                if coordinate[0] == part.x && coordinate[1] == part.y {
                    return false;
                }
            }
            for fruit in fruits.into_iter() {
                if coordinate[0] == fruit.x && coordinate[1] == fruit.y {
                    return false;
                }
            }
            true
        })
        .collect();
    let spot = rand::thread_rng().gen_range(0, coordinates.len());
    fruits.push(Fruit {
        x: coordinates[spot][0],
        y: coordinates[spot][1],
    });
}

fn main() -> Result<(), &'static str> {
    let opengl = OpenGL::V3_2;
    let mut window: Window = WindowSettings::new("Falke-Snake", [WS * PS as u32, WS * PS as u32])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();
    let mut app = App {
        gl: GlGraphics::new(opengl),
    };

    let mut snake = Snake::new(1, 1);
    let mut fruits: Vec<Fruit> = randomize_fruits(1);

    let mut event_settings = EventSettings::new();
    event_settings.ups = 8;
    let mut events = Events::new(event_settings);

    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(args, &snake, &fruits);
        }

        if let Some(_) = e.update_args() {
            snake.go(&fruits);
            if snake.touches_self()
                || snake.x < 0
                || snake.x >= WS as i16
                || snake.y < 0
                || snake.y >= WS as i16
            {
                return Err("Snake collided with something");
            }

            for i in 0..fruits.len() {
                if fruits[i].x == snake.x && fruits[i].y == snake.y {
                    fruits.remove(i);
                    new_random_fruit(&mut fruits, &snake);
                }
            }
        }

        match e.press_args() {
            Some(Button::Keyboard(Key::Space)) => println!("Space"),
            Some(Button::Keyboard(Key::Up)) => {
                if snake.going_horizontally == true {
                    snake.speed = -1;
                    snake.going_horizontally = false
                }
            }
            Some(Button::Keyboard(Key::Down)) => {
                if snake.going_horizontally == true {
                    snake.speed = 1;
                    snake.going_horizontally = false
                }
            }
            Some(Button::Keyboard(Key::Left)) => {
                if snake.going_horizontally == false {
                    snake.speed = -1;
                    snake.going_horizontally = true
                }
            }
            Some(Button::Keyboard(Key::Right)) => {
                if snake.going_horizontally == false {
                    snake.speed = 1;
                    snake.going_horizontally = true
                }
            }
            _ => (),
        }
    }
    Ok(())
}

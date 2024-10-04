extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use std::collections::HashSet;
use std::hash::Hash;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateEvent};
use piston::window::WindowSettings;
use piston::{Button, ButtonArgs, ButtonEvent, ButtonState, EventLoop, Key};
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::seq::SliceRandom;
use rand::thread_rng;

const SQUARE_SIZE: i16 = 9;
const WINDOW_WIDTH_SIZE: i16 = 300;
const WINDOW_HEIGHT_SIZE: i16 = 300;

// 生成随机位置,画布面积300x300,方块的边长为9,以步长为9采样创建游戏格子
fn generate_random_position(width: i16, height: i16) -> Position {
    let mut rng = rand::thread_rng();
    let width_range: Vec<i16> = (0..=(width - SQUARE_SIZE))
        .step_by(SQUARE_SIZE.try_into().unwrap())
        .collect();
    let height_range: Vec<i16> = (0..=(height - SQUARE_SIZE))
        .step_by(SQUARE_SIZE.try_into().unwrap())
        .collect();
    Position {
        x: *width_range.choose(&mut rng).unwrap_or(&0),
        y: *height_range.choose(&mut rng).unwrap_or(&0),
    }
}

// 生成画布内所有格子,使用hashset可以快速生成食物的位置(与蛇身集合取差集)
fn generate_all_position(width: i16, height: i16) -> HashSet<Position> {
    let mut all_positions_vec: Vec<Position> = (0..=(width - SQUARE_SIZE))
        .step_by(SQUARE_SIZE.try_into().unwrap())
        .flat_map(|x| {
            (0..=(height - SQUARE_SIZE))
                .step_by(SQUARE_SIZE.try_into().unwrap())
                .map(move |y| Position { x , y })
        })
        .collect();
    all_positions_vec.shuffle(&mut thread_rng());
    all_positions_vec.into_iter().collect()
}

// 蛇身方向检测 不可180°掉头
fn is_direction_conflict(prev_direction: &Direction, new_direction: Direction) -> bool {
    if (matches!(prev_direction, Direction::Up) && matches!(new_direction, Direction::Down)
        || matches!(prev_direction, Direction::Down) && matches!(new_direction, Direction::Up)
        || matches!(prev_direction, Direction::Left) && matches!(new_direction, Direction::Right)
        || matches!(prev_direction, Direction::Right) && matches!(new_direction, Direction::Left))
    {
        return true;
    } else {
        return false;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Position {
    x: i16,
    y: i16,
}

struct Food {
    position: Position,
}

impl Food {
    fn new(width: i16, height: i16) -> Self {
        let init_position = generate_random_position(width, height);

        Food {
            position: init_position,
        }
    }

    fn refresh_position(
        &self,
        all_positions: HashSet<Position>,
        invalid_positions: HashSet<Position>,
    ) -> Option<Position> {
        let available_position: HashSet<_> = all_positions
            .difference(&invalid_positions)
            .cloned()
            .collect();
        match !available_position.is_empty() {
            true => available_position.iter().next().copied(),
            false => None,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Distribution<Direction> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0..=3) {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            _ => Direction::Right,
        }
    }
}

pub struct Snake {
    body: Vec<Position>,
    body_set: HashSet<Position>,
    direction: Direction,
}

impl Snake {
    fn new(width: i16, height: i16) -> Self {
        let init_direction: Direction = rand::random();
        let init_position = generate_random_position(width, height);
        let mut body_vec = Vec::new();
        let mut body_hashset = HashSet::new();
        body_vec.push(init_position);
        body_hashset.insert(init_position);
        Snake {
            body: body_vec,
            body_set: body_hashset,
            direction: init_direction,
        }
    }
}

pub struct App {
    gl: GlGraphics,
    snake: Snake,
    food: Food,
    game_over: bool,
    all_position: HashSet<Position>,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

        let mut snake_segments: Vec<[f64; 4]> = Vec::new();

        for i in &self.snake.body {
            let x = i.x as f64;
            let y = i.y as f64;
            snake_segments.push(rectangle::square(x, y, SQUARE_SIZE.into()));
        }
        let food = rectangle::square(
            self.food.position.x as f64,
            self.food.position.y as f64,
            SQUARE_SIZE.into(),
        );

        self.gl.draw(args.viewport(), |c, gl| {
            clear(WHITE, gl);
            let transform = c.transform.trans(0.0, 0.0).rot_deg(0.0);

            for i in snake_segments {
                rectangle(BLUE, i, transform, gl);
            }
            rectangle(GREEN, food, transform, gl);
        })
    }

    fn update(&mut self) {
        if self.game_over {
            return;
        }
        let prev_body_len = self.snake.body.len();
        if matches!(self.snake.direction, Direction::Up) {
            self.snake.body.insert(
                0,
                Position {
                    x: self.snake.body[0].x,
                    y: self.snake.body[0].y - SQUARE_SIZE,
                },
            );
        } else if matches!(self.snake.direction, Direction::Down) {
            self.snake.body.insert(
                0,
                Position {
                    x: self.snake.body[0].x,
                    y: self.snake.body[0].y + SQUARE_SIZE,
                },
            );
        } else if matches!(self.snake.direction, Direction::Left) {
            self.snake.body.insert(
                0,
                Position {
                    x: self.snake.body[0].x - SQUARE_SIZE,
                    y: self.snake.body[0].y,
                },
            );
        } else if matches!(self.snake.direction, Direction::Right) {
            self.snake.body.insert(
                0,
                Position {
                    x: self.snake.body[0].x + SQUARE_SIZE,
                    y: self.snake.body[0].y,
                },
            );
        }

        if self.is_collision() {
            self.game_over = true;
            return;
        }

        if self.snake.body.len() != prev_body_len {
            self.snake.body_set.insert(self.snake.body[0]);
        }

        if self.snake.body[0].x == self.food.position.x
            && self.snake.body[0].y == self.food.position.y
        {
            match self
                .food
                .refresh_position(self.all_position.clone(), self.snake.body_set.clone())
            {
                Some(position) => self.food.position = position,
                None => {
                    self.game_over = true;
                    return;
                }
            };
        } else {
            self.snake
                .body_set
                .remove(&self.snake.body[self.snake.body.len() - 1]);
            self.snake.body.pop();
        }
    }

    fn change_directions(&mut self, args: &ButtonArgs) {
        if args.state == ButtonState::Press {
            if args.button == Button::Keyboard(Key::Up)
                && !is_direction_conflict(&self.snake.direction, Direction::Up)
            {
                self.snake.direction = Direction::Up;
            } else if args.button == Button::Keyboard(Key::Down)
                && !is_direction_conflict(&self.snake.direction, Direction::Down)
            {
                self.snake.direction = Direction::Down;
            } else if args.button == Button::Keyboard(Key::Left)
                && !is_direction_conflict(&self.snake.direction, Direction::Left)
            {
                self.snake.direction = Direction::Left;
            } else if args.button == Button::Keyboard(Key::Right)
                && !is_direction_conflict(&self.snake.direction, Direction::Right)
            {
                self.snake.direction = Direction::Right;
            }
        }
    }

    fn is_collision(&self) -> bool {
        let width_limit = WINDOW_WIDTH_SIZE - SQUARE_SIZE;
        let height_limit = WINDOW_HEIGHT_SIZE - SQUARE_SIZE;
        if self.snake.body[0].x > width_limit
            || self.snake.body[0].x < 0
            || self.snake.body[0].x > height_limit
        {
            return true;
        }
        if self.snake.body[0].y > width_limit
            || self.snake.body[0].y < 0
            || self.snake.body[0].y > height_limit
        {
            return true;
        }
        if self.snake.body_set.contains(&self.snake.body[0]) {
            return true;
        }
        return false;
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new(
        "snake-game",
        [WINDOW_WIDTH_SIZE as u32, WINDOW_HEIGHT_SIZE as u32],
    )
    .graphics_api(opengl)
    .resizable(false)
    .exit_on_esc(true)
    .build()
    .unwrap();

    let snake = Snake::new(WINDOW_WIDTH_SIZE, WINDOW_HEIGHT_SIZE);
    let food = Food::new(WINDOW_WIDTH_SIZE, WINDOW_HEIGHT_SIZE);
    let all_position = generate_all_position(WINDOW_WIDTH_SIZE, WINDOW_HEIGHT_SIZE);
    let mut app = App {
        gl: GlGraphics::new(opengl),
        snake,
        food,
        game_over: false,
        all_position,
    };
    let mut events = Events::new(EventSettings::new()).ups(10);
    let mut already_pressed = true;
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(_args) = e.update_args() {
            already_pressed = false;
            app.update();
        }

        if app.game_over {
            println!("Game over!");
            return;
        }
        if let Some(args) = e.button_args() {
            if !(already_pressed) {
                already_pressed = true;
                app.change_directions(&args);
            }
        }
    }
}

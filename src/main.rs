use minifb::{Key, Window, WindowOptions};
use std::{time::Duration, vec};
use rand::{Rng, rngs::ThreadRng};


const SPEED: f32 = 250.0;
const TILE_SIZEX: f32 = 64.0;
const TILE_SIZEY: f32 = 64.0;
const MAP_SIZEX: f32 = 35.0;
const MAP_SIZEY: f32 = 20.0;
const WIDTH: i32 = (MAP_SIZEX * TILE_SIZEX) as i32;
const HEIGHT: i32 = (MAP_SIZEY * TILE_SIZEY) as i32;


fn darken(color: u32, factor: f32) -> u32 {
    let a = (color >> 24) & 0xFF;
    let r = (color >> 16) & 0xFF;
    let g = (color >> 8) & 0xFF;
    let b = color & 0xFF;

    let r = ((r as f32 * factor).clamp(0.0, 255.0)) as u32;
    let g = ((g as f32 * factor).clamp(0.0, 255.0)) as u32;
    let b = ((b as f32 * factor).clamp(0.0, 255.0)) as u32;

    (a << 24) | (r << 16) | (g << 8) | b
}


#[derive(PartialEq)]
#[derive(Clone)]
enum Direction {
    None = 0,
    Up = 1,
    Down = 2,
    Left = 3,
    Right = 4,
}
impl Direction {
    fn inv(&self) -> Direction {
        return match &self {
            Direction::None => Direction::None,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        };
    }
    fn get(&self) -> (f32,f32) {
        return match &self {
            Direction::None => (0.0,0.0),
            Direction::Up => (0.0,-1.0),
            Direction::Down => (0.0,1.0),
            Direction::Left => (-1.0,0.0),
            Direction::Right => (1.0,0.0),
        };
    }
}


#[derive(PartialEq)]
#[derive(Clone)]
enum Fruit {
    None = 0,
    Apple = 1,
}
impl Fruit {
    fn color(&self) -> u32 {
        return match &self {
            Fruit::None => 0xFFFFFFFF,
            Fruit::Apple => 0xFFFF0000,
        };
    }
}


struct Map {
    score: u32,
    rng: ThreadRng,
    snake: Snake,
    data: Vec<Fruit>,
}

impl Map {
    fn new(head: (f32,f32),color: u32) -> Map {
        let mut map: Map = Map {
            score: 0,
            rng: rand::rng(),
            snake: Snake::new(head,color),
            data: vec![Fruit::None;(MAP_SIZEX * MAP_SIZEY) as usize],
        };
        map.spawn(Fruit::Apple);
        return map;
    }
    fn reset(&mut self) {
        self.score = 0;
        self.snake.reset();
    }
    fn spawn(&mut self,f: Fruit) {
        let mut nx;
        let mut ny;
        
        loop {
            nx = self.rng.random_range(0..(MAP_SIZEX) as i32);
            ny = self.rng.random_range(0..(MAP_SIZEY) as i32);

            if !self.snake.collision((nx as f32,ny as f32)) {
                break;
            }  
        }
        
        let index = (ny * MAP_SIZEX as i32 + nx) as usize;
            
        if index < self.data.len() {
            self.data[index] = f;
        }
    }
    fn update(&mut self,dt: f32) {
        self.snake.update(dt);
    }
    fn eat(&mut self) {
        let pos = ((self.snake.head.0 / TILE_SIZEX) as i32,(self.snake.head.1 / TILE_SIZEY) as i32);
        let index = (pos.1 * MAP_SIZEX as i32 + pos.0) as usize;
        
        if index < self.data.len() && self.data[index] != Fruit::None {
            self.data[index] = Fruit::None;
            self.score += 1;

            if self.score < (MAP_SIZEX * MAP_SIZEY) as u32 {
                self.spawn(Fruit::Apple);
                self.snake.grow(1);
            }else {
                self.snake.dead = true;
            }
        }
    }
    fn render_part(&self,buffer: &mut [u32], f_index: usize) {
        for iy in 0..TILE_SIZEY as i32 {
            for ix in 0..TILE_SIZEX as i32 {
                let mx = f_index % MAP_SIZEX as usize;
                let my = f_index / MAP_SIZEX as usize;

                let px = mx as i32 * TILE_SIZEX as i32 + ix;
                let py = my as i32 * TILE_SIZEY as i32 + iy;
                let index: usize = (py * WIDTH + px) as usize;
                
                if index < (WIDTH * HEIGHT) as usize {
                    if self.data[f_index] == Fruit::None {
                        //let light = 0.1 + 0.8 * (ix as f32) / (TILE_SIZEX as f32) * (iy as f32) / (TILE_SIZEY as f32);
                        //let light = 0.1 + 0.9 * (mx as f32) / (TILE_SIZEX as f32) * (my as f32) / (TILE_SIZEY as f32);
                        let light = 0.1 + 0.45 * (mx as f32) / (TILE_SIZEX as f32) + 0.45 * (my as f32) / (TILE_SIZEY as f32);
                        buffer[index] = darken(self.data[f_index].color(),light);
                    }else{
                        buffer[index] = self.data[f_index].color();
                    }
                }
            }
        }
    }
    fn render(&self,buffer: &mut [u32]) {
        for i in 0..self.data.len() {
            self.render_part(buffer, i);
        }
        self.snake.render(buffer);
    }
}


struct Snake {
    head: (f32,f32),
    body: Vec<Direction>,
    color: u32,
    dead: bool,
}

impl Snake {
    fn new(head: (f32,f32),color: u32) -> Snake {
        Snake {
            head: head,
            body: vec![Direction::Right,Direction::None,Direction::None,Direction::None,Direction::None],
            color: color,
            dead: false,
        }
    }
    fn reset(&mut self) {
        self.dead = false;
        
        self.body.clear();
        self.body.push(Direction::Right);

        self.head.0 = 0.0;
        self.head.1 = 0.0;
    }
    fn collision(&self,p: (f32,f32)) -> bool {
        let mut rpos = self.head.clone();
        
        for i in 1..self.body.len() {
            let d = self.body[i].clone();
            let rnpos: (f32,f32) = d.get();
            rpos = (rpos.0 - rnpos.0,rpos.1 - rnpos.1);
            
            if (rpos.0 as i32 == p.0 as i32 && rpos.1 as i32 == p.1 as i32) && d != Direction::None {
                return true;
            }
        }
        return false;
    }
    fn update(&mut self,dt: f32) {
        if !self.dead {
            let spos: (f32,f32) = self.body.get(0).unwrap().get();
            let b_head = self.head.clone();

            self.head.0 += spos.0 * SPEED * dt;
            self.head.1 += spos.1 * SPEED * dt;

            if (b_head.0 / TILE_SIZEX) as i32 != (self.head.0 / TILE_SIZEX) as i32 || (b_head.1 / TILE_SIZEY) as i32 != (self.head.1 / TILE_SIZEY) as i32 {
                for i in (1..self.body.len()).rev() {
                    self.body[i] = self.body[i - 1].clone();
                }
            }
            if  ((self.head.0 / TILE_SIZEX) as i32) < 0 ||
                ((self.head.1 / TILE_SIZEY) as i32) < 0 ||
                ((self.head.0 / TILE_SIZEX) as i32) >= MAP_SIZEX as i32 ||
                ((self.head.1 / TILE_SIZEY) as i32) >= MAP_SIZEY as i32
            {
                self.dead = true;
            }
            if self.collision(self.head.clone()) {
                self.dead = true;
            }
        }
    }
    fn grow(&mut self,n: i32) {
        for _ in 0..n {
            self.body.push(Direction::None);
        }
    }
    fn dir(&mut self,dir: Direction) {
        if self.body.len() > 0 && self.body[0].inv() != dir {
            self.body[0] = dir;
        }
    }
    fn render_part(&self,buffer: &mut [u32], rpos: (f32,f32), color: u32) {
        for iy in 0..TILE_SIZEY as i32 {
            for ix in 0..TILE_SIZEX as i32 {
                let px = (((self.head.0 / TILE_SIZEX) as i32) + rpos.0 as i32) * TILE_SIZEX as i32 + ix;
                let py = (((self.head.1 / TILE_SIZEY) as i32) + rpos.1 as i32) * TILE_SIZEY as i32 + iy;
                let index: usize = (py * WIDTH + px) as usize;
                    
                if index < (WIDTH * HEIGHT) as usize {
                    buffer[index] = color;
                }
            }
        }
    }
    fn render(&self,buffer: &mut [u32]) {
        let mut rpos = self.body.get(0).unwrap().get();
        rpos = (rpos.0,rpos.1);

        let color = if self.dead { 0xFF880000 } else { self.color.clone() };

        for i in 0..self.body.len() {
            let d = &self.body[i];
            let rnpos: (f32,f32) = d.get();
            rpos = (rpos.0 - rnpos.0,rpos.1 - rnpos.1);
            self.render_part(buffer, rpos, if i==0 { color } else { darken(color,0.9) });
        }
    }
}


fn main() {
    let mut window = Window::new(
        "Snake",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .unwrap();

    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    let mut last_time = std::time::Instant::now();

    let mut map = Map::new((3.0,3.0),0xFF00FF00);
    
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();
        let dt = (now - last_time).as_secs_f32();
        last_time = now;

        let fps = (10.0 / dt) as i32 as f32 / 10.0;

        window.set_title(&format!("Snake | Score: {} | FPS: {}",map.score,fps));

        map.update(dt);
        map.eat();

        if window.is_key_down(Key::Enter) {
            map.reset();
        }
        
        if window.is_key_down(Key::W) {
            map.snake.dir(Direction::Up);
        }
        if window.is_key_down(Key::S) {
            map.snake.dir(Direction::Down);
        }
        if window.is_key_down(Key::A) {
            map.snake.dir(Direction::Left);
        }
        if window.is_key_down(Key::D) {
            map.snake.dir(Direction::Right);
        }

        buffer.fill(0);

        //draw_rect(&mut buffer, Left_paddle.x, Left_paddle.y as usize, PADDLE_WIDTH, PADDLE_HEIGHT, 0xFFFFFF);
        map.render(&mut buffer);

        window.update_with_buffer(&buffer,WIDTH as usize,HEIGHT as usize).unwrap();

        std::thread::sleep(Duration::from_millis(1));
    }
}
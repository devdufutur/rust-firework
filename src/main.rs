#![windows_subsystem = "windows"]

extern crate minifb;
extern crate palette;
extern crate rand;

use std::time::Instant;

use minifb::{CursorStyle, Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use palette::{LinSrgb, Shade};
use palette::rgb::Rgb;
use rand::Rng;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

#[derive(Debug)]
struct Rocket {
    x: usize,
    y: usize,
    color: u32,
    trails: Vec<u32>,
}

impl Rocket {
    fn new(x: usize, y: usize) -> Rocket {
        let color = random_rgb_color();
        let mut trails: Vec<u32> = Vec::new();
        let mut trail_color = color;
        while trail_color > 0 {
            trail_color = darken_color(trail_color, 0.01);
            trails.push(trail_color);
        }
        Rocket {
            x,
            y,
            color,
            trails,
        }
    }
}

pub fn random_y_index() -> usize {
    let mut random = rand::thread_rng();
    random.gen_range(0..HEIGHT)
}

pub fn random_rgb_color() -> u32 {
    let mut random = rand::thread_rng();
    random.gen_range(0..0xffffff)
}

pub fn darken_color(from_color: u32, darken_ratio: f64) -> u32 {
    let [_, red, green, blue] = from_color.to_be_bytes();

    let color = LinSrgb::new(red, green, blue);
    // obligé de passer en f64 pour pouvoir faire le darken
    let color = color.into_format::<f64>();
    let color = color.darken(darken_ratio);
    let Rgb { red, green, blue, standard: _ } = color.into_format::<u8>();
    u32::from_be_bytes([0, red, green, blue])
}

pub fn show_cross(grid: &mut Vec<Vec<u32>>, size: (usize, usize), mouse_coord: (usize, usize), color: u32) {
    let (width, height) = size;
    let (mouse_x, mouse_y) = mouse_coord;
    for i in 0..width {
        grid[mouse_y][i] = color;
    }

    for j in 0..height {
        grid[j][mouse_x] = color;
    }
}

fn draw_cycle(grid: &mut Vec<Vec<u32>>, rockets: &mut Vec<Rocket>, window: &mut Window, size: (usize, usize)) {
    let (width, height) = size;
    *grid = vec![vec![0; width]; height];

    if window.is_key_pressed(Key::Space, KeyRepeat::No) {
        let rocket = Rocket::new(0, random_y_index());
        rockets.push(rocket);
        // grid[random_y_index()][0] = random_rgb_color();
    }

    let opt_mouse_pos = window.get_mouse_pos(MouseMode::Discard)
        .map(|(x, y)| (x.floor() as usize, y.floor() as usize))
        .filter(|(x, y)| *x < width && *y < height);

    if window.get_mouse_down(MouseButton::Left) {
        if let Some((mouse_x, mouse_y)) = opt_mouse_pos {
            let rocket = Rocket::new(mouse_x, mouse_y);
            rockets.push(rocket);
            // grid[mouse_y][mouse_x] = random_rgb_color();
        }
    }

    for rocket in rockets.iter() {
        let Rocket { x, y, color, trails } = rocket;
        grid[*y][*x] = *color;

        let mut cur_x: isize = *x as isize - 1;

        for trail in trails.iter() {
            if cur_x >= 0 {
                grid[*y][cur_x as usize] = *trail;
            }
            cur_x -= 1;
        }
    }

    if let Some((mouse_x, mouse_y)) = opt_mouse_pos {
        let cross_color = if window.get_mouse_down(MouseButton::Left) {
            0xffffff
        } else if window.get_mouse_down(MouseButton::Middle) {
            0xff0000
        } else if window.get_mouse_down(MouseButton::Right) {
            0x0000ff
        } else {
            0xa0a0a0
        };
        show_cross(grid, (width, height), (mouse_x, mouse_y), cross_color);
    }

    let buffer: Vec<u32> = grid
        .iter()
        .flatten()
        .copied()
        .collect();

    window
        .update_with_buffer(&buffer, width, height)
        .unwrap_or_else(|e| eprintln!("Erreur lors de la mise à jour du buffer : {}", e));

    *rockets = rockets.into_iter()
        .map(|r| Rocket { x: r.x + 2, y: r.y, color: r.color, trails: r.trails.clone() })
        .filter(|f| f.x < width)
        .collect();
}

fn play(size: (usize, usize)) {
    let (width, height) = size;
    let mut grid: Vec<Vec<u32>> = vec![vec![0; width]; height];
    let mut rockets: Vec<Rocket> = Vec::new();

    let options = WindowOptions::default();
    // options.resize = true;
    // options.scale = Scale::FitScreen;
    // options.scale_mode = ScaleMode::Stretch;

    let mut window = Window::new(
        "Rust Firework - Mitrailler ESPACE ou cliquer pour lancer des fusées - ECHAP pour quitter",
        width,
        height,
        options,
    ).unwrap_or_else(|e| {
        panic!("Impossible d'ouvrir une fenêtre. Erreur : {}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_millis(16))); // ~60fps max
    window.set_cursor_style(CursorStyle::Crosshair);

    loop {
        let t1 = Instant::now();

        if !window.is_open() || window.is_key_down(Key::Escape) {
            return;
        }

        draw_cycle(&mut grid, &mut rockets, &mut window, size);

        let t2 = Instant::now();
        println!("cycle duration {:?}", t2.duration_since(t1));
    }
}

fn main() {
    play((WIDTH, HEIGHT));
}

#[cfg(test)]
mod tests {
    use crate::{darken_color, show_cross};

    #[test]
    fn darken_color_ok() {
        assert_eq!(darken_color(0xbc24b1, 0.1), 0xa30a98);
    }

    #[test]
    fn show_cross_ok() {
        let mut grid: Vec<Vec<u32>> = vec![vec![0_u32; 10]; 10];
        show_cross(&mut grid, (10, 10), (2, 3), 0x123456);

        // x axis
        assert_eq!(grid[6][2], 0x123456);

        // y axis
        assert_eq!(grid[3][8], 0x123456);

        // anywhere else
        assert_eq!(grid[0][0], 0);
    }
}
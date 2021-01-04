extern crate minifb;
extern crate rand;
extern crate palette;

use minifb::{Key, Window, WindowOptions, KeyRepeat, Scale, ScaleMode, MouseButton, MouseMode, CursorStyle};
use rand::Rng;
use palette::{Shade, LinSrgb};
use palette::rgb::Rgb;

const WIDTH: usize = 300;
const HEIGHT: usize = 200;

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

fn main() {
    let mut grid: Vec<Vec<u32>> = vec![vec![0; WIDTH]; HEIGHT];

    let mut options = WindowOptions::default();
    options.resize = true;
    options.scale = Scale::FitScreen;
    options.scale_mode = ScaleMode::Stretch;

    let mut window = Window::new(
        "Rust Firework - Mitrailler ESPACE ou cliquer pour lancer des fusées - ECHAP pour quitter",
        WIDTH,
        HEIGHT,
        options,
    )
        .unwrap_or_else(|e| {
            panic!("Impossible d'ouvrir une fenêtre. Erreur : {}", e);
        });

    window.limit_update_rate(Some(std::time::Duration::from_millis(10)));
    window.set_cursor_style(CursorStyle::Crosshair); // on rigole pas !

    loop {
        if !window.is_open() || window.is_key_down(Key::Escape) {
            break;
        }

        if window.is_key_pressed(Key::Space, KeyRepeat::No) {
            grid[random_y_index()][0] = random_rgb_color();
        }

        if window.get_mouse_down(MouseButton::Left) {
            let opt_mouse_pos = window.get_mouse_pos(MouseMode::Discard)
                .map(|(x, y)| (x.floor() as usize, y.floor() as usize))
                .filter(|(x, y)| *x < WIDTH && *y < HEIGHT);

            if let Some((mouse_x, mouse_y)) = opt_mouse_pos {
                grid[mouse_y][mouse_x] = random_rgb_color();
            }
        }

        grid = grid.iter().map(|line| line.into_iter()
            .enumerate()
            .map(|(idx, item)| {
                if *item != 0 {
                    darken_color(*item, 0.02)
                } else if idx > 0 && line[idx - 1] != 0 {
                    line[idx - 1]
                } else {
                    0
                }
            })
            .collect()
        ).collect();

        let buffer: Vec<u32> = grid
            .iter()
            .flatten()
            .copied()
            .collect();

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap_or_else(|e| eprintln!("Erreur lors de la mise à jour du buffer : {}", e));
    }
}

#[cfg(test)]
mod tests {
    use crate::darken_color;

    #[test]
    fn darken_color_ok() {
        assert_eq!(darken_color(0xbc24b1, 0.1), 0xa30a98);
    }
}
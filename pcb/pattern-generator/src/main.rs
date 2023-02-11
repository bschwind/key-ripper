use std::ops::{Add, Mul};

const SQUARE_SIZE: f64 = 2.5;

fn main() {
    let board_width = 285.0;
    let board_height = 130.0;

    let diagonal = (SQUARE_SIZE * SQUARE_SIZE + SQUARE_SIZE * SQUARE_SIZE).sqrt();
    let half_diagonal = diagonal / 2.0;

    let mut x = 0.0;

    while x < board_width {
        let mut y = 0.0;

        let t = x / (board_width);

        let mut counter = 0;
        while y < board_height {
            let x = if counter % 2 == 0 { x } else { x + half_diagonal };
            draw_square(x, y, t);

            y += half_diagonal;
            counter += 1;
        }

        x += diagonal;
    }
}

fn draw_square(x: f64, y: f64, t: f64) {
    let diagonal = (SQUARE_SIZE * SQUARE_SIZE + SQUARE_SIZE * SQUARE_SIZE).sqrt();
    let half_diagonal = diagonal / 2.0;

    let t = map_triangle(t);

    let y_offset = lerp(y + diagonal - 0.1, y + half_diagonal + 0.1, t);
    let points = vec![
        (x - half_diagonal, y + half_diagonal),
        (x, y + diagonal),
        (x + half_diagonal, y + half_diagonal),
        (x, y_offset),
        (x - half_diagonal, y + half_diagonal),
    ];

    println!("(polygon");
    println!("  (pts");

    for (x, y) in points {
        println!("    (xy {x} {y})");
    }

    println!("  )");
    println!(")");
}

// Maps [0.0, 0.5) to [0.0, 1.0],
// and [0.5, 1.0) to [1.0, 0.0]
fn map_triangle(t: f64) -> f64 {
    1.0 - 2.0 * (t - 0.5).abs()
}

fn lerp<T>(a: T, b: T, t: f64) -> T
where
    T: Add<Output = T> + Mul<f64, Output = T>,
{
    (a * (1.0 - t)) + (b * t)
}

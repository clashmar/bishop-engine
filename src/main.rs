use macroquad::prelude::*;

const TILE_SIZE: f32 = 100.0;

#[macroquad::main("Tilemap Demo")]
async fn main() {
    let map = vec![
        vec![0, 1, 1, 2],
        vec![0, 0, 1, 0],
        vec![1, 1, 1, 0],
    ];

    loop {
        clear_background(BLACK);

        for(y, row) in map.iter().enumerate() {
            for(x, &tile) in row.iter().enumerate() {
                let color = match tile {
                    0 => GRAY,
                    1 => DARKGRAY,
                    _ => RED,
                };

                draw_rectangle(
                    x as f32 * TILE_SIZE,
                    y as f32 * TILE_SIZE,
                    TILE_SIZE,
                    TILE_SIZE,
                    color,
                );
            }
        }

        next_frame().await;
    }
}

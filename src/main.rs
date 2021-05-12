
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{TextureCreator, Canvas, Texture};
use sdl2::image::LoadTexture;
use sdl2::rect::{Rect, Point};
use sdl2::video::Window;
use sdl2::gfx::primitives::DrawRenderer;
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use signal_hook::flag;

mod wfc;
mod tilesets;
use wfc::*;
use tilesets::*;

const SHOW_CONNECTIONS: bool = false;
const SHOW_TILESET: bool = false;
const AUTO_TRY: bool = false;
const STOP_ON_SUCCESS: bool = true;
const STARTING_SEED: u64 = 204;

const TILESIZE: u32 = 10;
const SCALE: u32 = 20;
const TILESIZE_SCALED: u32 = TILESIZE * SCALE;
//const MAP_SIZE: (usize, usize, usize) = ((800/TILESIZE/SCALE) as usize, (600/TILESIZE/SCALE) as usize, 1);
//const MAP_SIZE: (usize, usize, usize) = (10, 10, 7);
const MAP_SIZE: (usize, usize, usize) = (1, 1, 1);

const CON_TYPE_COLORS: [Color;3] = [
    Color::RED,
    Color::CYAN,
    Color::YELLOW,
];

#[allow(unused_must_use)]
fn draw_tile(canvas: &mut Canvas<Window>, tilemap: &Texture, col: u32, row: u32, x: u32, y:u32, angle: f64) {
    canvas.copy_ex(
        &tilemap,
        Rect::new((col*TILESIZE) as i32, (row*TILESIZE) as i32, TILESIZE, TILESIZE),
        Rect::new((x*TILESIZE_SCALED) as i32, (y*TILESIZE_SCALED) as i32, TILESIZE_SCALED, TILESIZE_SCALED),
        angle,
        Point::new((TILESIZE_SCALED/2) as i32, (TILESIZE_SCALED/2) as i32),
        false,
        false);
}


fn get_col_row(tile: &WfcTile, size: u32) -> (u32, u32) {
    let col = tile.index % size;
    let row = tile.index / size;
    return (col, row);
}


fn draw_outline_circle(canvas: &mut Canvas<Window>, x: u32, y: u32, r: i16, color: Color) {
    canvas.filled_circle(x as i16, y as i16, r, Color::BLACK);
    canvas.filled_circle(x as i16, y as i16, r-4, color);
}


fn draw_wfc_tile(canvas: &mut Canvas<Window>, tilemap: &Texture, wfc_tile: &WfcTile, size: u32, x: u32, y: u32) {
    let (col, row) = get_col_row(wfc_tile, size);
    draw_tile(canvas, &tilemap, col, row, x, y, wfc_tile.angle as f64);

    if !SHOW_CONNECTIONS {
        return;
    }

    let con_types = wfc_tile.connection_types;
    let x = x*TILESIZE_SCALED;
    let y = y*TILESIZE_SCALED;
    let half = TILESIZE_SCALED/2;
    draw_outline_circle(canvas, x+half, y, 10, CON_TYPE_COLORS[con_types[0]]);
    draw_outline_circle(canvas, x, y+half, 10, CON_TYPE_COLORS[con_types[1]]);
    draw_outline_circle(canvas, x+half, y + TILESIZE_SCALED, 10, CON_TYPE_COLORS[con_types[2]]);
    draw_outline_circle(canvas, x + TILESIZE_SCALED, y+half, 10, CON_TYPE_COLORS[con_types[3]]);
}

fn draw_text_in_rect<A>(canvas: &mut Canvas<Window>, font: &sdl2::ttf::Font, texture_creator: &TextureCreator<A>, rect: Rect, text: &str, color: Color) {
    let font_surface = font
        .render(text)
        .solid(color)
        .unwrap();
    let font_texture = font_surface.as_texture(&texture_creator).unwrap();

    let coef: f64 = std::cmp::max(rect.width(), rect.height()) as f64 / std::cmp::max(font_surface.width(), font_surface.height()) as f64;
    let mut r = Rect::new(rect.x, rect.y, (font_surface.width() as f64 * coef) as u32, (font_surface.height() as f64 * coef) as u32);

    // center inside `rect`
    r.x += ((rect.width() - r.width()) / 2) as i32;
    r.y += ((rect.height() - r.height()) / 2) as i32;

    canvas.copy(&font_texture, None, r);
}

fn draw_stack_of_tiles<A>(canvas: &mut Canvas<Window>, tilemap: &Texture, font: &sdl2::ttf::Font, texture_creator: &TextureCreator<A>, stack: &Vec<WfcTile>, tilemap_size: u32, x: i32, y: i32) {
    let stack_size: i32 = 6;
    let stack_len: usize = (stack_size*stack_size - 1) as usize;
    for (i, tile) in (stack).iter().enumerate() {
        let half_tilesize = TILESIZE_SCALED / stack_size as u32;
        let xx = i as i32 % stack_size;
        let yy = i as i32 / stack_size;

        if i>=stack_len {
            draw_text_in_rect(
                canvas,
                &font,
                &texture_creator,
                Rect::new(x + xx*half_tilesize as i32, y + yy*half_tilesize as i32, half_tilesize, half_tilesize),
                &((stack.len() - stack_len).to_string()),
                Color::WHITE);
            break;
        }

        let (col, row) = get_col_row(tile, tilemap_size);
        canvas.copy_ex(
            tilemap,
            Rect::new((col*TILESIZE) as i32, (row*TILESIZE) as i32, TILESIZE, TILESIZE),
            Rect::new(x + xx*half_tilesize as i32, y + yy*half_tilesize as i32, half_tilesize, half_tilesize),
            tile.angle as f64,
            Point::new((half_tilesize/2) as i32, (half_tilesize/2) as i32),
            false,
            false);
    }
}

fn test_path(worldmap: Worldmap, seed: u64, size: (usize, usize)) -> (WFC, Vec<WfcTile>, String, u32) {
    let (tilemap_path, tiles, tilemap_size) = flat_city_paths_only();
    let mut ttiles = tiles.clone();
    ttiles.pop();
    let mut wfc = WFC::init(worldmap, ttiles, seed);

    wfc.add_tile([2,2,0], *tiles[3].clone().rotate(2)).unwrap();
    wfc.add_tile([9,9,0], tiles[3]).unwrap();

    let (x_size, y_size) = size;
    for x in 0..x_size {
        for y in 0..y_size {
            if x == 0 || y == 0 || x == x_size-1 || y == y_size-1 {
            wfc.worldmap[[x,y,0]].clear();
            wfc.worldmap[[x,y,0]].push(tiles[0]);
            wfc.propagate([x,y,0]);
            }
        }
    }
    return (wfc, tiles, tilemap_path, tilemap_size)
}

pub fn main() {
    better_panic::install();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(155, 155, 155));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator: TextureCreator<sdl2::video::WindowContext> = canvas.texture_creator();

    // load font
    let font = ttf_context.load_font("/home/terra/.local/share/fonts/Ubuntu-B.ttf", 128).unwrap();

    let mut seed = STARTING_SEED;
    let (x_size, y_size, z_size) = MAP_SIZE;
    let worldmap = Worldmap::new3d(x_size, y_size, z_size);
    println!("worldmap: {} {:?}", worldmap.len, worldmap.size);

    //let (mut wfc, tiles, tilemap_path, tilemap_size) = test_path(worldmap, seed, (x_size, y_size));
    let (tilemap_path, mut tiles, tilemap_size) = pipes();
    //let (tilemap_path, tiles, tilemap_size) = pipes();
    //let (tilemap_path, tiles, tilemap_size) = flat_city();
    //let (tilemap_path, tiles) = stairs_3d();

    // TODO:
    // - show window (aka see results)
    //   + pipes with cross-tile
    //   - pipes with L-tile
    //   - pipes with both big-tiles
    // - add Z direction
    // - 3d (replace stairs hack with bigtile)
    // - extract function

//    let bbig_tile1 = vec![
//        Some((4*2+0, [0,2,0,0,0,0])), Some((4*2+1, [0,0,0,1,0,0])),
//        Some((4*3+0, [0,1,0,0,0,0])), Some((4*3+1, [0,0,0,2,0,0])),
//    ];
//    let bbig_tile2 = vec![
//        Some((4*2+2, [0,0,9,9,0,0])), Some((4*2+3, [2,9,2,1,0,0])),
//        Some((4*3+2, [9,0,1,0,0,0])), None,
//    ];
//    let size = (2,2,1);
//    let big_tile_parts = create_big_tile(size, bbig_tile2);
    let big_tile_parts = create_big_tile((2,2,1), vec![
        Some((4*2+2, [0,0,9,9,0,0])), Some((4*2+3, [2,9,2,1,0,0])),
        Some((4*3+2, [9,0,1,0,0,0])), None,
    ]);
    for t in &big_tile_parts {
        println!("{:?}", t);
    }

//    for chunk in rv.chunks(7) {
//        for v in chunk {
//            print!("{:^4} ", v);
//        }
//        println!("");
//    }

    // TODO:
    // - guess how many times tile has to be rotated based on the connection_types
    //   - same for big-tiles
    // - something more elegant than multiplying connections by 1000, for example:
    //   have connection_types store tuple of (u16,u16), so user has u16 for his connections, and
    //   then I have u16 for internal connections.

    // add rotations
    for _tile in &big_tile_parts {
        //for rot in 1..=1 { // X-tile only needs these rotations
        for rot in 0..4 {
            let mut tile = _tile.clone();
            tile.rotate(rot as u32);
            for i in 0..6 {
                if tile.connection_types[i] >= 1000 {
                    tile.connection_types[i] *= rot+1;
                }
            }
            println!("{:?}", tile);
            tiles.push(tile)
        }
    }
//    tiles.push(WfcTile {
//        index: 4*2 + 0,
//        connection_types: [0,2,13,10,0,0],
//        angle: 1,
//    });
//    tiles.push(WfcTile {
//        index: 4*2 + 1,
//        connection_types: [0,10,11,1,0,0],
//        angle: 1,
//    });
//    tiles.push(WfcTile {
//        index: 4*3 + 0,
//        connection_types: [13,1,0,12,0,0],
//        angle: 1,
//    });
//    tiles.push(WfcTile {
//        index: 4*3 + 1,
//        connection_types: [11,12,0,2,0,0],
//        angle: 1,
//    });
    let mut wfc = WFC::init(worldmap, tiles.clone(), seed);

    let initial_worldmap = wfc.worldmap.clone();

    if AUTO_TRY {
        let interupt_flag = Arc::new(AtomicBool::new(false));
        flag::register(signal_hook::consts::SIGINT, Arc::clone(&interupt_flag)).unwrap();

        // this is important, because worldmap might have been initialised by add_tile()
        let worldmap = wfc.worldmap.clone();
        loop {
            if interupt_flag.load(Ordering::Relaxed) {
                println!("\nReceived interrupt; Quiting...");
                return;
            }

            ::std::thread::sleep(Duration::from_millis(1));
            println!("Trying seed: {})", wfc.seed);

            let result = match wfc.run() {
                Ok(_) => true,
                Err(_) => false,
            };

            if STOP_ON_SUCCESS == result {
                break;
            }
            seed += 1;
            wfc.init_rng(seed);
            wfc.worldmap = worldmap.clone();
        }
        wfc.print_worldmap();
        return;
    }

    let tilemap = texture_creator.load_texture(tilemap_path).unwrap();
    let mut error_lock = false;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::F), .. } => {
                    if error_lock {
                        println!("Locked in error state. Press [R] to restart current seed or [N] to try new seed.");
                        continue;
                    }

                    match wfc.wfc_step() {
                        Err(e) => {
                              println!("{}", e);
                              error_lock = true;
                          },
                          _ => (),
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::N), .. } => {
                    wfc.worldmap = initial_worldmap.clone();
                    seed += 1;
                    wfc.init_rng(seed);
                    println!("-- seed {} --", seed);
                },
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    wfc.worldmap = initial_worldmap.clone();
                    wfc.init_rng(seed);
                    error_lock = false;
                    println!("-- seed {} --", seed);
                },
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    if error_lock {
                        println!("Locked in error state. Press [R] to restart current seed or [N] to try new seed.");
                    }
                    match wfc.run() {
                        Err(e) => {
                            println!("{}", e);
                            error_lock = true;
                        },
                        _ => (),
                    };
                },
                _ => {}
            }
        }

        // The rest of the game loop goes here...
        canvas.set_draw_color(Color::RGB(135, 135, 135));
        canvas.clear();

        if SHOW_TILESET {
            for (i, tile) in tiles.iter().enumerate() {
                draw_wfc_tile(&mut canvas, &tilemap, tile, tilemap_size, (i * 2) as u32, 4 as u32);
            }
        } else {
            // draw world map
            for x in 0..x_size {
                for y in 0..y_size {
                    let stack = &wfc.worldmap[(x,y)];
                    if stack.len() == 1 {
                        draw_wfc_tile(&mut canvas, &tilemap, &(stack[0]), tilemap_size, x as u32, y as u32);
                    } else {
                        draw_stack_of_tiles(
                            &mut canvas,
                            &tilemap,
                            &font,
                            &texture_creator,
                            &stack,
                            tilemap_size,
                            x as i32 * TILESIZE_SCALED as i32,
                            y as i32 * TILESIZE_SCALED as i32);
                    }
                }
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

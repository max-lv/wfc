
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

const TILESIZE: u32 = 8;
const SCALE: u32 = 8;
const TILESIZE_SCALED: u32 = TILESIZE * SCALE;
const WIN_WIDTH: u32 = 1290;
const WIN_HEIGHT: u32 = 720;
//const MAP_SIZE: (usize, usize, usize) = ((WIN_WIDTH/TILESIZE/SCALE) as usize, (WIN_HEIGHT/TILESIZE/SCALE) as usize, 1);
//const MAP_SIZE: (usize, usize, usize) = (20, 20, 20);
const MAP_SIZE: (usize, usize, usize) = (10, 10, 1);

const CON_TYPE_COLORS: [Color;4] = [
    Color::RED,
    Color::CYAN,
    Color::YELLOW,
    Color::BLUE,
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
    let fifth = TILESIZE_SCALED/7;
    if con_types[0] < CON_TYPE_COLORS.len()
        { draw_outline_circle(canvas, x + half,                    y + fifth,                   10, CON_TYPE_COLORS[con_types[0] % CON_TYPE_COLORS.len()]); }
    if con_types[1] < CON_TYPE_COLORS.len()
        { draw_outline_circle(canvas, x + TILESIZE_SCALED - fifth, y + half,                    10, CON_TYPE_COLORS[con_types[1] % CON_TYPE_COLORS.len()]); }
    if con_types[2] < CON_TYPE_COLORS.len()
        { draw_outline_circle(canvas, x + half,                    y + TILESIZE_SCALED - fifth, 10, CON_TYPE_COLORS[con_types[2] % CON_TYPE_COLORS.len()]); }
    if con_types[3] < CON_TYPE_COLORS.len()
        { draw_outline_circle(canvas, x + fifth,                   y + half,                    10, CON_TYPE_COLORS[con_types[3] % CON_TYPE_COLORS.len()]); }
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
    wfc.add_tile([8,8,0], tiles[3]).unwrap();

    let (x_size, y_size) = size;
    for x in 0..x_size {
        for y in 0..y_size {
            if x == 0 || y == 0 || x == x_size-1 || y == y_size-1 {
                wfc.add_tile([x,y,0], tiles[0]).unwrap();
            }
        }
    }

    return (wfc, tiles, tilemap_path, tilemap_size)
}

pub fn main() {
    better_panic::install();

    let mut seed = STARTING_SEED;
    let (x_size, y_size, z_size) = MAP_SIZE;
    let worldmap = Worldmap::new3d(x_size, y_size, z_size);
    println!("worldmap: {} {:?}", worldmap.len, worldmap.size);

    let (mut wfc, _, tilemap_path, tilemap_size) = test_path(worldmap, seed, (x_size, y_size));
//    let (tilemap_path, mut tiles, tilemap_size) = pipes();
    let (tilemap_path, tiles, tilemap_size) = flat_city();
    //let (tilemap_path, mut tiles, tilemap_size) = stairs_3d();
//    let (tilemap_path, mut tiles, tilemap_size, deadend) = stairs_3d_path();


//    let mut wfc = WFC::init(worldmap, tiles.clone(), seed);

//    // surround worldmap with empty tiles
//    let empty_tile = tiles[0];
//    wfc.surround_worldmap(&empty_tile);
//    // add starting points
//    wfc.add_tile([3,3,1], deadend).unwrap();
//    wfc.add_tile([3,3,15], deadend).unwrap();

    let initial_worldmap = wfc.worldmap.clone();
    let mut stage = 0;


/// Removes tiles from `square` stack which do not allow path from `dir` to `other_dir`
fn preserve_path(wfc: &mut WFC, roads: &Vec<usize>, square: &[usize; 3], dir: Direction, other_dir: Direction) {
    let dir: usize = dir.into();
    let other_dir: usize = other_dir.into();

    wfc.worldmap[*square].retain(|&x|
        roads.contains(&x.connection_types[dir])
        && roads.contains(&x.connection_types[other_dir]));
    wfc.propagate(*square);
}

/// like preserve_path, but also preserve walls between immidieate neighbours.
fn preserve_connections(wfc: &mut WFC, path_squares: &Vec<[usize; 3]>, roads: &Vec<usize>, walls: &Vec<usize>, square: &[usize; 3], dir: Direction, other_dir: Direction) {
    let dir: usize = dir.into();
    let other_dir: usize = other_dir.into();

    let keep_connections = &mut [false, false, false, false, false, false];
    for i in 0..6 {
        if let Some(sq) = wfc.worldmap.move_(*square, &Direction::from(i)) {
            keep_connections[i] = path_squares.contains(&sq);
        }
    }

    wfc.worldmap[*square].retain(|&tile| {
        // keep only tiles which:
        // - `dir` & `other_dir` directions have `roads` connections
        //   AND in keep_connections (excl. dir and other_dir) have
        //   `walls` connections.
        for i in 0..6 {
            // road directions
            if i == dir || i == other_dir {
                if !roads.contains(&tile.connection_types[i]) {
                    return false;
                }
            // walls direction
            } else if keep_connections[i] {
                if !walls.contains(&tile.connection_types[i]) {
                    return false;
                }
            } else {
                // any connections are ok
            }
        }
        return true;
    });
    wfc.propagate(*square);
}

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

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", WIN_WIDTH, WIN_HEIGHT)
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

    let tilemap = texture_creator.load_texture(tilemap_path).unwrap();
    let mut error_lock = false;
    let mut main_path = Vec::<([usize;3], Direction, Direction)>::new();
    let mut main_path_squares = Vec::<[usize;3]>::new();

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
                    stage = 0;
                    println!("-- seed {} --", seed);
                },
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    wfc.worldmap = initial_worldmap.clone();
                    stage = 0;
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
                Event::KeyDown { keycode: Some(Keycode::O), .. } => {
                    match stage {
                        0 => {
                            println!("stage 0 | generate path");
                            wfc.run_until_success();
                        },
                        1 => {
                            println!("stage 1 | find path");
                            // replace all tiles not part of main path with empty
                            let deadend_square = [2,2,0];
                            let follow_conn = 0;
                            let mut square = deadend_square;
                            main_path.clear();
                            main_path_squares.clear();
                            main_path_squares.push(square);
                            let mut last_dir = 99;
                            'outer: loop {
                                //println!("{:?}", wfc.worldmap[square][0].connection_types);
                                for i in 0..4 {
                                    let val = &wfc.worldmap[square][0].connection_types[i];
                                    if val == &follow_conn && last_dir != i {
                                        let dir = Direction::from(i);
                                        //println!("follow {} {:?} {:?}", i, dir, square);
                                        if last_dir != 99 {
                                            main_path.push((square, dir.clone(), Direction::from(last_dir)));
                                        }
                                        main_path_squares.push(square);
                                        square = wfc.worldmap.move_(square, &dir).unwrap();
                                        last_dir = dir.flip().into();
                                        //println!("new square {:?}", square);
                                        continue 'outer;
                                    }
                                }
                                break;
                            }

                            // visually show path
                            for square in &wfc.squares_list {
                                if main_path_squares.contains(square) {
                                    continue;
                                }
                                wfc.worldmap[*square].clear();
                            }
                        },
                        2 => {
                            println!("stage 2 | recreate path as connections");
                            wfc = WFC::init(Worldmap::new3d(x_size, y_size, z_size), tiles.clone(), seed);
                            //wfc.worldmap = initial_worldmap.clone();
                            // surround
                            // TODO: move surround functions into worldmap
                            wfc.surround_worldmap_2d(&tiles[0]).unwrap();

                            // place deadends

                            let roads = vec![0];
                            let walls = vec![1];
                            for (square, dir, other_dir) in &main_path {
                                let _dir: usize = dir.clone().into();
                                let _other_dir: usize = other_dir.clone().into();

                                //preserve_path(&mut wfc, &roads, square, dir.clone(), other_dir.clone());
                                preserve_connections(&mut wfc, &main_path_squares, &roads, &walls, &square, dir.clone(), other_dir.clone());
                            }
                        },
                        3 => {
                            println!("stage 3 | ");
                            wfc.run_until_success();
                        },
                        _ => {
                            println!("stage {} unknown", stage);
                        }
                    };
                    stage += 1;
                },
                _ => {}
            }
        }

        // The rest of the game loop goes here...
        canvas.set_draw_color(Color::RGB(135, 135, 135));
        canvas.clear();

        if SHOW_TILESET && wfc.worldmap[0].len() > 1 {
            for (i, tile) in wfc.worldmap[0].iter().enumerate() {
                let x = i % x_size;
                let y = i / x_size;
                draw_wfc_tile(&mut canvas, &tilemap, tile, tilemap_size, x as u32, y as u32);
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

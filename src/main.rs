
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
use rand::Rng;
use rand::SeedableRng;
use signal_hook::flag;

mod wfc;
mod tilesets;
use wfc::*;
use tilesets::*;

const SHOW_CONNECTIONS: bool = false;
const SHOW_TILESET: bool = false;
const AUTO_TRY: bool = true;
const STOP_ON_SUCCESS: bool = true;
const STARTING_SEED: u64 = 144;

const TILESIZE: u32 = 10;
const SCALE: u32 = 5;
const TILESIZE_SCALED: u32 = TILESIZE * SCALE;
const MAP_WIDTH: usize = 10;//(800/TILESIZE/SCALE) as usize;
const MAP_HEIGHT: usize = 10;//(600/TILESIZE/SCALE) as usize;

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


fn draw_outline_circle(canvas: &mut Canvas<Window>, x: u32, y: u32, r: i16, color: Color) {
    canvas.filled_circle(x as i16, y as i16, r, Color::BLACK);
    canvas.filled_circle(x as i16, y as i16, r-4, color);
}


fn draw_wfc_tile(canvas: &mut Canvas<Window>, tilemap: &Texture, wfc_tile: &WFC_Tile, x: u32, y: u32) {
    draw_tile(canvas, &tilemap, wfc_tile.col, wfc_tile.row, x, y, wfc_tile.angle as f64);

    if !SHOW_CONNECTIONS {
        return;
    }

    //let con_types = rotate_array(wfc_tile.connection_types, (wfc_tile.angle/90) as usize);
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

fn draw_stack_of_tiles<A>(canvas: &mut Canvas<Window>, tilemap: &Texture, font: &sdl2::ttf::Font, texture_creator: &TextureCreator<A>, stack: &Vec<WFC_Tile>, x: i32, y: i32) {
    for (i, tile) in (stack).iter().enumerate() {
        let half_tilesize = TILESIZE_SCALED/2;
        let xx = i as i32 %2;
        let yy = i as i32 /2;

        if i>=3 {
            draw_text_in_rect(
                canvas,
                &font,
                &texture_creator,
                Rect::new(x + xx*half_tilesize as i32, y + yy*half_tilesize as i32, half_tilesize, half_tilesize),
                &((stack.len()-3).to_string()),
                Color::WHITE);
            break;
        }

        canvas.copy_ex(
            tilemap,
            Rect::new((tile.col*TILESIZE) as i32, (tile.row*TILESIZE) as i32, TILESIZE, TILESIZE),
            Rect::new(x + xx*half_tilesize as i32, y + yy*half_tilesize as i32, half_tilesize, half_tilesize),
            tile.angle as f64,
            Point::new((half_tilesize/2) as i32, (half_tilesize/2) as i32),
            false,
            false);
    }
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

    //let (tilemap_path, tiles) = pipes();
    //let (tilemap_path, tiles) = flat_city();
    let (tilemap_path, tiles) = stairs_3d();
    //let tilemap = texture_creator.load_texture(tilemap_path).unwrap();
    let ttiles = tiles.clone();
    let mut seed = STARTING_SEED;
    let worldmap = Worldmap::new3d(MAP_WIDTH, MAP_HEIGHT, 7);
    println!("worldmap: {} {:?}", worldmap.len, worldmap.size);
    let mut wfc = WFC::init(worldmap, tiles);
    wfc.rng = rand::rngs::StdRng::seed_from_u64(seed);

    if AUTO_TRY {
        let interupt_flag = Arc::new(AtomicBool::new(false));
        flag::register(signal_hook::consts::SIGINT, Arc::clone(&interupt_flag));

        for try_num in 0..10_000 {
            if interupt_flag.load(Ordering::Relaxed) {
                println!("\nReceived interrupt; Quiting...");
                return;
            }

            ::std::thread::sleep(Duration::from_millis(1));
            println!("Try #{} (seed: {})", try_num, seed);
            loop {
                if wfc.debug_break {
                    break;
                }
                //println!("wfc_step");
                let tile = match wfc.wfc_step() {
                    Some(x) => x,
                    None => break,
                };
                wfc.propagate(tile);
            }

            for tile in &wfc.worldmap.values {
                if tile[0].col == 1 {
                    wfc.print_worldmap();
                    break
                }
            }

            if wfc.debug_break == STOP_ON_SUCCESS {
                wfc.debug_break = false;
                wfc.init_worldmap();
                seed += 1;
                wfc.rng = rand::rngs::StdRng::seed_from_u64(seed);
            } else {
                break;
            }
        }
    }

    return;
    let tilemap = texture_creator.load_texture(tilemap_path).unwrap();

// instead of going to each tile and changing their possible tiles list
// we can:
// 1. start with empty Map[][]
// 2. choose random point and select tile there (observer)
// 3. for all adjesent tiles create list with valid tiles
// 4. take note of these tiles
// 5. (observer) random tile from list of tiles we took note of on step 4
// 6. repeat from step 3

// Also back tracking: before observing, save current map state, if during steps 3..6
// we get to the point where map-square has no valid tile options, then restore map state
// and remove tile we choose last time from a list. Pick new tile.

// function observe() {
  // 1. find tiles with length of stack above 1
  // 2. choose random map-square from that list
  // 3. choose random tile from stack list
  // 4. propagate change in 4 directions

  // propagation
  // 1. compare choosen tile.connections and remove invalid tiles from stack
  // IDEA: tbh stacks, in addition to holding tile indexes they should hold connection types they accept on each direction
  // if after changing stack, connection types changed, then we have to propagate in
  // those directions

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::F), .. } => {
                    if wfc.debug_break {
                        println!("Locked via debug_break");
                    } else {
                        //println!("wfc_step");
                        let tile = match wfc.wfc_step() {
                            Some(x) => x,
                            None => break,
                        };
                        wfc.propagate(tile);
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::N), .. } => {
                    wfc.init_worldmap();
                    seed += 1;
                    wfc.rng = rand::rngs::StdRng::seed_from_u64(seed);
                    println!("-- seed {} --", seed);
                },
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    wfc.debug_break = false;
                    wfc.init_worldmap();
                    wfc.rng = rand::rngs::StdRng::seed_from_u64(seed);
                    println!("-- seed {} --", seed);
                },
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    loop {
                        if wfc.debug_break {
                            break;
                        }
                        //println!("wfc_step");
                        let tile = match wfc.wfc_step() {
                            Some(x) => x,
                            None => break,
                        };
                        wfc.propagate(tile);
                    }
                },
                _ => {}
            }
        }

        // The rest of the game loop goes here...
        canvas.set_draw_color(Color::RGB(135, 135, 135));
        canvas.clear();

        if SHOW_TILESET {
            for (i, tile) in ttiles.iter().enumerate() {
                draw_wfc_tile(&mut canvas, &tilemap, tile, (i * 2) as u32, 4 as u32);
            }
        } else {
            // draw world map
            for x in 0..MAP_WIDTH {
                for y in 0..MAP_HEIGHT {
                    let stack = &wfc.worldmap[(x,y)];
                    if stack.len() == 1 {
                        draw_wfc_tile(&mut canvas, &tilemap, &(stack[0]), x as u32, y as u32);
                    } else {
                        draw_stack_of_tiles(
                            &mut canvas,
                            &tilemap,
                            &font,
                            &texture_creator,
                            &stack,
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

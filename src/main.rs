
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{TextureCreator, Canvas, Texture};
use sdl2::image::LoadTexture;
use sdl2::rect::{Rect, Point};
use sdl2::video::Window;
use sdl2::gfx::primitives::DrawRenderer;
use std::time::Duration;
use std::collections::{HashMap, HashSet};
use std::ops::IndexMut;
use rand::Rng;
use rand::SeedableRng;

const SHOW_CONNECTIONS: bool = false;
const SHOW_TILESET: bool = false;
const AUTO_TRY: bool = false;
const STOP_ON_SUCCESS: bool = false;
const STARTING_SEED: u64 = 144;

const TILESIZE: u32 = 8;
const SCALE: u32 = 5;
const TILESIZE_SCALED: u32 = TILESIZE * SCALE;
const MAP_WIDTH: usize = (800/TILESIZE/SCALE) as usize;
const MAP_HEIGHT: usize = (600/TILESIZE/SCALE) as usize;

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


#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
struct WFC_Tile {
    col: u32,
    row: u32,
    connection_types: [usize; 4],
    angle: u32,
}

fn rotate_array(arr: [usize; 4], rot: usize) -> [usize; 4] {
    [arr[rot % 4], arr[(rot+1) % 4], arr[(rot+2) % 4], arr[(rot+3) % 4]]
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

struct WFC {
    rng: rand::rngs::StdRng,
    tiles: Vec<WFC_Tile>,
    worldmap: Worldmap,
    debug_break: bool,
}

/*
worldmap
- will always be container of tiles
- have to be indexed by (arbitrary?) index
  x,y for 2d (tiles, triangles, hexes)
  x,y,z for 3d
  I want to index it in order
- directions
  2d has: 3 (trinagles), 4 (tiles), 6 (hexes)
  - WFC and WFC_tile should be generic over directions. This will
    bring type safety for my tilesets a.k.a saves me headache of checking if all tiles in tileset have neccesery directions.
- I don't care about tiles or their rotation, I just need Vec<tile> and tile must have directions.
  main task I'm solving: given 'two stacks of tiles and direction' remove all non-compatible tiles.
*/

// 3d worldmap
// struct with vec-vec-of-T
// index generic ???
// iter() over all indexes

struct Worldmap {
    values: Vec<Vec<WFC_Tile>>,
    size: Vec<usize>,
    len: usize,
}

impl Worldmap {
    fn new2d(x: usize, y: usize) -> Worldmap {
        Worldmap {
            values: vec![vec![]; x*y],
            size: vec![x, y],
            len: x*y,
        }
    }
    fn new3d(x: usize, y: usize, z: usize) -> Worldmap {
        Worldmap {
            values: vec![vec![]; x*y*z],
            size: vec![x, y, z],
            len: x*y*z,
        }
    }
    fn len(&self) -> usize {
        self.len
    }
}

// !!WATCH YOUR STEP!! Rust Hell Below
// -----------------------------------
impl std::ops::Index<usize> for Worldmap {
    type Output = Vec<WFC_Tile>;
    fn index<'a>(&'a self, idx: usize) -> &'a Vec<WFC_Tile> {
        return &self.values[idx]
    }
}

impl std::ops::IndexMut<usize> for Worldmap {
    fn index_mut<'a>(&'a mut self, idx: usize) -> &'a mut Vec<WFC_Tile> {
        return &mut self.values[idx]
    }
}
impl std::ops::Index<(usize, usize)> for Worldmap {
    type Output = Vec<WFC_Tile>;
    fn index<'a>(&'a self, idx: (usize, usize)) -> &'a Vec<WFC_Tile> {
        let (x, y) = idx;
        return &self.values[x + y*self.size[0]]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Worldmap {
    fn index_mut<'a>(&'a mut self, idx: (usize, usize)) -> &'a mut Vec<WFC_Tile> {
        let (x, y) = idx;
        return &mut self.values[x + y*self.size[0]]
    }
}

impl std::ops::Index<(usize, usize, usize)> for Worldmap {
    type Output = Vec<WFC_Tile>;
    fn index<'a>(&'a self, idx: (usize, usize, usize)) -> &'a Vec<WFC_Tile> {
        let (x, y, z) = idx;
        return &self.values[x + y*self.size[1] + z*self.size[2]]
    }
}

impl std::ops::IndexMut<(usize, usize, usize)> for Worldmap {
    fn index_mut<'a>(&'a mut self, idx: (usize, usize, usize)) -> &'a mut Vec<WFC_Tile> {
        let (x, y, z) = idx;
        return &mut self.values[x + y*self.size[0] + z*self.size[0]*self.size[1]]
    }
}


// IndexDirection
// - can be generic for WFC_Tile & Worldmap
// (usize,usize) vs (usize,usize,usize)
// - triangle/hex directions for later
// - have to take care of rotation
//   - 4 for 2d, ??? for 3d
// - move_sq (in the direction)
// - directions ENUM
//   - PX, NX, PY, NY, PZ, NZ
// hmmm how to make it simple 3d (only Y-axis rotation)


impl WFC {
    // TODO: worldmap should hold indexes into tiles
    fn init(worldmap: Worldmap, tiles: Vec<WFC_Tile>) -> WFC {
        let mut wfc = WFC {
            tiles,
            worldmap,
            rng: rand::rngs::StdRng::seed_from_u64(1),
            debug_break: false,
        };
        wfc.init_worldmap();
        wfc
    }

    fn init_tile(tiles: &Vec<WFC_Tile>, square: &mut Vec<WFC_Tile>) {
        square.clear();
        for tile in tiles {
            for i in 0..4 {
                // TODO: worldmap could have stored u16 indexes into self.tiles, would be alot more
                // compact!
                let new_wfc_tile = WFC_Tile {
                    col: tile.col,
                    row: tile.row,
                    connection_types: rotate_array(tile.connection_types, i),
                    angle: (i*90) as u32,
                };
                square.push(new_wfc_tile);
            }
        }
    }

    fn init_worldmap(&mut self) {
        // fill worldmap with stuff
        for i in 0..self.worldmap.len {
            WFC::init_tile(&self.tiles, &mut self.worldmap[i]);
        }
    }

    fn print_worldmap(&self) {
        // debug print worldmap
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                print!("{:?} ", self.worldmap[(x,y)].len());
            }
            println!("");
        }
    }

    fn find_squares_in_order(&self) -> Vec::<(usize, usize)> {
        let mut available_squares = Vec::<(usize, usize)>::with_capacity(1);
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                if self.worldmap[(x,y)].len() == 1 {
                    continue;
                }
                available_squares.push((x,y));
                return available_squares;
            }
        }

        // always empty:
        return available_squares;
    }

    fn wfc_step(&mut self) -> Option<(usize, usize)> { // wfc3d: index has to be 2d/3d (or multidimensional)
        //let available_squares = self.find_undecided_squares();
        //let available_squares = self.find_adjacent_undecided_squares();
        //let available_squares = self.find_lowest_undecided_squares();
        //let available_squares = self.find_surrounded_undecided_squares();
        //let available_squares = self.find_touched_undecided_squares();
        let available_squares = self.find_squares_in_order();

        if available_squares.len() == 0 {
            println!("done");
            return None
        }
        let map_square = *choose_random(&mut self.rng, &available_squares);

        // observe
        let selected_tile = *choose_random(&mut self.rng, &self.worldmap[map_square]);
        self.worldmap[map_square].clear(); // @question: is this does free() ?
        self.worldmap[map_square].push(selected_tile);
        return Some(map_square);
    }

    // wfc3d: map_square has to be 2d/3d and directions
    /// returns true if we changed available connections, false otherwise
    fn update_tile_stack(&mut self, connections: &HashSet<usize>, map_square: (usize, usize), dir: usize) -> bool {
        // we are trying to access tile beyond edge
        let (x, y) = match WFC::move_sq(map_square, dir) {
            Some(x) => x,
            None => return false,
        };

        // this direction has all connections available
        // FIXME: 3 is hardcoded, but should be calculated.
        //        probably should put it into WFC struct.
        //        can be calculated from original tiles.
        if connections.len() == 4 {
            return false;
        }

        let mut ok_stack = Vec::<WFC_Tile>::with_capacity(5);

        let stack = &self.worldmap[(x,y)];
        for tile in stack {
            // connection_types: [N, E, S, W]
            //                    0  1  2  3
            if !connections.contains(&tile.connection_types[(dir+2) % 4]) {
                continue
            }
            ok_stack.push(*tile);
        }
        if self.worldmap[(x,y)].len() == ok_stack.len() {
            return false;
        }
        if ok_stack.len() == 0 {
            println!("error: tile-stack reduced to 0!!!  tile: {:?}", map_square);
            self.worldmap[(x,y)].clear();
            self.debug_break = true;
            return true;
        }
        self.worldmap[(x,y)] = ok_stack;
//        println!("update_tile_stack has changed connections {:?}", map_square);
        return true;
    }

    // wfc3d: this should be moved to Index/Direction type
    fn move_sq(map_square: (usize, usize), dir: usize) -> Option<(usize, usize)> {
        let (dx, dy) = match dir {
            0 => ( 0, -1), // NORTH
            1 => (-1,  0), // EAST ??? for some reason EAST and WEST switched around, now it should go counter-clockwise
            2 => ( 0,  1), // SOUTH
            3 => ( 1,  0), // WEST
            _ => panic!("Unknown dir {}", dir),
        };
        let (x, y) = map_square;
        let x = x as i32 + dx;
        let y = y as i32 + dy;
        if x < 0 || x as usize >= MAP_WIDTH { return None; }
        if y < 0 || y as usize >= MAP_HEIGHT { return None; }
        return Some((x as usize, y as usize));
    }

    // finds all available connections from this tile for each direction
    fn gather_available_connections(&self, square: (usize, usize)) -> Vec::<std::collections::HashSet<usize>> {
        let (x,y) = square;
        let mut connections = Vec::<std::collections::HashSet<usize>>::with_capacity(4);
        for _ in 0..4 {
            connections.push(std::collections::HashSet::<usize>::new());
        }
        let stack = &self.worldmap[(x,y)];
        for tile in stack {
            for i in 0..4 {
                connections[i].insert(tile.connection_types[i]);
            }
        }
        return connections;
    }

    // wfc3d: map_square probably can be anything
    fn propagate(&mut self, map_square: (usize, usize)) {
        if self.debug_break {
            return;
        }

        let connections = self.gather_available_connections(map_square);

        // clear direction
        let is_recurse0 = self.update_tile_stack(&connections[0], map_square, 0);
        let is_recurse1 = self.update_tile_stack(&connections[1], map_square, 1);
        let is_recurse2 = self.update_tile_stack(&connections[2], map_square, 2);
        let is_recurse3 = self.update_tile_stack(&connections[3], map_square, 3);

        // recurse in that direction
        if is_recurse0 { self.propagate(WFC::move_sq(map_square, 0).unwrap()); }
        if is_recurse1 { self.propagate(WFC::move_sq(map_square, 1).unwrap()); }
        if is_recurse2 { self.propagate(WFC::move_sq(map_square, 2).unwrap()); }
        if is_recurse3 { self.propagate(WFC::move_sq(map_square, 3).unwrap()); }
    }
}

fn choose_random<'a, Any>(rng: &mut rand::rngs::StdRng, vec: &'a Vec<Any>) -> &'a Any {
    &vec[rng.gen_range(0..vec.len())]
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
    let texture_creator = canvas.texture_creator();

    // load font
    let font = ttf_context.load_font("/home/terra/.local/share/fonts/Ubuntu-B.ttf", 128).unwrap();
//    let tilemap = texture_creator.load_texture("./pipes_tileset.png").unwrap();
//
//    // WFC Stuff
//    // ---------
//    // INIT worldmap
//    let mut tiles = Vec::new();
//    tiles.push(WFC_Tile {
//        col: 0,
//        row: 0,
//        connection_types: [1,1,0,1],
//        angle: 0,
//    });
//    tiles.push(WFC_Tile {
//        col: 2,
//        row: 0,
//        connection_types: [0,0,0,0],
//        angle: 0,
//    });
//    tiles.push(WFC_Tile {
//        col: 0,
//        row: 1,
//        connection_types: [0,1,0,1],
//        angle: 0,
//    });
//    tiles.push(WFC_Tile {
//        col: 1,
//        row: 0,
//        connection_types: [1,1,1,1],
//        angle: 0,
//    });
//    tiles.push(WFC_Tile {
//        col: 1,
//        row: 1,
//        connection_types: [0,0,1,1],
//        angle: 0,
//    });
//    // connecting pipe
//    tiles.push(WFC_Tile {
//        col: 2,
//        row: 1,
//        connection_types: [0,1,0,2],
//        angle: 0,
//    });
//    tiles.push(WFC_Tile {
//        col: 3,
//        row: 0,
//        connection_types: [0,0,2,2],
//        angle: 0,
//    });
//    tiles.push(WFC_Tile {
//        col: 3,
//        row: 1,
//        connection_types: [0,2,0,2],
//        angle: 0,
//    });

    let tilemap = texture_creator.load_texture("./flat-city.png").unwrap();

    // WFC Stuff
    // ---------
    // INIT worldmap
    let mut tiles = Vec::new();
    tiles.push(WFC_Tile {
        col: 0,
        row: 0,
        connection_types: [1,1,1,1],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 1,
        row: 0,
        connection_types: [1,1,1,1],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 2,
        row: 0,
        connection_types: [1,0,1,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 3,
        row: 0,
        connection_types: [1,1,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 4,
        row: 0,
        connection_types: [1,1,1,1],
        angle: 0,
    });
    // walls
    tiles.push(WFC_Tile {
        col: 0,
        row: 1,
        connection_types: [1,2,1,2],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 1,
        row: 1,
        connection_types: [1,1,2,2],
        angle: 0,
    });
    // fat blocks
    tiles.push(WFC_Tile {
        col: 2,
        row: 1,
        connection_types: [1,1,3,3],
        angle: 0,
    });
//    tiles.push(WFC_Tile {
//        col: 3,
//        row: 1,
//        connection_types: [1,1,3,3],
//        angle: 0,
//    });
    tiles.push(WFC_Tile {
        col: 4,
        row: 1,
        connection_types: [3,3,3,3],
        angle: 0,
    });

    let ttiles = tiles.clone();
    let mut seed = STARTING_SEED;
    let worldmap = Worldmap::new2d(MAP_WIDTH, MAP_HEIGHT);
    println!("worldmap: {} {:?}", worldmap.len, worldmap.size);
    let mut wfc = WFC::init(worldmap, tiles);
    wfc.rng = rand::rngs::StdRng::seed_from_u64(seed);

    if AUTO_TRY {
        for try_num in 0..10_000 {
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

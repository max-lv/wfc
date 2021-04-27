
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
use rand::Rng;
use rand::SeedableRng;

const SHOW_CONNECTIONS: bool = false;
const SHOW_TILESET: bool = false;
const AUTO_TRY: bool = true;
const STOP_ON_SUCCESS: bool = false;
const STARTING_SEED: u64 = 144;

const TILESIZE: u32 = 10;
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
    worldmap: Vec<Vec<Vec<WFC_Tile>>>,
    debug_break: bool,
}

impl WFC {
    fn init(tiles: Vec<WFC_Tile>) -> WFC {
        let mut worldmap: Vec<Vec<Vec<WFC_Tile>>>;
        worldmap = Vec::with_capacity(10);
        for x in 0..MAP_WIDTH {
            worldmap.push(Vec::with_capacity(10));
            for y in 0..MAP_HEIGHT {
                worldmap[x].push(Vec::with_capacity(5));
            }
        }

        let mut wfc = WFC {
            tiles,
            worldmap,
            rng: rand::rngs::StdRng::seed_from_u64(1),
            debug_break: false,
        };
        wfc.init_worldmap();
        wfc
    }

    fn init_tile(&mut self, square: (usize, usize)) {
        let (x,y) = square;
        self.worldmap[x][y].clear();
        for tile in &self.tiles {
            for i in 0..4 {
                // TODO: worldmap could have stored u16 indexes into self.tiles, would be alot more
                // compact!
                let new_wfc_tile = WFC_Tile {
                    col: tile.col,
                    row: tile.row,
                    connection_types: rotate_array(tile.connection_types, i),
                    angle: (i*90) as u32,
                };
                self.worldmap[x][y].push(new_wfc_tile);
            }
        }
    }

    fn init_worldmap(&mut self) {
        // fill worldmap with stuff
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                self.init_tile((x,y));
            }
        }
    }

    fn print_worldmap(&self) {
        // debug print worldmap
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                print!("{:?} ", self.worldmap[x][y].len());
            }
            println!("");
        }
    }

    fn find_undecided_squares(&self) -> Vec::<(usize, usize)> {
        // find tiles with length of stack above 1
        let mut available_squares = Vec::<(usize, usize)>::with_capacity(MAP_WIDTH*MAP_HEIGHT);
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                if self.worldmap[x][y].len() == 1 {
                    continue;
                }
                available_squares.push((x,y));
            }
        }
        return available_squares;
    }

    /// like find_undecided_squares() but also excludes squares with maximum tile-stack size
    fn find_touched_undecided_squares(&self) -> Vec::<(usize, usize)> {
        let mut available_squares = Vec::<(usize, usize)>::with_capacity(MAP_WIDTH*MAP_HEIGHT);
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                if self.worldmap[x][y].len() == 1 || self.worldmap[x][y].len() == 3 {
                    continue;
                }
                available_squares.push((x,y));
            }
        }
        return available_squares;
    }

    /// collects all undecided squares around already collapsed squares
    fn find_adjacent_undecided_squares(&mut self) -> Vec::<(usize, usize)> {
        let mut available_squares = HashSet::<(usize, usize)>::with_capacity(MAP_WIDTH*MAP_HEIGHT);
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                if self.worldmap[x][y].len() > 1 {
                    continue;
                }
                for d in 0..4 {
                    let (_x, _y) = match WFC::move_sq((x,y), d) {
                        Some(sq) => sq,
                        None => continue,
                    };
                    if self.worldmap[_x][_y].len() > 1 && self.worldmap[_x][_y].len() < 4 {
                        available_squares.insert((_x,_y));
                    }
                }
            }
        }

        if available_squares.len() == 0 {
            let x = self.rng.gen_range(0..MAP_WIDTH);
            let y = self.rng.gen_range(0..MAP_HEIGHT);
            available_squares.insert((x,y));
        }
        let mut rv = available_squares.into_iter().collect::<Vec::<(usize, usize)>>();
        rv.sort();
        return rv;
    }

    fn find_lowest_undecided_squares(&self) -> Vec::<(usize, usize)> {
        let tiles_variations = self.tiles.len()*4 + 1;
        let mut available_squares = Vec::<Vec<(usize, usize)>>::with_capacity(tiles_variations);
        for _ in 0..tiles_variations {
            available_squares.push(Vec::<(usize, usize)>::with_capacity(MAP_WIDTH*MAP_HEIGHT));
        }
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                let len = self.worldmap[x][y].len();
                if len == 1 {
                    continue;
                }
                available_squares[len].push((x,y));
            }
        }

        for x in &available_squares {
            if x.len() > 0 {
                return x.to_vec();
            }
        }
        return available_squares[0].to_vec();
    }

    fn find_surrounded_undecided_squares(&self) -> Vec::<(usize, usize)> {
        let mut available_squares = Vec::<Vec<(usize, usize)>>::with_capacity(5);
        for _ in 0..5 {
            available_squares.push(Vec::<(usize, usize)>::with_capacity(MAP_WIDTH*MAP_HEIGHT));
        }
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                if self.worldmap[x][y].len() == 1 {
                    continue;
                }
                // count decided tiles surrounding this square
                let mut count = 0;
                let square = (x,y);
                for d in 0..4 {
                    let (x,y) = match WFC::move_sq(square, d) {
                        Some(sq) => sq,
                        None => continue,
                    };
                    if self.worldmap[x][y].len() == 1 {
                        count += 1;
                    }
                }
                available_squares[count].push((x,y));
            }
        }

        for i in (0..4).rev() {
            if available_squares[i].len() > 0 {
                return available_squares[i].to_vec();
            }
        }
        // always empty:
        return available_squares[0].to_vec();
    }

    fn wfc_step(&mut self) -> Option<(usize, usize)> {
        //let available_squares = self.find_undecided_squares();
        //let available_squares = self.find_adjacent_undecided_squares();
        let available_squares = self.find_lowest_undecided_squares();
        //let available_squares = self.find_surrounded_undecided_squares();
        //let available_squares = self.find_touched_undecided_squares();
        if available_squares.len() == 0 {
            println!("done");
            return None
        }
        let map_square = *choose_random(&mut self.rng, &available_squares);

//        println!("Selected map square: {:?}", map_square);

        // observe
        let (x,y) = map_square;
        let selected_tile = *choose_random(&mut self.rng, &self.worldmap[x][y]);
        self.worldmap[x][y].clear(); // @question: is this does free() ?
        self.worldmap[x][y].push(selected_tile);
//        println!("Selected tile: {:?}", selected_tile);
        return Some(map_square);

        // TODO: propagate
    }

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
        if connections.len() == 3 {
            return false;
        }

        let mut ok_stack = Vec::<WFC_Tile>::with_capacity(5);

        let stack = &self.worldmap[x][y];
        for tile in stack {
            // connection_types: [N, E, S, W]
            //                    0  1  2  3
            if !connections.contains(&tile.connection_types[(dir+2) % 4]) {
                continue
            }
            ok_stack.push(*tile);
        }
        if self.worldmap[x][y].len() == ok_stack.len() {
            return false;
        }
        if ok_stack.len() == 0 {
            println!("error: tile-stack reduced to 0!!!  tile: {:?}", map_square);
            self.worldmap[x][y].clear();
            self.debug_break = true;
            return true;
        }
        self.worldmap[x][y] = ok_stack;
//        println!("update_tile_stack has changed connections {:?}", map_square);
        return true;
    }

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
        let stack = &self.worldmap[x][y];
        for tile in stack {
            for i in 0..4 {
                connections[i].insert(tile.connection_types[i]);
            }
        }
        return connections;
    }

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
    let tilemap = texture_creator.load_texture("./pipes_tileset.png").unwrap();

    // WFC Stuff
    // ---------
    // INIT worldmap
    let mut tiles = Vec::new();
    tiles.push(WFC_Tile {
        col: 0,
        row: 0,
        connection_types: [1,1,0,1],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 2,
        row: 0,
        connection_types: [0,0,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 0,
        row: 1,
        connection_types: [0,1,0,1],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 1,
        row: 0,
        connection_types: [1,1,1,1],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 1,
        row: 1,
        connection_types: [0,0,1,1],
        angle: 0,
    });
    // connecting pipe
    tiles.push(WFC_Tile {
        col: 2,
        row: 1,
        connection_types: [0,1,0,2],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 3,
        row: 0,
        connection_types: [0,0,2,2],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 3,
        row: 1,
        connection_types: [0,2,0,2],
        angle: 0,
    });
    let ttiles = tiles.clone();

    let mut seed = STARTING_SEED;
    let mut wfc = WFC::init(tiles);
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
                    let stack = &wfc.worldmap[x][y];
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

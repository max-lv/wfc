
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
    size: [usize; 3],
    len: usize,
}

impl Worldmap {
    fn new3d(x: usize, y: usize, z: usize) -> Worldmap {
        Worldmap {
            values: vec![vec![]; x*y*z],
            size: [x, y, z],
            len: x*y*z,
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn tmp_iter(&self) -> impl Iterator<Item = Position> {
        let mut values = Vec::<Position>::with_capacity(self.len);
        for x in 0..self.size[0] {
        for y in 0..self.size[1] {
        for z in 0..self.size[2] {
            values.push([x,y,z]);
        }
        }
        }
        return values.into_iter();
    }

    fn move_(&self, square: Position, dir: &Direction) -> Option<Position> {
        let (dx, dy, dz) = match dir {
            Direction::NORTH => ( 0, -1,  0),
            Direction::EAST  => (-1,  0,  0), // FIXME: ??? for some reason EAST and WEST switched around, 
            Direction::SOUTH => ( 0,  1,  0), //        now it should go counter-clockwise
            Direction::WEST  => ( 1,  0,  0),
            Direction::UP    => ( 0,  0,  1),
            Direction::DOWN  => ( 0,  0, -1),
        };
        let [XS,YS,ZS] = self.size;
        let [x, y, z] = square;
        let x = x as i32 + dx;
        let y = y as i32 + dy;
        let z = z as i32 + dz;
        if x < 0 || x as usize >= XS { return None; }
        if y < 0 || y as usize >= YS { return None; }
        if z < 0 || z as usize >= ZS { return None; }
        return Some([x as usize, y as usize, z as usize]);
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
        return &self.values[x + y*self.size[0] + z*self.size[0]*self.size[1]]
    }
}

impl std::ops::IndexMut<(usize, usize, usize)> for Worldmap {
    fn index_mut<'a>(&'a mut self, idx: (usize, usize, usize)) -> &'a mut Vec<WFC_Tile> {
        let (x, y, z) = idx;
        return &mut self.values[x + y*self.size[0] + z*self.size[0]*self.size[1]]
    }
}

impl std::ops::Index<Position> for Worldmap {
    type Output = Vec<WFC_Tile>;
    fn index<'a>(&'a self, idx: Position) -> &'a Vec<WFC_Tile> {
        let [x, y, z] = idx;
        return &self.values[x + y*self.size[0] + z*self.size[0]*self.size[1]]
    }
}

impl std::ops::IndexMut<Position> for Worldmap {
    fn index_mut<'a>(&'a mut self, idx: Position) -> &'a mut Vec<WFC_Tile> {
        let [x, y, z] = idx;
        return &mut self.values[x + y*self.size[0] + z*self.size[0]*self.size[1]]
    }
}


type Position = [usize; 3];
enum Direction {NORTH, EAST, SOUTH, WEST, UP, DOWN}

impl Direction {
    fn flip(&self) -> Direction {
        match self {
            Direction::NORTH => Direction::SOUTH,
            Direction::EAST  => Direction::WEST,
            Direction::UP    => Direction::DOWN,
            Direction::SOUTH => Direction::NORTH,
            Direction::WEST  => Direction::EAST,
            Direction::DOWN  => Direction::UP,
        }
    }
}

impl Into<usize> for Direction {
    fn into(self) -> usize {
        match self {
            Direction::NORTH => 0,
            Direction::EAST  => 1,
            Direction::SOUTH => 2,
            Direction::WEST  => 3,
            Direction::UP    => 4,
            Direction::DOWN  => 5,
        }
    }
}

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

    #[allow(non_snake_case)]
    fn print_worldmap(&self) {
        // debug print worldmap
        let [XS,YS,ZS] = self.worldmap.size;
        for x in 0..XS {
            for y in 0..YS {
                for z in 0..ZS {
                    print!("{:?} ", self.worldmap[(x,y,z)].len());
                }
            }
            println!("");
        }
    }

    fn find_squares_in_order(&self) -> Vec::<Position> {
        let mut available_squares = Vec::<Position>::with_capacity(1);
        for square in self.worldmap.tmp_iter() {
            if self.worldmap[square].len() == 1 {
                continue;
            }
            available_squares.push(square);
            return available_squares;
        }

        // always empty:
        return available_squares;
    }

    fn wfc_step(&mut self) -> Option<Position> {
        let available_squares = self.find_squares_in_order();

        if available_squares.len() == 0 {
            println!("done");
            return None
        }
        let square = *choose_random(&mut self.rng, &available_squares);

        // observe
        let selected_tile = *choose_random(&mut self.rng, &self.worldmap[square]);
        //println!("selected_tile: {:?}  square: {:?}  stack: {:?}", selected_tile, square, self.worldmap[square].len());
        self.worldmap[square].clear();
        self.worldmap[square].push(selected_tile);
        return Some(square);
    }

    /// returns true if we changed available connections, false otherwise
    fn update_tile_stack(&mut self, connections: &HashSet<usize>, square: Position, dir: Direction) -> bool {
        // we are trying to access tile beyond edge
        let square = match self.worldmap.move_(square, &dir) {
            Some(x) => x,
            None => return false,
        };

        // this direction has all connections available
        // FIXME: 3 is hardcoded, but should be calculated.
        //        probably should put it into WFC struct.
        //        can be calculated from original tiles.
        // TODO: self.worldmap.max_connections(dir) -> usize
        if connections.len() == 4 {
            return false;
        }

        let mut ok_stack = Vec::<WFC_Tile>::with_capacity(5);

        let stack = &self.worldmap[square];
        for tile in stack {
            let d: usize = dir.flip().into();
            if !connections.contains(&tile.connection_types[d]) {
                continue
            }
            ok_stack.push(*tile);
        }
        if self.worldmap[square].len() == ok_stack.len() {
            return false;
        }
        if ok_stack.len() == 0 {
            println!("error: tile-stack reduced to 0!!!  tile: {:?}", square);
            self.worldmap[square].clear();
            self.debug_break = true;
            return true;
        }
        self.worldmap[square] = ok_stack;
//        println!("update_tile_stack has changed connections {:?}", map_square);
        return true;
    }

    // finds all available connections from this tile for each direction
    fn gather_available_connections(&self, square: Position) -> Vec::<std::collections::HashSet<usize>> {
        let mut connections = Vec::<std::collections::HashSet<usize>>::with_capacity(4);
        for _ in 0..4 {
            connections.push(std::collections::HashSet::<usize>::new());
        }
        let stack = &self.worldmap[square];
        for tile in stack {
            for i in 0..4 {
                connections[i].insert(tile.connection_types[i]);
            }
        }
        return connections;
    }

    // wfc3d: map_square probably can be anything
    fn propagate(&mut self, square: Position) {
        if self.debug_break {
            return;
        }

        let connections = self.gather_available_connections(square);

        // clear direction
        let is_recurse0 = self.update_tile_stack(&connections[0], square, Direction::NORTH);
        let is_recurse1 = self.update_tile_stack(&connections[1], square, Direction::EAST);
        let is_recurse2 = self.update_tile_stack(&connections[2], square, Direction::SOUTH);
        let is_recurse3 = self.update_tile_stack(&connections[3], square, Direction::WEST);

        // recurse in that direction
        if is_recurse0 { self.propagate(self.worldmap.move_(square, &Direction::NORTH).unwrap()); }
        if is_recurse1 { self.propagate(self.worldmap.move_(square, &Direction::EAST).unwrap()); }
        if is_recurse2 { self.propagate(self.worldmap.move_(square, &Direction::SOUTH).unwrap()); }
        if is_recurse3 { self.propagate(self.worldmap.move_(square, &Direction::WEST).unwrap()); }
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
    let worldmap = Worldmap::new3d(MAP_WIDTH, MAP_HEIGHT, 1);
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

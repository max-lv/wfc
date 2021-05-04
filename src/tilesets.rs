
use crate::wfc::{WFC_Tile};

pub fn pipes() -> (String, Vec<WFC_Tile>) {
    let tilemap = String::from("./pipes_tileset.png");

    let mut tiles = Vec::new();
    tiles.push(WFC_Tile {
        col: 0,
        row: 0,
        connection_types: [1,1,0,1,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 2,
        row: 0,
        connection_types: [0,0,0,0,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 0,
        row: 1,
        connection_types: [0,1,0,1,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 1,
        row: 0,
        connection_types: [1,1,1,1,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 1,
        row: 1,
        connection_types: [0,0,1,1,0,0],
        angle: 0,
    });
    // connecting pipe
    tiles.push(WFC_Tile {
        col: 2,
        row: 1,
        connection_types: [0,1,0,2,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 3,
        row: 0,
        connection_types: [0,0,2,2,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 3,
        row: 1,
        connection_types: [0,2,0,2,0,0],
        angle: 0,
    });
    return (tilemap, tiles);
}

pub fn flat_city() -> (String, Vec<WFC_Tile>) {
    let tilemap = String::from("./flat-city.png");

    let mut tiles = Vec::new();
    tiles.push(WFC_Tile {
        col: 0,
        row: 0,
        connection_types: [1,1,1,1,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 1,
        row: 0,
        connection_types: [1,1,1,1,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 2,
        row: 0,
        connection_types: [1,0,1,0,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 3,
        row: 0,
        connection_types: [1,1,0,0,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 4,
        row: 0,
        connection_types: [1,1,1,1,0,0],
        angle: 0,
    });
    // walls
    tiles.push(WFC_Tile {
        col: 0,
        row: 1,
        connection_types: [1,2,1,2,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 1,
        row: 1,
        connection_types: [1,1,2,2,0,0],
        angle: 0,
    });
    // fat blocks
    tiles.push(WFC_Tile {
        col: 2,
        row: 1,
        connection_types: [1,1,3,3,0,0],
        angle: 0,
    });
//    tiles.push(WFC_Tile {
//        col: 3,
//        row: 1,
//        connection_types: [1,1,3,3,0,0],
//        angle: 0,
//    });
    tiles.push(WFC_Tile {
        col: 4,
        row: 1,
        connection_types: [3,3,3,3,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 5,
        row: 0,
        connection_types: [1,0,1,1,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 5,
        row: 1,
        connection_types: [1,0,0,0,0,0],
        angle: 0,
    });

    return (tilemap, tiles);
}

pub fn flat_city_paths_only() -> (String, Vec<WFC_Tile>) {
    let tilemap = String::from("./flat-city.png");

    let mut tiles = Vec::new();
    tiles.push(WFC_Tile {
        col: 0,
        row: 0,
        connection_types: [1,1,1,1,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 2,
        row: 0,
        connection_types: [1,0,1,0,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 3,
        row: 0,
        connection_types: [1,1,0,0,0,0],
        angle: 0,
    });
    tiles.push(WFC_Tile {
        col: 5,
        row: 0,
        connection_types: [1,0,1,1,0,0],
        angle: 0,
    });

    return (tilemap, tiles);
}

pub fn stairs_3d() -> (String, Vec<WFC_Tile>) {
    let tilemap = String::from("");

    let mut tiles = Vec::new();
    // empty
    tiles.push(WFC_Tile {
        col: 0,
        row: 0,
        connection_types: [0,0,0,0,0,0],
        angle: 0,
    });
    // stairs empty
    tiles.push(WFC_Tile {
        col: 5,
        row: 0,
        connection_types: [0,0,1,0,0,2],
        angle: 0,
    });
    // stairs
    tiles.push(WFC_Tile {
        col: 1,
        row: 0,
        connection_types: [1,0,0,0,2,0],
        angle: 0,
    });
    // line
    tiles.push(WFC_Tile {
        col: 2,
        row: 0,
        connection_types: [1,0,1,0,0,0],
        angle: 0,
    });
    // T-junction
    tiles.push(WFC_Tile {
        col: 3,
        row: 0,
        connection_types: [1,1,1,0,0,0],
        angle: 0,
    });
    // deadend
    tiles.push(WFC_Tile {
        col: 4,
        row: 0,
        connection_types: [1,0,0,0,0,0],
        angle: 0,
    });
//    // debug
//    tiles.push(WFC_Tile {
//        col: 5,
//        row: 0,
//        connection_types: [0,0,0,0,3,2],
//        angle: 0,
//    });
    return (tilemap, tiles);
}


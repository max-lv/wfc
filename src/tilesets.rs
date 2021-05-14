
use crate::wfc::{WfcTile, create_big_tile};

pub fn pipes() -> (String, Vec<WfcTile>, u32) {
    let tilemap = String::from("./pipes_tileset.png");

    let mut tiles = Vec::new();
    // T-junction
    tiles.push(WfcTile {
        index: 4*0 + 0, // col: 0, row: 0,
        connection_types: [1,1,0,1,0,0],
        angle: 0,
        is_rotatable: true,
    });
    // empty
    tiles.push(WfcTile {
        index: 4*0 + 2, // col: 2, row: 0,
        connection_types: [0,0,0,0,0,0],
        angle: 0,
        is_rotatable: true,
    });
    // line
    tiles.push(WfcTile {
        index: 4*1 + 0, // col: 0, row: 1,
        connection_types: [0,1,0,1,0,0],
        angle: 0,
        is_rotatable: true,
    });
    // X-junction
    tiles.push(WfcTile {
        index: 4*0 + 1, // col: 1, row: 0,
        connection_types: [1,1,1,1,0,0],
        angle: 0,
        is_rotatable: true,
    });
    // corner
    tiles.push(WfcTile {
        index: 4*1 + 1, // col: 1, row: 1,
        connection_types: [0,1,1,0,0,0],
        angle: 0,
        is_rotatable: true,
    });
    // connecting pipe
    tiles.push(WfcTile {
        index: 4*1 + 2, // col: 2, row: 1,
        connection_types: [0,2,0,1,0,0],
        angle: 0,
        is_rotatable: true,
    });
    // _red corner
    tiles.push(WfcTile {
        index: 4*0 + 3, // col: 3, row: 0,
        connection_types: [0,2,2,0,0,0],
        angle: 0,
        is_rotatable: true,
    });
    // _red line
    tiles.push(WfcTile {
        index: 4*1 + 3, // col: 3, row: 1,
        connection_types: [0,2,0,2,0,0],
        angle: 0,
        is_rotatable: true,
    });
    // big-tiles
    let mut conn = 1000;
    tiles.extend(create_big_tile(&mut conn, (2,2,1), vec![
        Some((4*2+0, [0,0,0,2,0,0])), Some((4*2+1, [0,1,0,0,0,0])),
        Some((4*3+0, [0,0,0,1,0,0])), Some((4*3+1, [0,2,0,0,0,0])),
    ]));
    tiles.extend(create_big_tile(&mut conn, (2,2,1), vec![
        Some((4*2+2, [0,9,9,0,0,0])), Some((4*2+3, [0,1,0,9,0,0])),
        Some((4*3+2, [9,0,2,0,0,0])), None,
    ]));
    return (tilemap, tiles, 4);
}

pub fn flat_city() -> (String, Vec<WfcTile>, u32) {
    let tilemap = String::from("./flat-city.png");

    let mut tiles = Vec::new();
    tiles.push(WfcTile {
        index: 8*0 + 0,
        connection_types: [1,1,1,1,0,0],
        angle: 0,
        is_rotatable: true,
    });
    tiles.push(WfcTile {
        index: 8*0 + 1,
        connection_types: [1,1,1,1,0,0],
        angle: 0,
        is_rotatable: true,
    });
    tiles.push(WfcTile {
        index: 8*0 + 2,
        connection_types: [1,0,1,0,0,0],
        angle: 0,
        is_rotatable: true,
    });
    tiles.push(WfcTile {
        index: 8*0 + 3,
        connection_types: [1,0,0,1,0,0],
        angle: 0,
        is_rotatable: true,
    });
    tiles.push(WfcTile {
        index: 8*0 + 4,
        connection_types: [1,1,1,1,0,0],
        angle: 0,
        is_rotatable: true,
    });
    // walls
    tiles.push(WfcTile {
        index: 8*1 + 0,
        connection_types: [1,2,1,2,0,0],
        angle: 0,
        is_rotatable: true,
    });
    tiles.push(WfcTile {
        index: 8*1 + 1,
        connection_types: [1,2,2,1,0,0],
        angle: 0,
        is_rotatable: true,
    });
    // fat blocks
    tiles.push(WfcTile {
        index: 8*1 + 2,
        connection_types: [1,3,3,1,0,0],
        angle: 0,
        is_rotatable: true,
    });
//    tiles.push(WfcTile {
//        col: 3,
//        row: 1,
//        connection_types: [1,1,3,3,0,0],
//        angle: 0,
//        is_rotatable: true,
//    });
    tiles.push(WfcTile {
        index: 8*1 + 4,
        connection_types: [3,3,3,3,0,0],
        angle: 0,
        is_rotatable: true,
    });
    tiles.push(WfcTile {
        index: 8*0 + 5,
        connection_types: [1,1,1,0,0,0],
        angle: 0,
        is_rotatable: true,
    });
    tiles.push(WfcTile {
        index: 8*1 + 5,
        connection_types: [1,0,0,0,0,0],
        angle: 0,
        is_rotatable: true,
    });

    return (tilemap, tiles, 8);
}

pub fn flat_city_paths_only() -> (String, Vec<WfcTile>, u32) {
    let tilemap = String::from("./flat-city.png");

    let mut tiles = Vec::new();
    tiles.push(WfcTile {
        index: 8*0 + 0,
        connection_types: [1,1,1,1,0,0],
        angle: 0,
        is_rotatable: true,
    });
    tiles.push(WfcTile {
        index: 8*0 + 2,
        connection_types: [1,0,1,0,0,0],
        angle: 0,
        is_rotatable: true,
    });
    tiles.push(WfcTile {
        index: 8*0 + 3,
        connection_types: [1,0,0,1,0,0],
        angle: 0,
        is_rotatable: true,
    });
    tiles.push(WfcTile {
        index: 8*0 + 5,
        connection_types: [1,1,1,0,0,0],
        angle: 0,
        is_rotatable: true,
    });

    return (tilemap, tiles, 8);
}

pub fn stairs_3d() -> (String, Vec<WfcTile>, u32) {
    let tilemap = String::from("");

    let mut tiles = Vec::new();
    // empty
    tiles.push(WfcTile {
        index: 0,
        connection_types: [0,0,0,0,0,0],
        angle: 0,
        is_rotatable: true,
    });
    // line
    tiles.push(WfcTile {
        index: 2,
        connection_types: [1,0,1,0,0,0],
        angle: 0,
        is_rotatable: true,
    });
    // T-junction
    tiles.push(WfcTile {
        index: 3,
        connection_types: [1,0,1,1,0,0],
        angle: 0,
        is_rotatable: true,
    });
    // deadend
    tiles.push(WfcTile {
        index: 4,
        connection_types: [1,0,0,0,0,0],
        angle: 0,
        is_rotatable: true,
    });
    // corner
    tiles.push(WfcTile {
        index: 5,
        connection_types: [1,0,0,1,0,0],
        angle: 0,
        is_rotatable: true,
    });
//    // debug
//    tiles.push(WfcTile {
//        index: 6,
//        connection_types: [0,0,0,0,3,2],
//        angle: 0,
//        is_rotatable: true,
//    });
    // 3d stairs
    let mut conn = 1000;
    tiles.extend(create_big_tile(&mut conn, (1,1,2), vec![
        Some((1, [1,0,0,0,0,0])), // stairs
        Some((0, [0,0,1,0,0,0])), // empty
    ]));
    return (tilemap, tiles, 0);
}

pub fn stairs_3d_path() -> (String, Vec<WfcTile>, u32, WfcTile) {
    let tilemap = String::from("");

    let mut tiles = Vec::new();
    // empty
    tiles.push(WfcTile {
        index: 0,
        connection_types: [0,0,0,0,0,0],
        angle: 0,
        is_rotatable: true,
    });
    // line
    tiles.push(WfcTile {
        index: 2,
        connection_types: [1,0,1,0,0,0],
        angle: 0,
        is_rotatable: true,
    });
    // corner
    tiles.push(WfcTile {
        index: 5,
        connection_types: [1,0,0,1,0,0],
        angle: 0,
        is_rotatable: true,
    });
    // deadend
    let deadend = WfcTile {
        index: 4,
        connection_types: [1,0,0,0,0,0],
        angle: 0,
        is_rotatable: true,
    };

    // 3d stairs
    let mut conn = 1000;
    tiles.extend(create_big_tile(&mut conn, (1,1,2), vec![
        Some((1, [1,0,0,0,0,0])), // stairs
        Some((0, [0,0,1,0,0,0])), // empty
    ]));
    return (tilemap, tiles, 0, deadend);
}


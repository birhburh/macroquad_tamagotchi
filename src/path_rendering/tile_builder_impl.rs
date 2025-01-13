use super::rasterizer::{TileBuilder, TILE_SIZE};

const BYTES_PER_PIXEL: usize = 4;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: [i16; 2],
    pub uv: [i16; 2],
    pub col: [u8; 4],
}

pub struct Builder {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub atlas: Vec<u8>,
    pub color: [u8; 4],
    next_row: u16,
    next_col: u16,
}

pub const ATLAS_SIZE: usize = 4096;

impl Builder {
    pub fn new() -> Builder {
        let mut atlas = vec![0; ATLAS_SIZE * ATLAS_SIZE * BYTES_PER_PIXEL];
        for row in 0..TILE_SIZE {
            for col in 0..TILE_SIZE {
                let first_byte = (row * ATLAS_SIZE + col) * BYTES_PER_PIXEL;
                atlas[first_byte] = 255;
                atlas[first_byte + 1] = 255;
                atlas[first_byte + 2] = 255;
                atlas[first_byte + 3] = 0;
            }
        }

        Builder {
            vertices: Vec::new(),
            indices: Vec::new(),
            atlas,
            next_row: 0,
            next_col: 1,
            color: [255; 4],
        }
    }
}

impl TileBuilder for Builder {
    fn tile(&mut self, x: i16, y: i16, data: [u8; TILE_SIZE * TILE_SIZE]) {
        let base = self.vertices.len() as u16;

        let u1 = (self.next_col * TILE_SIZE as u16) as i16;
        let u2 = ((self.next_col + 1) * TILE_SIZE as u16) as i16;
        let v1 = (self.next_row * TILE_SIZE as u16) as i16;
        let v2 = ((self.next_row + 1) * TILE_SIZE as u16) as i16;

        // dbg!([u1, u2, v1, v2]);

        self.vertices.extend_from_slice(&[
            Vertex {
                pos: [x, y],
                col: self.color,
                uv: [u1, v1],
            },
            Vertex {
                pos: [x + TILE_SIZE as i16, y],
                col: self.color,
                uv: [u2, v1],
            },
            Vertex {
                pos: [x + TILE_SIZE as i16, y + TILE_SIZE as i16],
                col: self.color,
                uv: [u2, v2],
            },
            Vertex {
                pos: [x, y + TILE_SIZE as i16],
                col: self.color,
                uv: [u1, v2],
            },
        ]);
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        // println!("tile at ({}, {}):", x, y);
        // for row in 0..TILE_SIZE {
        //     print!("  ");
        //     for col in 0..TILE_SIZE {
        //         print!("{:3} ", data[row * TILE_SIZE + col]);
        //     }
        //     print!("\n");
        // }
        for row in 0..TILE_SIZE {
            for col in 0..TILE_SIZE {
                self.atlas[self.next_row as usize * TILE_SIZE * ATLAS_SIZE * BYTES_PER_PIXEL
                    + row * ATLAS_SIZE * BYTES_PER_PIXEL
                    + self.next_col as usize * TILE_SIZE * BYTES_PER_PIXEL
                    + col * BYTES_PER_PIXEL] = data[row * TILE_SIZE + col];
            }
        }

        self.next_col += 1;
        if self.next_col as usize == ATLAS_SIZE / BYTES_PER_PIXEL / TILE_SIZE {
            self.next_col = 0;
            self.next_row += 1;
        }
    }

    fn span(&mut self, x: i16, y: i16, width: u16) {
        let base = self.vertices.len() as u16;

        self.vertices.push(Vertex {
            pos: [x, y],
            col: self.color,
            uv: [0, 0],
        });
        self.vertices.push(Vertex {
            pos: [x + (width as i16), y],
            col: self.color,
            uv: [0, 0],
        });
        self.vertices.push(Vertex {
            pos: [x + (width as i16), y + TILE_SIZE as i16],
            col: self.color,
            uv: [0, 0],
        });
        self.vertices.push(Vertex {
            pos: [x, y + TILE_SIZE as i16],
            col: self.color,
            uv: [0, 0],
        });
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

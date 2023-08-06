use crate::websocket::PlaceWebSocketConnection;
use actix::{Addr, Message};
use byteorder::{LittleEndian, ReadBytesExt};
use crossbeam::atomic::AtomicCell;
use flate2::write::GzEncoder;
use flate2::Compression;
use rand::Rng;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::sync::{Arc, Mutex};

type Grid = Vec<AtomicCell<u8>>;
type Color = (u8, u8, u8);

#[derive(Message, Clone, Copy)]
#[rtype(result = "()")]
pub struct UpdateMessage(pub usize, pub usize, pub usize, pub u8);

pub struct Voxel {
    pub id: i64,
    pub name: String,
    pub grid_size: (usize, usize, usize),
    pub grid: Grid,
    pub palette: Vec<Color>,
    pub path: String,
    sessions: Mutex<Vec<Addr<PlaceWebSocketConnection>>>,
}

impl Voxel {
    pub fn new(
        id: i64,
        name: &str,
        palette: Option<Vec<Color>>,
        grid_size: (usize, usize, usize),
        grid: Option<Grid>,
    ) -> Self {
        let grid = grid.unwrap_or_else(|| Voxel::generate_empty_grid(grid_size));

        let palette: Vec<Color> = vec![
            (0x6d, 0x00, 0x1a),
            (0xbe, 0x00, 0x39),
            (0xff, 0x45, 0x00),
            (0xff, 0xa8, 0x00),
            (0xff, 0xd6, 0x35),
            (0xff, 0xf8, 0xb8),
            (0x00, 0xa3, 0x68),
            (0x00, 0xcc, 0x78),
            (0x7e, 0xed, 0x56),
            (0x00, 0x75, 0x6f),
            (0x00, 0x9e, 0xaa),
            (0x00, 0xcc, 0xc0),
            (0x24, 0x50, 0xa4),
            (0x36, 0x90, 0xea),
            (0x51, 0xe9, 0xf4),
            (0x49, 0x3a, 0xc1),
            (0x6a, 0x5c, 0xff),
            (0x94, 0xb3, 0xff),
            (0x81, 0x1e, 0x9f),
            (0xb4, 0x4a, 0xc0),
            (0xe4, 0xab, 0xff),
            (0xde, 0x10, 0x7f),
            (0xff, 0x38, 0x81),
            (0xff, 0x99, 0xaa),
            (0x6d, 0x48, 0x2f),
            (0x9c, 0x69, 0x26),
            (0xff, 0xb4, 0x70),
            (0x00, 0x00, 0x00),
            (0x51, 0x52, 0x52),
            (0x89, 0x8d, 0x90),
            (0xd4, 0xd7, 0xd9),
            (0xff, 0xff, 0xff),
        ];

        Self {
            id,
            name: name.to_string(),
            grid_size,
            grid,
            palette,
            sessions: Mutex::new(Vec::new()),
            path: format!("voxels/{}.vxl", id),
        }
    }

    pub fn draw_voxel(
        &self,
        x: usize,
        y: usize,
        z: usize,
        color: u8,
    ) -> Result<(), String> {
        let grid = &self.grid;
        let at_bottom = y == 0;
        let mut has_neighbor = false;

        if x >= self.grid_size.0 || y >= self.grid_size.1 || z >= self.grid_size.2 {
            return Err("Out of bounds".to_string());
        }

        if x > 0 && grid[self.get_index(x - 1, y, z)].load() > 0 {
            has_neighbor = true;
        }
        if x < self.grid_size.0 - 1 && grid[self.get_index(x + 1, y, z)].load() > 0 {
            has_neighbor = true;
        }
        if y > 0 && grid[self.get_index(x, y - 1, z)].load() > 0 {
            has_neighbor = true;
        }
        if y < self.grid_size.1 - 1 && grid[self.get_index(x, y + 1, z)].load() > 0 {
            has_neighbor = true;
        }
        if z > 0 && grid[self.get_index(x, y, z - 1)].load() > 0 {
            has_neighbor = true;
        }
        if z < self.grid_size.2 - 1 && grid[self.get_index(x, y, z + 1)].load() > 0 {
            has_neighbor = true;
        }

        if at_bottom || has_neighbor || grid[self.get_index(x, y, z)].load() > 0 {
            grid[self.get_index(x, y, z)].store(color);
            self.broadcast(UpdateMessage(x, y, z, color));
            Ok(())
        } else {
            Err("Voxel has no neighbors".to_string())
        }
    }

    pub fn start_save_loop(self: &Arc<Self>) {
        let voxel_object_clone = Arc::clone(self);
        tokio::spawn(async move {
            voxel_object_clone.save_loop().await;
        });
    }

    async fn save_loop(self: &Arc<Self>) {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            self.write(&self.path).unwrap();
        }
    }

    pub fn add_session(&self, session: Addr<PlaceWebSocketConnection>) {
        self.sessions.lock().unwrap().push(session);
    }

    fn broadcast(&self, update_message: UpdateMessage) {
        let sessions = self.sessions.lock().unwrap();
        for session in sessions.iter() {
            session.do_send(update_message);
        }
    }

    fn get_index(&self, x: usize, y: usize, z: usize) -> usize {
        x * self.grid_size.0 * self.grid_size.0 + y * self.grid_size.1 + z
    }

    fn generate_random_grid(grid_size: (usize, usize, usize)) -> Grid {
        let mut rng = rand::thread_rng();
        let mut grid_data = Vec::new();
        for _ in 0..grid_size.0 {
            for y in 0..grid_size.1 {
                for _ in 0..grid_size.2 {
                    let voxel_spawn_rate =
                        1.0 / (1.0 + ((y as f64 / grid_size.1 as f64) * 16.0 - 1.0).exp());
                    if rng.gen::<f64>() < voxel_spawn_rate {
                        grid_data.push(rng.gen_range(1..=32));
                    } else {
                        grid_data.push(0);
                    }
                }
            }
        }
        grid_data.into_iter().map(AtomicCell::new).collect()
    }

    fn generate_empty_grid(grid_size: (usize, usize, usize)) -> Grid {
        let mut grid_data = Vec::new();
        for _ in 0..grid_size.0 {
            for _ in 0..grid_size.1 {
                for _ in 0..grid_size.2 {
                    grid_data.push(0);
                }
            }
        }
        grid_data.into_iter().map(AtomicCell::new).collect()
    }

    pub fn write(&self, path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        let mut bytes = Vec::new();

        bytes.extend_from_slice(b"VXL ");
        bytes.push(self.name.len() as u8);
        bytes.extend_from_slice(self.name.as_bytes());
        bytes.push(1);
        bytes.extend_from_slice(b"0100");
        bytes.push(self.palette.len() as u8);
        bytes.extend_from_slice(&(self.grid_size.0 as u16).to_le_bytes());
        bytes.extend_from_slice(&(self.grid_size.1 as u16).to_le_bytes());
        bytes.extend_from_slice(&(self.grid_size.2 as u16).to_le_bytes());

        for color in self.palette.iter() {
            bytes.push(color.0);
            bytes.push(color.1);
            bytes.push(color.2);
        }

        let grid: Vec<u8> = self.grid.iter().map(|cell| cell.load()).collect();

        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write_all(&grid).expect("Failed to write data");
        let compressed_data = e.finish().expect("Failed to finish compression");

        bytes.extend(compressed_data);

        writer.write_all(&bytes)?;
        writer.flush()?;

        Ok(())
    }

    pub fn read(path: &str, id: i64) -> Result<Voxel, std::io::Error> {
        let file = File::open(path)?;
        let mut reader = std::io::BufReader::new(file);

        let mut magic = [0; 4];
        reader.read_exact(&mut magic)?;
        if magic != *b"VXL " {
            panic!("Invalid magic number");
        }

        let name_length = reader.read_u8()?;
        let mut name = vec![0; name_length as usize];
        reader.read_exact(&mut name)?;
        let name = String::from_utf8(name).unwrap();

        reader.read_u8()?;

        let mut version = [0; 4];
        reader.read_exact(&mut version)?;
        if version != *b"0100" {
            panic!("Wrong version");
        }

        let palette_size = reader.read_u8()?;
        let grid_size_x = reader.read_u16::<LittleEndian>()?;
        let grid_size_y = reader.read_u16::<LittleEndian>()?;
        let grid_size_z = reader.read_u16::<LittleEndian>()?;

        let mut palette = Vec::new();
        for _ in 0..palette_size {
            let color: Color = (
                reader.read_u8()?,
                reader.read_u8()?,
                reader.read_u8()?,
            );
            palette.push(color);
        }

        let mut grid = Vec::new();
        let mut compressed_data = Vec::new();
        reader.read_to_end(&mut compressed_data)?;
        let mut decoder = flate2::read::GzDecoder::new(&compressed_data[..]);
        decoder.read_to_end(&mut grid)?;
        let grid = grid.into_iter().map(AtomicCell::new).collect();

        Ok(Self::new(
            id,
            &name,
            Some(palette),
            (
                grid_size_x as usize,
                grid_size_y as usize,
                grid_size_z as usize,
            ),
            Some(grid),
        ))
    }
}

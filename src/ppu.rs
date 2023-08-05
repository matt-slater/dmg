use crate::mmu;
use crate::utils;

const SCANLINE_TICKS: u16 = 456;
const SCREEN_LINES: u8 = 144;
const SCANLINE_PIXELS: u8 = 160;
const OAM_SEARCH_TICKS: u16 = 40;

enum PpuState {
    OamSearch,     // Object Attribute Memory
    PixelTransfer, // Push pixels to display
    HBlank,        // Time to next line
    VBlank,        // Time to next frame
}

pub struct Ppu {
    screen: Screen,
    ticks: u16,      // keeps track of timing for various states
    state: PpuState, // state of the PPU FSM
    ly: u8,          // current line on screen
    x: u8,           // current pixel on line
    fetcher: Fetcher,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            screen: Screen::new(),
            ticks: 0,
            state: PpuState::OamSearch,
            ly: 0,
            x: 0,
            fetcher: Fetcher::new(),
        }
    }

    pub fn tick(&mut self, mmu: &mut mmu::Mmu) {
        self.ticks += 1;

        match self.state {
            PpuState::OamSearch => {
                if self.ticks == OAM_SEARCH_TICKS {
                    self.x = 0;
                    let tile_line = self.ly % 8;
                    let tile_map_row_addr = 0x9800 + ((self.ly / 8) as u16 * 32);
                    self.fetcher.start(tile_map_row_addr, tile_line);
                    self.state = PpuState::PixelTransfer;
                }
            }
            PpuState::PixelTransfer => {
                self.fetcher.tick(mmu);
                if self.fetcher.rb.size() <= 8 {
                    return;
                }

                let pixel = self.fetcher.rb.get();
                self.screen.write(pixel);
                self.x += 1;
                if self.x == SCANLINE_PIXELS {
                    self.screen.h_blank();
                    self.state = PpuState::HBlank;
                }
            }
            PpuState::HBlank => {
                if self.ticks == SCANLINE_TICKS {
                    self.ticks = 0;
                    self.ly += 1;
                    mmu.write_byte(0xff44, self.ly);
                    if self.ly == SCREEN_LINES {
                        self.screen.v_blank();
                        self.state = PpuState::VBlank;
                    } else {
                        self.state = PpuState::OamSearch;
                    }
                }
            }
            PpuState::VBlank => {
                if self.ticks == SCANLINE_TICKS {
                    self.ticks = 0;
                    self.ly += 1;
                    mmu.write_byte(0xff44, self.ly);
                    if self.ly == (SCREEN_LINES + 9) {
                        self.ly = 0;
                        self.state = PpuState::OamSearch;
                    }
                }
            }
        }
    }
}

enum FetcherState {
    ReadTileId,
    ReadTileData0,
    ReadTileData1,
    PushToFifo,
}

struct Fetcher {
    state: FetcherState,
    ticks: u8,
    tile_index: u8,
    tile_line: u8,
    tile_id: u8,
    map_addr: u16,
    tile_data: [u8; 8],
    rb: utils::RingBuffer,
}

impl Fetcher {
    pub fn new() -> Self {
        Fetcher {
            state: FetcherState::ReadTileId,
            ticks: 0,
            tile_index: 0,
            map_addr: 0,
            tile_id: 0,
            tile_line: 0,
            tile_data: [0; 8],
            rb: utils::RingBuffer::new(16),
        }
    }

    pub fn tick(&mut self, mmu: &mut mmu::Mmu) {
        self.ticks += 1;
        if self.ticks < 2 {
            return;
        }
        self.ticks = 0;

        match self.state {
            FetcherState::ReadTileId => {
                self.tile_id = mmu.read_byte(self.map_addr + self.tile_index as u16);
                self.state = FetcherState::ReadTileData0;
            }
            FetcherState::ReadTileData0 => {
                let offset = 0x8000 + self.tile_id as u16;
                let addr = offset + ((self.tile_line as u16) * 2);
                let data = mmu.read_byte(addr);
                for i in 0..8 {
                    self.tile_data[i] = (data >> i) & 1;
                }

                self.state = FetcherState::ReadTileData1;
            }
            FetcherState::ReadTileData1 => {
                let offset = 0x8000 + self.tile_id as u16;
                let addr = offset + ((self.tile_line as u16) * 2);
                let data = mmu.read_byte(addr);
                for i in 0..8 {
                    self.tile_data[i] = ((data >> i) & 1) << 1;
                }
                self.state = FetcherState::PushToFifo;
            }
            FetcherState::PushToFifo => {
                if self.rb.size() <= 8 {
                    for i in (0..8).rev() {
                        self.rb.add(self.tile_data[i]);
                    }
                    self.tile_index += 1;
                    self.state = FetcherState::ReadTileId;
                }
            }
        }
    }

    pub fn start(&mut self, map_addr: u16, tile_line: u8) {
        self.tile_index = 0;
        self.map_addr = map_addr;
        self.tile_line = tile_line;
        self.state = FetcherState::ReadTileId;
        self.rb.clear();
    }
}

pub struct Screen {
    palette: [char; 4],
}

impl Screen {
    pub fn new() -> Self {
        Screen {
            palette: ['█', '▒', '░', ' '],
        }
    }

    pub fn write(&self, i: u8) {
        print!("{}{}", self.palette[i as usize], self.palette[i as usize]);
    }

    pub fn h_blank(&self) {
        print!("\n");
    }

    pub fn v_blank(&self) {
        print!("\n==========VBLANK=====================\n");
        //print!("\x1B[2J\x1B[1;1H");
    }
}

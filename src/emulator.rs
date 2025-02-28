use crate::Status;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

pub enum Variant {
    CosmacVip,
    SuperChip,
    SuperChipExtended
}

pub struct Emulator {
    registers: [u8; 16],
    i_register: u16,
    delay_timer: Arc<RwLock<u8>>,
    sound_timer: Arc<RwLock<u8>>,
    stack: Vec<u16>,
    program_counter: u16,
    stack_pointer: u8,
    memory: Arc<RwLock<Vec<u8>>>,
    status: Arc<RwLock<Status>>,
    display_state: Arc<RwLock<[[usize; 64]; 32]>>,
    clock_frequency: u64,
    last_instruction_cycle: Instant,
    keys: Arc<RwLock<[bool;16]>>,
    variant: Variant
}

impl Emulator {
    pub fn new(
        status: Arc<RwLock<Status>>,
        memory: Arc<RwLock<Vec<u8>>>,
        display_state: Arc<RwLock<[[usize; 64]; 32]>>,
        keys: Arc<RwLock<[bool;16]>>,
        variant: Variant
    ) -> Emulator {
        let delay_timer = Arc::new(RwLock::new(0));
        let sound_timer = Arc::new(RwLock::new(0));
        let dt_clone = delay_timer.clone();
        let st_clone = sound_timer.clone();
        let dt_status_clone = status.clone();
        let st_status_clone = status.clone();
        thread::spawn(|| run_timer(dt_clone, dt_status_clone));
        thread::spawn(|| run_timer(st_clone, st_status_clone));
        Emulator {
            registers: [0; 16],
            i_register: 0,
            delay_timer,
            sound_timer,
            stack: vec![0u16; 16],
            stack_pointer: 0,
            program_counter: 0,
            status,
            display_state,
            memory,
            clock_frequency: 600,
            last_instruction_cycle: Instant::now(),
            keys,
            variant
        }
    }

    fn execute_instruction(&mut self) {
        let read_memory = self.memory.read().unwrap();
        let instruction = (read_memory[self.program_counter as usize] as u16) << 8
            | read_memory[self.program_counter as usize + 1] as u16;
        self.program_counter += 2;
        drop(read_memory);

        // println!("Executing instruction: {:#x}, Program counter: {}", instruction, self.program_counter);

        match instruction & 0xF000 {
            0x0000 => match instruction & 0x00FF {
                0x00E0 => {
                    self.clear_screen();
                }
                0x00EE => {
                    self.program_counter = self.stack[self.stack_pointer as usize];
                    self.stack_pointer -= 1;
                }
                _ => {} //Syscall, ignored in emulators
            }
            0x1000 => {
                self.program_counter = instruction & 0x0FFF;
            }
            0x2000 => {
                self.stack_pointer += 1;
                self.stack[self.stack_pointer as usize] = self.program_counter;
                self.program_counter = instruction & 0x0FFF;
            }
            0x3000 => {
                if self.registers[((instruction & 0x0F00) >> 8) as usize] == (instruction & 0x0FF) as u8 {
                    self.program_counter += 2;
                }
            }
            0x4000 => {
                if self.registers[((instruction & 0x0F00) >> 8) as usize] != (instruction & 0x0FF) as u8 {
                    self.program_counter += 2;
                }
            }
            0x5000 => {
                if self.registers[((instruction & 0x0F00) >> 8) as usize] == self.registers[((instruction & 0x00F0) >> 4) as usize] {
                    self.program_counter += 2;
                }
            }
            0x6000 => {
                self.registers[((instruction & 0x0F00) >> 8) as usize] = (instruction & 0x00FF) as u8;
            }
            0x7000 => {
                self.registers[((instruction & 0x0F00) >> 8) as usize] = self
                    .registers[((instruction & 0x0F00) >> 8) as usize]
                    .wrapping_add((instruction & 0x00FF) as u8);
            }
            0x8000 => match instruction & 0x000F {
                0x0 => {
                    self.registers[((instruction & 0x0F00) >> 8) as usize] =
                        self.registers[((instruction & 0x00F0) >> 4) as usize];
                }
                0x1 => {
                    self.registers[((instruction & 0x0F00) >> 8) as usize] |=
                        self.registers[((instruction & 0x00F0) >> 4) as usize];
                    if matches!(self.variant, Variant::CosmacVip) {
                        self.registers[0xF] = 0;
                    }
                }
                0x2 => {
                    self.registers[((instruction & 0x0F00) >> 8) as usize] &=
                        self.registers[((instruction & 0x00F0) >> 4) as usize];
                    if matches!(self.variant, Variant::CosmacVip) {
                        self.registers[0xF] = 0;
                    }
                }
                0x3 => {
                    self.registers[((instruction & 0x0F00) >> 8) as usize] ^=
                        self.registers[((instruction & 0x00F0) >> 4) as usize];
                    if matches!(self.variant, Variant::CosmacVip) {
                        self.registers[0xF] = 0;
                    }
                }
                0x4 => {
                    let mut f_value = 0;
                    if 0xFF - self.registers[((instruction & 0x0F00) >> 8) as usize] < self.registers[((instruction & 0x00F0) >> 4) as usize] {
                        f_value = 1;
                    }
                    self.registers[((instruction & 0x0F00) >> 8) as usize] =
                        self.registers[((instruction & 0x0F00) >> 8) as usize].wrapping_add(self.registers[((instruction & 0x00F0) >> 4) as usize]);
                    self.registers[0xF] = f_value;
                }
                0x5 => {
                    let mut f_value = 0;
                    if self.registers[((instruction & 0x0F00) >> 8) as usize]
                        >= self.registers[((instruction & 0x00F0) >> 4) as usize] {
                        f_value = 1;
                    }
                    self.registers[((instruction & 0x0F00) >> 8) as usize] =
                        self.registers[((instruction & 0x0F00) >> 8) as usize].wrapping_sub(self.registers[((instruction & 0x00F0) >> 4) as usize]);
                    self.registers[0xF] = f_value;
                }
                0x6 => {
                    if !matches!(self.variant, Variant::SuperChip) && !matches!(self.variant, Variant::SuperChipExtended) {
                        self.registers[((instruction & 0x0F00) >> 8) as usize] = self.registers[((instruction & 0x0F0) >> 4) as usize]
                    };
                    let mut f_value = 0;
                    if self.registers[((instruction & 0x0F00) >> 8) as usize] & 0x1 == 1 {
                        f_value = 1;
                    }
                    self.registers[((instruction & 0x0F00) >> 8) as usize] = self.registers[((instruction & 0x0F00) >> 8) as usize].wrapping_shr(1);
                    self.registers[0xF] = f_value;
                }
                0x7 => {
                    let mut f_value = 0;
                    if self.registers[((instruction & 0x00F0) >> 4) as usize] >= self.registers[((instruction & 0x0F00) >> 8) as usize] {
                        f_value = 1;
                    }
                    self.registers[((instruction & 0x0F00) >> 8) as usize] =
                        self.registers[((instruction & 0x00F0) >> 4) as usize].wrapping_sub(self.registers[((instruction & 0x0F00) >> 8) as usize]);
                    self.registers[0xF] = f_value;
                }
                0xE => {
                    if !matches!(self.variant, Variant::SuperChip) && !matches!(self.variant, Variant::SuperChipExtended) {
                        self.registers[((instruction & 0x0F00) >> 8) as usize] = self.registers[((instruction & 0x0F0) >> 4) as usize]
                    };
                    let mut f_value = 0;
                    if self.registers[((instruction & 0x0F00) >> 8) as usize] & 0b1000_0000 == 0b1000_0000 {
                        f_value = 1;
                    }
                    self.registers[((instruction & 0x0F00) >> 8) as usize] = self.registers[((instruction & 0x0F00) >> 8) as usize].wrapping_shl(1);
                    self.registers[0xF] = f_value;
                }
                _ => {
                    println!("Not an instruction: {:#x}", instruction);
                }
            }
            0x9000 => {
                if self.registers[((instruction & 0x0F00) >> 8) as usize]
                    != self.registers[((instruction & 0x00F0) >> 4) as usize]
                {
                    self.program_counter += 2;
                }
            }
            0xA000 => {
                self.i_register = instruction & 0x0FFF;
            }
            0xB000 => { self.program_counter = (0x0FFF & instruction).wrapping_add(self.registers[0] as u16);}
            0xC000 => {
                //todo: implement a better RNG algorithm
                let mut rng = 0;
                for val in self.registers {rng ^= val;}
                self.registers[((0x0F00 & instruction)>>8) as usize] = rng & (0x00FF & instruction) as u8;
            }
            0xD000 => {
                let collision = self.draw_sprite(
                    ((instruction & 0x0F00) >> 8) as usize,
                    ((instruction & 0x00F0) >> 4) as usize,
                    (instruction & 0x000F) as u16,
                );
                if collision {
                    self.registers[15] = 1;
                } else {
                    self.registers[15] = 0;
                }
            }
            0xE000 => match instruction & 0x00FF {
                0x009E => {
                    let key_index = (self.registers[((0x0F00 & instruction)>>8) as usize] & 0xF) as usize;
                    let read_keys = self.keys.read().unwrap();
                    if read_keys[key_index] {
                        self.program_counter += 2;
                    }
                }
                0x00A1 => {
                    let key_value = (self.registers[((0x0F00 & instruction)>>8) as usize] &0xF) as usize;
                    let read_keys = self.keys.read().unwrap();
                    if !read_keys[key_value] {
                        self.program_counter += 2;
                    }
                }
                _ => {println!("Not an instruction: {:#x}", instruction);}
            }
            0xF000 => match instruction & 0x00FF {
                0x0007 => {
                    self.registers[((0xF00 & instruction)>>8) as usize] = *self.delay_timer.read().unwrap();
                }
                0x000A => {
                    let mut key_found = false;
                    let mut key_value = 0;
                    while !key_found {
                        let read_keys = self.keys.read().unwrap();
                        for i in 0..read_keys.len() {
                            if read_keys[i] {
                                key_found = true;
                                key_value = i;
                                break;
                            }
                        }
                        if !key_found {thread::sleep(Duration::from_millis(20));}
                    }
                    println!("Found pressed key: {}", key_value);
                    self.registers[((0x0F00 & instruction) >> 8) as usize] = key_value as u8;
                }
                0x0015 => {
                    *self.delay_timer.write().unwrap() = self.registers[((0xF00 & instruction)>>8) as usize];
                }
                0x0018 => {
                    *self.sound_timer.write().unwrap() = self.registers[((0xF00 & instruction)>>8) as usize];
                }
                0x001E => {
                    self.i_register = self.i_register.wrapping_add(self.registers[((0xF00 & instruction)>>8) as usize] as u16);
                }
                0x0029 => {
                    match self.variant {
                        Variant::CosmacVip => {
                            self.i_register = 5 * (self.registers[((0xF00 & instruction)>>8) as usize] & 0xF) as u16;
                        }
                        _ => {self.i_register = 5 * self.registers[((0xF00 & instruction)>>8) as usize] as u16;}
                    }
                }
                0x0033 => {
                    let num = self.registers[((instruction & 0x0F00) >> 8) as usize];
                    let hundreds = num / 100;
                    let tens = num % 100 / 10;
                    let ones = num % 10;
                    let mut write_memory = self.memory.write().unwrap();
                    write_memory[self.i_register as usize] = hundreds;
                    write_memory[self.i_register as usize + 1] = tens;
                    write_memory[self.i_register as usize + 2] = ones;
                }
                0x0055 => {
                    let x = ((instruction & 0x0F00) >> 8) as usize;
                    let mut write_memory = self.memory.write().unwrap();
                    for i in 0 ..x+1 {
                        write_memory[self.i_register as usize + i] = self.registers[i];
                    }
                    match self.variant {
                        Variant::CosmacVip => {self.i_register += (x + 1) as u16;}
                        _ => {}
                    }
                }
                0x0065 => {
                    let x = ((instruction & 0x0F00) >> 8) as usize;
                    let read_memory = self.memory.read().unwrap();
                    for i in 0 ..x+1 {
                        self.registers[i] = read_memory[i + self.i_register as usize];
                    }
                    match self.variant {
                        Variant::CosmacVip => {self.i_register += (x + 1) as u16;}
                        _ => {}
                    }
                }
                _ => {println!("Not an instruction: {:#x}", instruction);}
            }
            _ => {
                println!("Not an instruction: {:#x}", instruction);
            }
        }
    }

    pub fn run(&mut self) {
        let mut op_count = 0;
        let ops_per_cycle = 16;

        loop {
            let status_read = self.status.read().unwrap();
            match *status_read {
                Status::Running => {}
                Status::Stopped => {
                    return;
                }
                _ => {
                    drop(status_read);
                    thread::sleep(Duration::from_millis(250));
                    continue;
                }
            }
            drop(status_read);
            if (self.program_counter + 1) as usize >= self.memory.read().unwrap().len() {
                return;
            }
            if *self.sound_timer.read().unwrap() > 0 { //todo: implement actual audio
                // run_audio();
            }

            if op_count == 0 {
                self.last_instruction_cycle = Instant::now();
            }
            if op_count == ops_per_cycle - 1 {
                let time_from_ops_ms = Instant::now()
                    .duration_since(self.last_instruction_cycle)
                    .as_millis() as u64;
                let expected_ms = 1000 / self.clock_frequency * ops_per_cycle;
                if time_from_ops_ms < expected_ms {
                    thread::sleep(Duration::from_millis(expected_ms - time_from_ops_ms));
                }
            }
            self.execute_instruction();
            op_count = (op_count + 1) % ops_per_cycle;
        }
    }

    fn clear_screen(&self) {
        let mut display = self.display_state.write().unwrap();
        for i in 0..display.len() {
            for j in 0..display[i].len() {
                display[i][j] = 0;
            }
        }
    }

    fn draw_sprite(&self, reg1: usize, reg2: usize, n: u16) -> bool {
        let mut collision_flag = false;

        let initial_x = self.registers[reg1];
        let initial_y = self.registers[reg2];

        let mut y = initial_y as usize;
        let read_memory = self.memory.read().unwrap();
        for i in self.i_register..self.i_register + n {
            let mut x = initial_x as usize;
            let mut write_display = self.display_state.write().unwrap();
            let sprite_line = read_memory[i as usize];
            for n in 0..8 {
                let bit = (sprite_line & (1 << 7 - n)) >> 7 - n;
                // Modulus is used to wrap to the other side.
                // For example, if attempting to write at [34][67]
                // they become -> [2][3] instead
                if write_display[y%32][x%64] & bit as usize == 1 {
                    collision_flag = true;
                }
                write_display[y%32][x%64] ^= bit as usize;
                x += 1;
            }
            y += 1;
        }
        collision_flag
    }
}

fn run_timer(timer: Arc<RwLock<u8>>, status: Arc<RwLock<Status>>) {
    loop {
        let duration = Duration::from_millis(1000 / 60);
        let mut timer_write = timer.write().unwrap();
        let status_read = status.read().unwrap();
        if matches!(*status_read, Status::Stopped) {
            return;
        }
        if *timer_write > 0 && matches!(*status_read, Status::Running) {
            *timer_write = *timer_write - 1;
        }
        drop(timer_write);
        drop(status_read);
        thread::sleep(duration);
    }
}

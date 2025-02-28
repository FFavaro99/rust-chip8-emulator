#![windows_subsystem = "windows"]

use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::{Arc, RwLock};
use std::thread;
use rfd::FileDialog;
use crate::emulator::Emulator;
use crate::gui::run_gui;

mod emulator;
mod gui;
mod sound;

pub enum Status {
    Starting,
    Running,
    Paused,
    Stopped,
}

pub fn main() {
    // Initializing the shared state
    let display_state: Arc<RwLock<[[usize;64];32]>> = Arc::new(RwLock::new([[0;64];32]));
    let memory: Arc<RwLock<Vec<u8>>> = Arc::new(RwLock::new(Vec::with_capacity(4096)));
    let status: Arc<RwLock<Status>> = Arc::new(RwLock::new(Status::Starting));
    let keys: Arc<RwLock<[bool;16]>> = Arc::new(RwLock::new([false;16]));

    let mut is_running = false;
    match load_game(memory.clone()) {
        Some(()) => {
            let mut status_write = status.write().unwrap();
            *status_write = Status::Running;
            is_running = true;
        },
        _ => {}
    }

    if is_running {
        let mut emulator = Emulator::new(status.clone(), memory.clone(), display_state.clone(), keys.clone());
        let emulator_handle = thread::spawn(move || emulator.run());
        let display_state_copy = display_state.clone();
        let pressed_key_gui_copy = keys.clone();
        let status_clone = status.clone();
        let gui_handle = thread::spawn(|| run_gui(display_state_copy, pressed_key_gui_copy, status_clone));
        gui_handle.join().unwrap();
        {
            let mut status_write = status.write().unwrap();
            *status_write = Status::Stopped;
        }
        emulator_handle.join().unwrap();
    }



}

fn load_game(memory: Arc<RwLock<Vec<u8>>>) -> Option<()>{
    let files = FileDialog::new()
        .add_filter("Chip 8", &["ch8"])
        .set_directory("/")
        .set_title("Choose a Chip 8 Program")
        .pick_file();

    if files.is_none() {return None};

    let file = File::open(files.unwrap()).unwrap();
    let buf = BufReader::new(file);
    let mut ram = memory.write().unwrap();
    *ram = vec![0;4096];
    for i in 0..SPRITES.len() {
        ram[i] = SPRITES[i];
    }
    let mut index = 0x200;
    buf.bytes().for_each(|i| {
        let byte = i.unwrap();
        ram[index] = byte;
        index += 1;
    });
    Some(())
}

const SPRITES: [u8;80] = [0xF0,0x90,0x90,0x90,0xF0, 0x20,0x60,0x20,0x20,0x70, 0xF0,0x10,0xF0,0x80,0xF0, 0xF0,0x10,0xF0,0x10,0xF0, 0x90,0x90,0xF0,0x10,0x10, 0xF0,0x80,0xF0,0x10,0xF0, 0xF0,0x80,0xF0,0x90,0xF0, 0xF0,0x10,0x20,0x40,0x40, 0xF0,0x90,0xF0,0x90,0xF0, 0xF0,0x90,0xF0,0x10,0xF0, 0xF0,0x90,0xF0,0x90,0x90, 0xE0,0x90,0xE0,0x90,0xE0, 0xF0,0x80,0x80,0x80,0xF0, 0xE0,0x90,0x90,0x90,0xE0, 0xF0,0x80,0xF0,0x80,0xF0, 0xF0,0x80,0xF0,0x80,0x80];
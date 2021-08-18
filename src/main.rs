mod tests;

use std::fs::File;
use std::io::{prelude::*, stdout};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::{self};

use kira::arrangement::{Arrangement, LoopArrangementSettings};
use kira::instance::{InstanceLoopStart, InstanceSettings, StopInstanceSettings};
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::SoundSettings;
use pixels::{Pixels, SurfaceTexture};
use winit::event::{ElementState, KeyboardInput};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use serde_derive::Deserialize;

mod emulator;
use emulator::*;

// when using the c8_test rom, refer to this documentation https://github.com/Skosulor/c8int/blob/master/test/chip8_test.txt
const DEFAULT_ROM: &str = "roms/c8_test.c8";
const CYCLE_SLEEP_DURATION: time::Duration = time::Duration::from_millis(16);
const INSTRUCTIONS_PER_CYCLE: u8 = 10;

// general todo
// todo implement error handling

/*
    - code ingestion
        - opcodes are 2 bytes 0xFFFF
    - registers
        data (16 0-F) 8 bits each
        address register 16bit
    - memory
        program address space 0x200 - 0x69F (2kb variant)
                              0x200 - 0xE8F (4kb variant) (3,215 bytes available)
        big-endian (x86_64 is little endian)
    - beep sound (lets make it a 4th octave G sound)
    - timers (run at 60hz)
        - delay timer
        - sound timer
    - display

    keyboard mapping:
    Keypad      Keyboard
    1 2 3 C     1 2 3 4
    4 5 6 D ->  Q W E R
    7 8 9 E     A S D F
    A 0 B F     Z X C V

    screen 64 x 32 pixels
    top left 00, 00
    bottom right 3F, 1F
*/

#[derive(Debug, Deserialize)]
struct Config {
    rom: Option<String>,
}

fn main() {
    // based on 4kb variant (hence 3215 bytes) (wait shouldn't it be 3583???)
    // all memory accesses will be in big endian
    // chip 8 is big endian!!!
    // halt on bad memory access?

    // planning on making these count down on another thread
    // use a condition variable to wake the counting thread on counter set
    // just use thread sleep to try and get the right 60hz freq

    //1. load program into memory from file
    //  if it exceeds size than halt with error
    //2. begin running byte code
    // (code is allowed to be self modifying (ie no write protection region))
    // error on any address read/write below 0x200

    let (event_loop, window, mut emulator) = init();
    let rom_path = get_rom_path();
    let bytes_read = emulator.load_program(&rom_path);
    println!("Loaded program, bytes {}", bytes_read);
    window.request_redraw();

    // putting the audio play / stop on its own thread so its not
    // affected by the emulation processing
    let sound_counter = Arc::clone(&emulator.sound_counter);
    thread::spawn(move || {
        let mut audio_manager = AudioManager::new(AudioManagerSettings::default()).unwrap();
        let sound_handle = audio_manager
            .load_sound("beep2.wav", SoundSettings::new())
            .unwrap();
        let mut arrangement_handle = audio_manager
            .add_arrangement(Arrangement::new_loop(
                &sound_handle,
                LoopArrangementSettings::default(),
            ))
            .unwrap();
        let mut is_beep_playing = false;

        // beep is loud af, so we turn that shit down
        let play_instance_settings = InstanceSettings {
            volume: 0.3.into(),
            ..InstanceSettings::default()
        };

        loop {
            let counter_val = sound_counter.load(Ordering::Acquire);
            if !is_beep_playing && counter_val > 0 {
                is_beep_playing = true;
                arrangement_handle.play(play_instance_settings).unwrap();
            }
            if is_beep_playing && counter_val == 0 {
                is_beep_playing = false;
                arrangement_handle
                    .stop(StopInstanceSettings::default())
                    .unwrap();
            }
            //we yield to keep this thread well behaved in the OS's eyes
            //this should actually help keep it more responsive on a normal system
            thread::yield_now();
        }
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The closed button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => emulator.pixels_surface_resize(size),
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                scancode, state, ..
                            },
                        ..
                    },
                ..
            } => {
                if scancode == 0x001 {
                    // escape hit
                    println!("Escape key hit, closing");
                    *control_flow = ControlFlow::Exit;
                }
                update_key_states(scancode, state, &mut emulator);
            }

            Event::MainEventsCleared => {
                emulator.update_time_counters();
                for _ in 1..INSTRUCTIONS_PER_CYCLE {
                    match emulator.execute_next_instruction() {
                        InstructionResult::Terminated => {
                            println!("Emulator self terminating");
                            *control_flow = ControlFlow::Exit;
                        }
                        InstructionResult::RedrawRequested => window.request_redraw(),
                        _ => (),
                    }
                }

                // so that stdout prints show up when printed
                stdout().flush().unwrap();
                thread::sleep(CYCLE_SLEEP_DURATION);
            }
            Event::RedrawRequested(_) => {
                emulator.pixels_render();
            }
            _ => (),
        }
    });
}

fn update_key_states(scancode: u32, state: ElementState, emulator: &mut Emulator) {
    // x key -> 0
    if scancode == 0x02D {
        if state == ElementState::Pressed {
            emulator.key_states[0x0] = true;
        } else {
            emulator.key_states[0x0] = false;
        }
    }
    // 1 key -> 1
    if scancode == 0x002 {
        if state == ElementState::Pressed {
            emulator.key_states[0x1] = true;
        } else {
            emulator.key_states[0x1] = false;
        }
    }
    // 2 key -> 2
    if scancode == 0x003 {
        if state == ElementState::Pressed {
            emulator.key_states[0x2] = true;
        } else {
            emulator.key_states[0x2] = false;
        }
    }
    // 3 key -> 3
    if scancode == 0x004 {
        if state == ElementState::Pressed {
            emulator.key_states[0x3] = true;
        } else {
            emulator.key_states[0x3] = false;
        }
    }
    // Q key -> 4
    if scancode == 0x010 {
        if state == ElementState::Pressed {
            emulator.key_states[0x4] = true;
        } else {
            emulator.key_states[0x4] = false;
        }
    }
    // W key -> 5
    if scancode == 0x011 {
        if state == ElementState::Pressed {
            emulator.key_states[0x5] = true;
        } else {
            emulator.key_states[0x5] = false;
        }
    }
    // E key -> 6
    if scancode == 0x012 {
        if state == ElementState::Pressed {
            emulator.key_states[0x6] = true;
        } else {
            emulator.key_states[0x6] = false;
        }
    }
    // A key -> 7
    if scancode == 0x01E {
        if state == ElementState::Pressed {
            emulator.key_states[0x7] = true;
        } else {
            emulator.key_states[0x7] = false;
        }
    }
    // S key -> 8
    if scancode == 0x01F {
        if state == ElementState::Pressed {
            emulator.key_states[0x8] = true;
        } else {
            emulator.key_states[0x8] = false;
        }
    }
    // D key -> 9
    if scancode == 0x020 {
        if state == ElementState::Pressed {
            emulator.key_states[0x9] = true;
        } else {
            emulator.key_states[0x9] = false;
        }
    }
    // Z key -> A
    if scancode == 0x02C {
        if state == ElementState::Pressed {
            emulator.key_states[0xA] = true;
        } else {
            emulator.key_states[0xA] = false;
        }
    }
    // C key -> B
    if scancode == 0x02E {
        if state == ElementState::Pressed {
            emulator.key_states[0xB] = true;
        } else {
            emulator.key_states[0xB] = false;
        }
    }
    // 4 key -> C
    if scancode == 0x005 {
        if state == ElementState::Pressed {
            emulator.key_states[0xC] = true;
        } else {
            emulator.key_states[0xC] = false;
        }
    }
    // R key -> D
    if scancode == 0x013 {
        if state == ElementState::Pressed {
            emulator.key_states[0xD] = true;
        } else {
            emulator.key_states[0xD] = false;
        }
    }
    // F key -> E
    if scancode == 0x021 {
        if state == ElementState::Pressed {
            emulator.key_states[0xE] = true;
        } else {
            emulator.key_states[0xE] = false;
        }
    }
    // V key -> F
    if scancode == 0x02F {
        if state == ElementState::Pressed {
            emulator.key_states[0xF] = true;
        } else {
            emulator.key_states[0xF] = false;
        }
    }
}

fn get_rom_path() -> String {
    let mut file_buffer = String::new();
    if let Ok(mut config_file) = File::open("chip8_rust_config.toml") {
        config_file.read_to_string(&mut file_buffer).unwrap();
    }
    let decoded_toml: Config = toml::from_str(&file_buffer).unwrap();
    println!("{:#?}", decoded_toml);
    let mut rom_path = DEFAULT_ROM.to_string();
    if let Some(path) = decoded_toml.rom {
        rom_path = path;
    }
    rom_path
}

fn init() -> (EventLoop<()>, winit::window::Window, Emulator) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("chip8_rust")
        .build(&event_loop)
        .unwrap();
    let pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(64, 32, surface_texture).unwrap()
    };
    let emulator = Emulator::new(pixels);
    (event_loop, window, emulator)
}

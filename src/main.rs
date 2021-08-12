use std::io::{prelude::*, stdout};
use std::thread;
use std::time;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod emulator;
use emulator::*;

const DEFAULT_ROM: &str = "roms/test_opcode.ch8";
const CYCLE_SLEEP_DURATION: time::Duration = time::Duration::from_millis(16);

// general todo
// todo get windowing up and running with winit
// todo draw pixel grid with pixels library
// todo tests!
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

    let mut emulator = Emulator::new();
    let bytes_read = emulator.load_program(DEFAULT_ROM);
    println!("Loaded program, bytes {}", bytes_read);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(64, 32, surface_texture).unwrap()
    };

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
            Event::MainEventsCleared => {
                if emulator.execute_cycle() == CycleResult::Terminated {
                    println!("Emulator self terminating");
                    *control_flow = ControlFlow::Exit;
                }

                // so that stdout prints show up when printed
                stdout().flush().unwrap();
                thread::sleep(CYCLE_SLEEP_DURATION);

                //todo only request redraw when emulator runs a visual instruction (clear, draw)
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                pixel_placer(pixels.get_frame());
                // render
                //todo need to start properly handling errors :|
                pixels.render();
            }
            _ => (),
        }
    });
}

fn pixel_placer(frame: &mut [u8]) {
    for rgba_chunk in frame.chunks_exact_mut(4) {
        // rgba
        let colour = &[0xff, 0x00, 0x00, 0xff];
        rgba_chunk.copy_from_slice(colour);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn jazz() {
        assert!(false);
    }
}

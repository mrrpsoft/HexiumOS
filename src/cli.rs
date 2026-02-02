use crate::writer::Writer;
use crate::keyboard::Keyboard;
use crate::vga_colors::Color;
use crate::idt;
use crate::snake::SnakeGame;
use crate::video_player::VideoPlayer;
use crate::bad_apple_data::{FRAME_COUNT as BAD_APPLE_FRAME_COUNT, FRAME_WIDTH as BAD_APPLE_FRAME_WIDTH, FRAME_HEIGHT as BAD_APPLE_FRAME_HEIGHT, TARGET_FPS as BAD_APPLE_TARGET_FPS, FRAMES as BAD_APPLE_FRAMES};
use crate::RAHH_data::{FRAME_COUNT as RAHH_FRAME_COUNT, FRAME_WIDTH as RAHH_FRAME_WIDTH, FRAME_HEIGHT as RAHH_FRAME_HEIGHT, TARGET_FPS as RAHH_TARGET_FPS, FRAMES as RAHH_FRAMES};

const MAX_COMMAND_LEN: usize = 80;

pub struct CLI {
    buffer: [u8; MAX_COMMAND_LEN],
    buffer_len: usize,
    shift_pressed: bool,
}

impl CLI {
    pub const fn new() -> Self {
        Self {
            buffer: [0; MAX_COMMAND_LEN],
            buffer_len: 0,
            shift_pressed: false,
        }
    }

    pub fn show_prompt(&self, writer: &mut Writer) {
        writer.set_color(Color::LightGreen, Color::Black);
        writer.write_str("HexiumOS> ");
        writer.set_color(Color::White, Color::Black);
    }

    pub fn run(&mut self, writer: &mut Writer) -> ! {
        self.show_prompt(writer);

        loop {
            let scancode = match idt::get_scancode() {
                Some(sc) => sc,
                None => {
                    idt::wait_for_interrupt();
                    continue;
                }
            };

            if scancode & 0x80 != 0 {
                if scancode == 0xAA || scancode == 0xB6 {
                    self.shift_pressed = false;
                }
                continue;
            }

            if scancode == 0x2A || scancode == 0x36 {
                self.shift_pressed = true;
                continue;
            }

            if scancode == 0x1C {
                writer.write_byte(b'\n');
                self.execute_command(writer);
                self.buffer_len = 0;
                self.show_prompt(writer);
                continue;
            }

            if scancode == 0x0E {
                if self.buffer_len > 0 {
                    self.buffer_len -= 1;
                    self.delete_char(writer);
                }
                continue;
            }

            if let Some(c) = Keyboard::scancode_to_char(scancode, self.shift_pressed) {
                if self.buffer_len < MAX_COMMAND_LEN {
                    self.buffer[self.buffer_len] = c as u8;
                    self.buffer_len += 1;
                    writer.write_byte(c as u8);
                }
            }
        }
    }

    fn delete_char(&self, writer: &mut Writer) {
        unsafe {
            let vga = 0xb8000 as *mut u8;
            if writer.get_col() > 0 {
                let col = writer.get_col() - 1;
                let row = writer.get_row();
                let offset = (row * 80 + col) * 2;
                *vga.add(offset) = b' ';
                writer.set_position(col, row);
            }
        }
    }

    fn execute_command(&self, writer: &mut Writer) {
        if self.buffer_len == 0 {
            return;
        }

        let cmd = &self.buffer[..self.buffer_len];

        if cmd == b"help" {
            writer.set_color(Color::LightCyan, Color::Black);
            writer.write_str("Available commands:\n");
            writer.set_color(Color::White, Color::Black);
            writer.write_str("  help         - Show this help message\n");
            writer.write_str("  clear        - Clear the screen\n");
            writer.write_str("  hello        - Print a greeting\n");
            writer.write_str("  info         - Display system information\n");
            writer.write_str("  echo <text>  - Echo back the text\n");
            writer.write_str("  snake        - Play the snake game\n");
            writer.write_str("  play <video> - Play a video (badapple)\n");
        } else if cmd == b"clear" {
            writer.clear();
        } else if cmd == b"hello" {
            writer.set_color(Color::Yellow, Color::Black);
            writer.write_str("Hello from HexiumOS!\n");
            writer.set_color(Color::White, Color::Black);
        } else if cmd == b"info" {
            writer.set_color(Color::LightCyan, Color::Black);
            writer.write_str("=== Hexium HexiumOS ===\n");
            writer.set_color(Color::White, Color::Black);
            writer.write_str("A simple operating system written in Rust\n");
            writer.write_str("Version: 0.1.0\n");
        } else if cmd.starts_with(b"echo ") {
            writer.write_bytes(&cmd[5..]);
            writer.write_byte(b'\n');
        } else if cmd.starts_with(b"play ") {
            let video_name = &cmd[5..];
            if video_name == b"badapple" {
                let mut player = VideoPlayer::new(BAD_APPLE_FRAMES, BAD_APPLE_FRAME_COUNT, BAD_APPLE_FRAME_WIDTH, BAD_APPLE_FRAME_HEIGHT, BAD_APPLE_TARGET_FPS);
                player.run();
                writer.clear();
                writer.write_str("Video finished!\n");
            } else if video_name == b"RAHH" {
                let mut player = VideoPlayer::new(RAHH_FRAMES, RAHH_FRAME_COUNT, RAHH_FRAME_WIDTH, RAHH_FRAME_HEIGHT, RAHH_TARGET_FPS);
                player.run();
                writer.clear();
                writer.write_str("Video finished!\n");
            } else {
                writer.set_color(Color::Red, Color::Black);
                writer.write_str("Unknown video: ");
                writer.write_bytes(video_name);
                writer.write_str("\nAvailable videos: badapple\n");
                writer.set_color(Color::White, Color::Black);
            }
        } else if cmd == b"snake" {
            let mut game = SnakeGame::new();
            game.run(writer);
            writer.clear();
            writer.write_str("Thanks for playing!\n");
        } else {
            writer.set_color(Color::Red, Color::Black);
            writer.write_str("Unknown command: ");
            writer.write_bytes(cmd);
            writer.write_str("\nType 'help' for available commands.\n");
            writer.set_color(Color::White, Color::Black);
        }
    }
}

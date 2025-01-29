use std::fs::File;
use std::io::Read;
use std::io::Write;

fn main() {
    let mut _terminal = termion::raw::IntoRawMode::into_raw_mode(std::io::stdout()).unwrap();
    let mut stdout = std::io::stdout();
    let mut x = 1;
    let mut y = 1;
    let selectable_chars = vec!["-", "|", "<", ">", " "];
    let mut character = '-';
    let mut char_index = 0;
    // create a buffer to store screen content
    let mut screen_buffer: Vec<Vec<char>> = vec![vec![' '; 1000]; 1000];

    write!(
        stdout,
        "{}{}",
        termion::cursor::Goto(1, 1),
        termion::clear::All
    )
    .unwrap();
    write!(
        stdout,
        "\x1b[?1002h\x1b[?1006h" // Enable mouse reporting with SGR protocol
    )
    .unwrap();
    write!(stdout, "{}", termion::cursor::Goto(1, 3)).unwrap();
    draw_borders(&mut stdout, character);

    stdout.flush().unwrap();

    let stdin = std::io::stdin();
    for c in stdin.bytes() {
        if let Ok((w, h)) = termion::terminal_size() {
            static mut LAST_SIZE: (u16, u16) = (0, 0);
            unsafe {
                if (w, h) != LAST_SIZE {
                    write!(stdout, "{}", termion::clear::All).unwrap();
                    draw_borders(&mut stdout, character);
                    LAST_SIZE = (w, h);
                }
            }
        }

        match c.unwrap() {
            b'\x1B' => {
                let seq: [u8; 2] = [
                    std::io::stdin().bytes().next().unwrap().unwrap(),
                    std::io::stdin().bytes().next().unwrap().unwrap(),
                ];
                match seq {
                    [b'[', b'<'] => {
                        let mut num = String::new();
                        let mut nums = Vec::new();

                        loop {
                            let b = std::io::stdin().bytes().next().unwrap().unwrap() as char;
                            match b {
                                '0'..='9' => num.push(b),
                                ';' => {
                                    if !num.is_empty() {
                                        nums.push(num.parse::<u16>().unwrap());
                                        num.clear();
                                    }
                                }
                                'M' | 'm' => {
                                    if !num.is_empty() {
                                        nums.push(num.parse::<u16>().unwrap());
                                    }
                                    if nums.len() >= 3 {
                                        // handle click and drag
                                        if nums[0] == 0 || nums[0] == 32 {
                                            x = nums[1];
                                            y = nums[2];
                                            write!(
                                                stdout,
                                                "{}{}{}",
                                                termion::cursor::Goto(x, y),
                                                character,
                                                termion::cursor::Goto(x, y)
                                            )
                                            .unwrap();
                                            screen_buffer[y as usize][x as usize] = character;
                                            stdout.flush().unwrap();
                                        }
                                    }
                                    break;
                                }
                                _ => (),
                            }
                        }
                    }
                    [b'[', b'A'] => {
                        if y > 1 {
                            y -= 1
                        }
                    }
                    [b'[', b'B'] => y += 1,
                    [b'[', b'C'] => x += 1,
                    [b'[', b'D'] => {
                        if x > 1 {
                            x -= 1
                        }
                    }
                    _ => (),
                }
                write!(stdout, "{}", termion::cursor::Goto(x, y)).unwrap();
                stdout.flush().unwrap();
            }
            b'w' => {
                // clear buffer
                screen_buffer = vec![vec![' '; 1000]; 1000];

                // clear
                let (w, h) = termion::terminal_size().unwrap();
                for y in 3..h - 1 {
                    write!(
                        stdout,
                        "{}{}",
                        termion::cursor::Goto(1, y),
                        " ".repeat((w - 1) as usize)
                    )
                    .unwrap();
                }
                stdout.flush().unwrap();

                // Show temporary notification
                let (save_x, save_y) = (x, y);
                std::thread::spawn(move || {
                    let mut thread_stdout = std::io::stdout();
                    write!(
                        thread_stdout,
                        "{}screen cleared!{}",
                        termion::cursor::Goto(w - 14, h),
                        termion::cursor::Goto(save_x, save_y)
                    )
                    .unwrap();
                    thread_stdout.flush().unwrap();

                    std::thread::sleep(std::time::Duration::from_secs(3));

                    write!(
                        thread_stdout,
                        "{}{}{}",
                        termion::cursor::Goto(w - 15, h),
                        " ".repeat(16),
                        termion::cursor::Goto(save_x, save_y)
                    )
                    .unwrap();
                    thread_stdout.flush().unwrap();
                });
            }
            b'x' => {
                char_index = (char_index + 1) % selectable_chars.len();
                character = selectable_chars[char_index].chars().next().unwrap();
                let (_w, h) = termion::terminal_size().unwrap();
                write!(
                    stdout,
                    "{}current character: {}{}",
                    termion::cursor::Goto(2, h),
                    character,
                    termion::cursor::Goto(x, y)
                )
                .unwrap();
                stdout.flush().unwrap();
            }
            b' ' => {
                write!(
                    stdout,
                    "{}{}{}",
                    termion::cursor::Goto(x, y),
                    character,
                    termion::cursor::Goto(x, y)
                )
                .unwrap();
                // save
                screen_buffer[y as usize][x as usize] = character;
                stdout.flush().unwrap();
            }
            b's' => {
                save_screen_content(&screen_buffer);
                let (w, h) = termion::terminal_size().unwrap();
                let (save_x, save_y) = (x, y);

                std::thread::spawn(move || {
                    let mut thread_stdout = std::io::stdout();
                    write!(
                        thread_stdout,
                        "{}saved to file!{}",
                        termion::cursor::Goto(w - 14, h),
                        termion::cursor::Goto(save_x, save_y)
                    )
                    .unwrap();
                    thread_stdout.flush().unwrap();

                    std::thread::sleep(std::time::Duration::from_secs(3));

                    write!(
                        thread_stdout,
                        "{}{}{}",
                        termion::cursor::Goto(w - 14, h),
                        " ".repeat(14),
                        termion::cursor::Goto(save_x, save_y)
                    )
                    .unwrap();
                    thread_stdout.flush().unwrap();
                });
            }
            b'l' => {
                if let Ok(content) = std::fs::read_to_string("screen_content.txt") {
                    // clear existing buffer
                    screen_buffer = vec![vec![' '; 1000]; 1000];

                    // gone
                    let (w, h) = termion::terminal_size().unwrap();
                    for y in 3..h - 1 {
                        write!(
                            stdout,
                            "{}{}",
                            termion::cursor::Goto(1, y),
                            " ".repeat((w - 1) as usize)
                        )
                        .unwrap();
                    }

                    // load stuff
                    let mut current_y = 3; // below border
                    for line in content.lines() {
                        for (i, ch) in line.chars().enumerate() {
                            let pos_x = (i + 1) as u16;
                            screen_buffer[current_y as usize][pos_x as usize] = ch;
                            write!(stdout, "{}{}", termion::cursor::Goto(pos_x, current_y), ch)
                                .unwrap();
                        }
                        current_y += 1;
                    }

                    // put cursor back
                    write!(stdout, "{}", termion::cursor::Goto(x, y)).unwrap();
                    stdout.flush().unwrap();

                    // on loaded
                    std::thread::spawn(move || {
                        let mut thread_stdout = std::io::stdout();
                        write!(
                            thread_stdout,
                            "{}loaded from file!{}",
                            termion::cursor::Goto(w - 16, h),
                            termion::cursor::Goto(x, y)
                        )
                        .unwrap();
                        thread_stdout.flush().unwrap();

                        std::thread::sleep(std::time::Duration::from_secs(3));

                        write!(
                            thread_stdout,
                            "{}{}{}",
                            termion::cursor::Goto(w - 20, h),
                            " ".repeat(21),
                            termion::cursor::Goto(x, y)
                        )
                        .unwrap();
                        thread_stdout.flush().unwrap();
                    });
                }
            }
            b'k' => {
                // show indicator
                let (w, h) = termion::terminal_size().unwrap();
                write!(
                    stdout,
                    "{}TYPING MODE{}",
                    termion::cursor::Goto(w - 11, h),
                    termion::cursor::Goto(x, y)
                )
                .unwrap();
                stdout.flush().unwrap();

                let typing_stdin = std::io::stdin();
                for typed in typing_stdin.bytes() {
                    match typed.unwrap() {
                        b'\x1B' => {
                            // quit typing mode on esc and get rid of the indicator
                            write!(
                                stdout,
                                "{}{}{}",
                                termion::cursor::Goto(w - 11, h),
                                " ".repeat(11),
                                termion::cursor::Goto(x, y)
                            )
                            .unwrap();
                            stdout.flush().unwrap();
                            break;
                        }
                        b'\x7F' => {
                            // handle backspace
                            if x > 1 {
                                x -= 1;
                                write!(
                                    stdout,
                                    "{}{}{}",
                                    termion::cursor::Goto(x, y),
                                    " ",
                                    termion::cursor::Goto(x, y)
                                )
                                .unwrap();
                                screen_buffer[y as usize][x as usize] = ' ';
                                stdout.flush().unwrap();
                            }
                        }
                        byte => {
                            if byte.is_ascii() && !byte.is_ascii_control() {
                                let ch = byte as char;
                                write!(stdout, "{}{}", termion::cursor::Goto(x, y), ch).unwrap();
                                screen_buffer[y as usize][x as usize] = ch;
                                x += 1;
                                write!(stdout, "{}", termion::cursor::Goto(x, y)).unwrap();
                                stdout.flush().unwrap();
                            }
                        }
                    }
                }
            }
            b'q' => {
                write!(
                    stdout,
                    "\x1b[?1000l\x1b[?1015l\x1b[?1006l" // Disable mouse reporting
                )
                .unwrap();
                break;
            }
            _ => (),
        }
    }
}

fn draw_borders(stdout: &mut std::io::Stdout, character: char) {
    let size = termion::terminal_size().unwrap();
    let width = size.0;
    let height = size.1;
    let (current_x, current_y) = termion::cursor::DetectCursorPos::cursor_pos(stdout).unwrap();

    write!(stdout, "{}flowdraw by eri", termion::cursor::Goto(2, 1),).unwrap();
    // draw top bar
    for i in 1..width {
        write!(stdout, "{}─", termion::cursor::Goto(i, 1 + 1)).unwrap();
    }

    // draw bottom bar
    for i in 1..width {
        write!(stdout, "{}─", termion::cursor::Goto(i, height - 1)).unwrap();
    }
    write!(
        stdout,
        "{}current character: {} | change char: x | type mode: k | save: s | load: l | clear: w | quit: q",
        termion::cursor::Goto(2, height),
        character
    )
    .unwrap();
    write!(stdout, "{}", termion::cursor::Goto(current_x, current_y)).unwrap();
    stdout.flush().unwrap();
}

fn save_screen_content(buffer: &Vec<Vec<char>>) {
    let mut file = File::create("screen_content.txt").unwrap();
    let (width, height) = termion::terminal_size().unwrap();

    for y in 3..height - 2 {
        let mut line = String::new();
        for x in 1..width {
            line.push(buffer[y as usize][x as usize]);
        }

        writeln!(file, "{}", line.trim_end()).unwrap();
    }
}

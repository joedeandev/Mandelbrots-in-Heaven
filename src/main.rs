use crossterm::{cursor, event, style, terminal, QueueableCommand};
use num::complex::Complex;
use std::io::{stdout, Stdout, Write};

fn lerp(perc: f64, low: u8, high: u8) -> u8 {
    let percentage: f64 = if perc < 1.0 {
        if perc > 0.0 {
            perc
        } else {
            0.0
        }
    } else {
        1.0
    };
    let low = low as f64;
    let high = high as f64;
    let result = low + ((high - low) * percentage);
    return result as u8;
}

fn scale_color(value: u16, max_value: u16) -> (u8, u8, u8) {
    let none_color = (0, 0, 0);
    let colors = [
        [13, 0, 51],
        [20, 28, 132],
        [111, 118, 210],
        [220, 106, 136],
        [240, 120, 140],
    ];

    if value == 0 {
        return none_color;
    }
    let percentage: f64 = value as f64 / max_value as f64;
    let low_index = (percentage * ((colors.len() - 2) as f64)) as usize;
    return (
        lerp(percentage, colors[low_index][0], colors[low_index + 1][0]),
        lerp(percentage, colors[low_index][1], colors[low_index + 1][1]),
        lerp(percentage, colors[low_index][2], colors[low_index + 1][2]),
    );
}

fn calculate_instability(c: Complex<f64>, max_iterations: u16) -> u16 {
    let mut prev_z = Complex::new(0.0, 0.0);
    for iteration in 1..=max_iterations {
        prev_z = (prev_z * prev_z) + c;
        if prev_z.norm() > 2.0 {
            return iteration;
        }
    }
    return 0;
}

fn generate_mandelbrot(
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    width: usize,
    height: usize,
    max_iterations: u16,
) -> (Vec<u16>, u16) {
    let mut grid: Vec<u16> = Vec::with_capacity(width * height);
    let mut max_value: u16 = 0;

    let width_delta = (x_max - x_min) / (width as f64);
    let height_delta = (y_max - y_min) / (height as f64);
    for height_interval in 0..height {
        for width_interval in 0..width {
            let x_pt = (x_min + (width_delta / 2.0)) + (width_interval as f64 * width_delta);
            let y_pt = (y_min + (height_delta / 2.0)) + (height_interval as f64 * height_delta);
            let c = Complex::new(x_pt, y_pt);
            let instability = calculate_instability(c, max_iterations);
            max_value = if instability > max_value {
                instability
            } else {
                max_value
            };
            grid.push(instability);
        }
    }

    return (grid, max_value);
}

fn draw_mandelbrot(
    output: &mut Stdout,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    width: usize,
    height: usize,
    max_iterations: u16,
) -> () {
    let (grid, max_value) =
        generate_mandelbrot(x_min, x_max, y_min, y_max, width, height, max_iterations);
    let mut x: usize = 0;
    let mut y: usize = 0;

    output.queue(cursor::Hide);

    let mut prev_value = 0;

    for index in 0..grid.len() {
        if index % width == 0 {
            x = 0;
            output.queue(cursor::MoveTo(x as u16, y as u16));
            y += 1;
        }

        let value = grid[index];

        if index == 0 || value != prev_value {
            prev_value = value;
            let color_value = scale_color(value, max_value);
            let color = style::Color::Rgb {
                r: color_value.0,
                g: color_value.1,
                b: color_value.2,
            };
            output.queue(style::SetBackgroundColor(color));
        }

        output.queue(style::Print(" "));
        x += 1;
    }

    output.queue(cursor::MoveTo(0, 0));
    output.queue(style::ResetColor);
    output.flush();
}

fn get_bounds(
    origin_x: f64,
    origin_y: f64,
    x_size: f64,
    y_size: f64,
    terminal_width: usize,
    terminal_height: usize,
) -> (f64, f64, f64, f64) {
    let x_size = x_size.abs();
    let y_size = y_size.abs();

    let mut x_min = origin_x - (x_size * 0.5);
    let mut x_max = origin_x + (x_size * 0.5);

    let mut y_min = origin_y - (y_size * 0.5);
    let mut y_max = origin_y + (y_size * 0.5);

    // TODO: fix this so that the ratio actually remains consistent
    let x_ratio = (x_max - x_min) / terminal_width as f64;
    let y_ratio = ((y_max - y_min) / terminal_height as f64) / 2.5;

    if x_ratio > y_ratio {
        let y_size = terminal_height as f64 * x_ratio;
        y_min -= y_size / 2.0;
        y_max += y_size / 2.0;
    }
    if y_ratio > x_ratio {
        let x_size = terminal_width as f64 * y_ratio;
        x_min -= x_size / 2.0;
        x_max += x_size / 2.0;
    }

    return (x_min, x_max, y_min, y_max);
}

fn print_help(output: &mut Stdout) {
    output.queue(cursor::MoveTo(0, 0));
    output.queue(style::ResetColor);
    output.queue(style::Print(
        "--------------------------------------------------\n",
    ));
    output.queue(style::Print(
        " Mandelbrot's in Heaven - joe@joedean.dev         \n",
    ));
    output.queue(style::Print(
        " Left click: Zoom in on point                     \n",
    ));
    output.queue(style::Print(
        " Right click: Zoom out from point                 \n",
    ));
    output.queue(style::Print(
        " i: Increase iterations                           \n",
    ));
    output.queue(style::Print(
        " j: Decrease iterations                           \n",
    ));
    output.queue(style::Print(
        " r: Reset settings                                \n",
    ));
    output.queue(style::Print(
        " c: See coords                                    \n",
    ));
    output.queue(style::Print(
        " q: Quit program                                  \n",
    ));
    output.queue(style::Print(
        "--------------------------------------------------",
    ));
    output.flush();
}

fn print_iterations(output: &mut Stdout, iterations: u16) {
    output.queue(cursor::MoveTo(0, 0));
    output.queue(style::ResetColor);
    output.queue(style::Print(
        "--------------------------------------------------\n",
    ));
    output.queue(style::Print(
        String::from(" Iterations: ")
            + &iterations.to_string()
            + &String::from("                                \n"),
    ));
    output.queue(style::Print(
        "--------------------------------------------------",
    ));
    output.flush();
}

fn print_coordinates(output: &mut Stdout, origin_x: f64, origin_y: f64, x_size: f64, y_size: f64) {
    output.queue(cursor::MoveTo(0, 0));
    output.queue(style::ResetColor);
    output.queue(style::Print(
        "--------------------------------------------------\n",
    ));
    output.queue(style::Print(
        String::from(" Origin X: ") + &origin_x.to_string() + &String::from("          \n"),
    ));
    output.queue(style::Print(
        String::from(" Origin Y: ") + &origin_y.to_string() + &String::from("          \n"),
    ));
    output.queue(style::Print(
        String::from(" Size (X): ") + &x_size.to_string() + &String::from("          \n"),
    ));
    output.queue(style::Print(
        String::from(" Size (Y): ") + &y_size.to_string() + &String::from("          \n"),
    ));
    output.queue(style::Print(
        "--------------------------------------------------\n",
    ));
    output.flush();
}

fn main() {
    let (mut terminal_width, mut terminal_height) = match terminal::size() {
        Result::Ok((w, h)) => (w as usize - 1, h as usize - 1),
        _ => (80, 80),
    };

    let origin_x_default = -0.75;
    let origin_y_default = 0.0;
    let x_size_default = 3.0;
    let y_size_default = 3.0;
    let iterations_default = 50;

    let mut zoom_factor = 0.25;
    let mut iterations = iterations_default;

    let mut origin_x = origin_x_default;
    let mut origin_y = origin_y_default;
    let mut x_size = x_size_default;
    let mut y_size = y_size_default;

    let mut bounds = (0.0, 0.0, 0.0, 0.0);

    let mut output = stdout();

    output.queue(terminal::EnterAlternateScreen);
    output.queue(terminal::SetTitle("Mandelbrot's in Heaven"));
    output.queue(terminal::Clear(terminal::ClearType::All));
    output.queue(cursor::DisableBlinking);
    output.queue(cursor::Hide);
    output.queue(event::EnableMouseCapture);
    output.flush();

    let mut changed = true;
    let mut show_help = true;
    let mut show_iterations = false;
    let mut show_coords = false;

    loop {
        match event::read() {
            Ok(result) => match result {
                event::Event::Key(key_event) => match key_event.code {
                    event::KeyCode::Char(key_character) => match key_character {
                        'q' => {
                            break;
                        }
                        'r' => {
                            origin_x = origin_x_default;
                            origin_y = origin_y_default;
                            x_size = x_size_default;
                            y_size = y_size_default;
                            iterations = iterations_default;
                            changed = true;
                        }
                        'c' => {
                            show_coords = true;
                            changed = true;
                        }
                        'i' => {
                            if iterations >= 1000 {
                                iterations += 1000;
                            } else if iterations >= 100 {
                                iterations += 100;
                            } else if iterations >= 10 {
                                iterations += 10;
                            } else {
                                iterations += 1;
                            }

                            show_iterations = true;
                            changed = true;
                        }
                        'j' => {
                            if iterations >= 2000 {
                                iterations -= 1000;
                            } else if iterations >= 200 {
                                iterations -= 100;
                            } else if iterations >= 20 {
                                iterations -= 10;
                            } else if iterations >= 2 {
                                iterations -= 1;
                            } else {
                                iterations = 1;
                            }
                            show_iterations = true;
                            changed = true;
                        }
                        _ => {
                            print_help(&mut output);
                        }
                    },
                    _ => {
                        print_help(&mut output);
                    }
                },

                event::Event::Mouse(mouse_event) => match mouse_event.kind {
                    event::MouseEventKind::Down(mouse_button) => {
                        let (row, column) = (mouse_event.row as f64, mouse_event.column as f64);

                        origin_x =
                            ((column / terminal_width as f64) * (bounds.1 - bounds.0)) + bounds.0;
                        origin_y =
                            ((row / terminal_height as f64) * (bounds.3 - bounds.2)) + bounds.2;

                        match mouse_button {
                            event::MouseButton::Left => {
                                x_size *= zoom_factor;
                                y_size *= zoom_factor;
                                changed = true;
                            }
                            event::MouseButton::Right => {
                                x_size /= zoom_factor;
                                y_size /= zoom_factor;
                                changed = true;
                            }
                            _ => print_help(&mut output),
                        }
                    }
                    _ => {}
                },

                event::Event::Resize(width, height) => {
                    terminal_width = width as usize - 1;
                    terminal_height = height as usize - 1;
                    changed = true;
                    show_help = true;
                }
            },
            Err(result) => {}
        }

        if changed {
            bounds = get_bounds(
                origin_x,
                origin_y,
                x_size,
                y_size,
                terminal_width,
                terminal_height,
            );
            draw_mandelbrot(
                &mut output,
                bounds.0,
                bounds.1,
                bounds.2,
                bounds.3,
                terminal_width,
                terminal_height,
                iterations,
            );
            changed = false;

            if show_help {
                print_help(&mut output);
                show_help = false;
            } else if show_iterations {
                print_iterations(&mut output, iterations);
                show_iterations = false;
            } else if show_coords {
                print_coordinates(&mut output, origin_x, origin_y, x_size, y_size);
                show_coords = false;
            }
        }

        output.flush();
    }

    output.queue(terminal::LeaveAlternateScreen);
    output.flush();
}

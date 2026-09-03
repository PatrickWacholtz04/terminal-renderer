use crossterm::{
    cursor,
    queue,
    style::{
        Color,
        Print,
        ResetColor,
        SetBackgroundColor,
        SetForegroundColor,
    },
    terminal::{
        self,
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};

use std::io::{Result, Stdout, Write, stdout};
use std::{thread, time::Duration};


const OUT_W:usize = 100;
const OUT_H:usize = 100;

#[derive(Default, Clone, Copy, Debug)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB {
    fn to_crossterm(self) -> Color {
        Color::Rgb {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
struct Point {
    x: usize,
    y: usize,
}

struct Renderer {
    out_w: usize,
    out_h: usize,
    screen_buffer: Vec<Vec<RGB>>,
    draw_pixel: char,
    stdout: Stdout,
}

impl Renderer {
    fn new() -> Self {
        // Create a screen buffer the size of the output containing an RGB value for each pixel
        Self {
            out_w: OUT_W,
            out_h: OUT_H,
            screen_buffer: vec![vec![RGB::default(); OUT_H]; OUT_W],
            draw_pixel: '▄',
            stdout: stdout(),
        }
        
    }

    fn set_pixel(&mut self, point: Point, color: RGB) {
        self.screen_buffer[point.x][point.y] = color;
    }

    fn draw_line(&mut self, point0: Point, point1: Point, color: RGB) {
        // Generate line using Bresenham's Line Algorithm
            // https://www.youtube.com/watch?v=CceepU1vIKo

        let x0 = point0.x;
        let x1 = point1.x;
        let y0 = point0.y;
        let y1 = point1.y;

        let dx = x1 as isize - x0 as isize;
        let dy = y1 as isize - y0 as isize;

        if dx.abs() > dy.abs() {
            self.draw_line_horizontal(point0, point1, color);
        }
        else {
            self.draw_line_vertical(point0, point1, color);
        }
    }

    fn draw_line_horizontal(&mut self, point0: Point, point1: Point, color: RGB) {
        // Generate line using Bresenham's Line Algorithm
            // https://www.youtube.com/watch?v=CceepU1vIKo

        let mut x0 = point0.x;
        let mut x1 = point1.x;
        let mut y0 = point0.y;
        let mut y1 = point1.y;

        if x0 > x1 {
            (x0, x1) = (x1, x0);
            (y0, y1) = (y1, y0);
        }

        let dx = x1 as isize - x0 as isize;
        let mut dy = y1 as isize - y0 as isize;

        let dir = if dy < 0 {-1} else {1};
        dy *= dir;

        if dx != 0 {
            let mut y = y0 as isize;
            let mut p = 2*dy - dx;
            for i in 0..(dx+1) {
                
                self.set_pixel(Point{x: x0 + i as usize, y: y as usize}, color);

                if p >= 0 {
                    y += dir;
                    p = p - 2*dx;
                }
                p = p + 2*dy;
            }
        }
    }

    fn draw_line_vertical(&mut self, point0: Point, point1: Point, color: RGB) {
        // Generate line using Bresenham's Line Algorithm
            // https://www.youtube.com/watch?v=CceepU1vIKo

        let mut x0 = point0.x;
        let mut x1 = point1.x;
        let mut y0 = point0.y;
        let mut y1 = point1.y;

        if y0 > y1 {
            (x0, x1) = (x1, x0);
            (y0, y1) = (y1, y0);
        }

        let mut dx = x1 as isize - x0 as isize;
        let dy = y1 as isize - y0 as isize;

        let dir = if dx < 0 {-1} else {1};
        dx *= dir;

        if dy != 0 {
            let mut x = x0 as isize;
            let mut p = 2*dx - dy;
            for i in 0..(dy+1) {
                self.set_pixel(Point{x: x as usize, y: y0 + i as usize}, color);

                if p >= 0 {
                    x += dir;
                    p = p - 2*dy;
                }
                p = p + 2*dx;
            }
        }
    }

    fn draw_triangle(&mut self, point0: Point, point1: Point, point2: Point, color: RGB) {
        // Draw lines connecting points into a triangle
        self.draw_line(point0, point1, color);
        self.draw_line(point1, point2, color);
        self.draw_line(point2, point0, color);
    }

    fn render_crossterm(&mut self) -> Result<()> {     
        // Reset cursor position to top left instead of letting terminal scroll
        queue!(
            self.stdout,
            cursor::MoveTo(0, 0),
        )?;

        // Loop through screen buffer
        for y in ( 0..self.out_h).step_by(2) {
            for x in 0..self.out_w {
                let rgb_top = self.screen_buffer[x][y];
                let rgb_btm = 
                    if y + 1 < self.out_h { self.screen_buffer[x][y+1] }
                    else { RGB::default() };

                // Add draw pixel to queue with color
                queue!(
                    self.stdout,
                    SetForegroundColor(rgb_btm.to_crossterm()),
                    SetBackgroundColor(rgb_top.to_crossterm()),
                    Print(self.draw_pixel),
                    ResetColor,
                )?;
            }
            // Move cursor to start of next line
            queue!(
                self.stdout,
                cursor::MoveTo(0, (y / 2 + 1) as u16),
            )?;
        }

        // Clear color settings
        queue!(
            self.stdout,
            ResetColor,
        )?;
        // Flush queue to screen
        self.stdout.flush()?;
        Ok(())
    }


    fn render(&self) {
        for y in ( 0..self.out_h).step_by(2) {
            for x in 0..self.out_w {
                let rgb_top = self.screen_buffer[x][y];
                let rgb_btm = 
                    if y + 1 < self.out_h { self.screen_buffer[x][y+1] }
                    else { RGB::default() }; 

                // Background Color (Top Pixel)
                // Foreground Color (Bottom Pixel)
                // Pixel Character
                // Reset Color(s)
                print!("\x1b[48;2;{};{};{}m\
                        \x1b[38;2;{};{};{}m\
                        {}\
                        \x1b[0m", 
                        rgb_top.r, rgb_top.g, rgb_top.b, 
                        rgb_btm.r, rgb_btm.g, rgb_btm.b, 
                        self.draw_pixel
                    );
            }
            println!();
        }
    }

}


fn main() -> Result<()> {
    let mut renderer = Renderer::new();

    renderer.stdout.execute(EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    renderer.stdout.execute(cursor::Hide)?;

    let result = run(&mut renderer);

    // Always restore terminal state.
    renderer.stdout.execute(cursor::Show)?;
    terminal::disable_raw_mode()?;
    renderer.stdout.execute(LeaveAlternateScreen)?;

    result
}


fn run(renderer: &mut Renderer) -> Result<()> {
    let mut frame_counter = 0;

    loop {
        frame_counter += 1;

        // Clear the logical framebuffer.
        renderer.screen_buffer.fill(
            vec![RGB::default(); renderer.out_h]
        );

        renderer.draw_triangle(
            Point { x: 5, y: 5 },
            Point { x: 22, y: 13 },
            Point { x: 5, y: 20 },
            RGB {
                r: 0,
                g: 0,
                b: 255,
            },
        );

        renderer.draw_triangle(
            Point { x: 5, y: 5 },
            Point { x: 22, y: 13 },
            Point { x: 14, y: 1 },
            RGB {
                r: 255,
                g: 0,
                b: 0,
            },
        );

        renderer.render_crossterm()?;

        thread::sleep(Duration::from_millis(16));
    }


}
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
    event::{
        self,
        Event,
        KeyCode,
        KeyEvent,
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


const OUT_W:usize = 256;
const OUT_H:usize = 256;

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
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
pub struct Point {
    pub x: usize,
    pub y: usize,
}

pub struct Renderer {
    pub out_w: usize,
    pub out_h: usize,
    pub screen_buffer: Vec<Vec<RGB>>,
    previous_buffer: Vec<Vec<RGB>>,
    draw_pixel: char,
    stdout: Stdout,
    pub target_framerate: u16,
    pub first_frame: bool,
}

pub struct InputHandler {
    pub exit: bool,
    pub terminal_resized: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            exit: false,
            terminal_resized: false,
        }
    }

    pub fn update(&mut self) -> Result<()> {
        self.terminal_resized = false;
        while event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(KeyEvent { code, .. }) => {
                    match code {
                        KeyCode::Esc => {
                            self.exit = true;
                        }
                        _ => {}
                    }
                }

                Event::Resize(_width, _height) => {
                    self.terminal_resized = true;
                }

                _ => {}
            }
        }

        Ok(())
    }

}

impl Renderer {
    pub fn new() -> Self {
        // Create a screen buffer the size of the output containing an RGB value for each pixel
        Self {
            out_w: OUT_W,
            out_h: OUT_H,
            screen_buffer: vec![vec![RGB::default(); OUT_H]; OUT_W],
            previous_buffer: vec![vec![RGB::default(); OUT_H]; OUT_W],
            draw_pixel: '▄',
            stdout: stdout(),
            target_framerate: 15,
            first_frame: true,
        }
        
    }

    fn set_pixel(&mut self, point: Point, color: RGB) {
        if point.x < self.out_w && point.y < self.out_h {
            self.screen_buffer[point.x][point.y] = color;
        }
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

    pub fn draw_triangle(&mut self, point0: Point, point1: Point, point2: Point, color: RGB) {
        // Draw lines connecting points into a triangle
        self.draw_line(point0, point1, color);
        self.draw_line(point1, point2, color);
        self.draw_line(point2, point0, color);
    }

    pub fn fill_triangle(&mut self, mut point0: Point, mut point1: Point, mut point2: Point, color: RGB) {
        // https://github.com/OneLoneCoder/Javidx9/blob/master/ConsoleGameEngine/olcConsoleGameEngine.h#L537

        let mut t1x: isize;   let mut t2x: isize;
        let mut y: isize;
        let mut min_x: isize; let mut max_x: isize;
        let mut t1xp: isize;  let mut t2xp: isize;

        let mut changed1 = false;   let mut changed2 = false;

        #[allow(unused_mut)]
        let mut sign_x1:i8;

        #[allow(unused_mut)]
        let mut sign_x2:i8;

        let mut dx1: isize;   let mut dy1: isize;
        let mut dx2: isize;   let mut dy2: isize;

        let mut e1: isize; let mut e2: isize;

        // Sort points by their Y value (lowest value first).
        if point0.y > point1.y { (point0, point1) = (point1, point0) }
        if point0.y > point2.y { (point0, point2) = (point2, point0) }
        if point1.y > point2.y { (point1, point2) = (point2, point1) }

        // Starting points
        t1x = point0.x as isize; t2x = t1x;
        y = point0.y as isize;

        // Determine dx and dy for points 0 and 1
        dx1 = point1.x as isize - point0.x as isize;
        if dx1 < 0 {dx1 = -dx1; sign_x1 = -1; }
        else { sign_x1 = 1; }
        dy1 = point1.y as isize - point0.y as isize;

        // Determine dx and dy for points 0 and 2
        dx2 = point2.x as isize - point0.x as isize;
        if dx2 < 0 { dx2 = -dx2;    sign_x2 = -1; }
        else { sign_x2 = 1; }
        dy2 = point2.y as isize - point0.y as isize;


        if dy1 > dx1 {   // swap values
			(dx1, dy1) = (dy1, dx1);
			changed1 = true;
		}
		if dy2 > dx2 {   // swap values
			(dy2, dx2) = (dx2, dy2);
			changed2 = true;
		}

        e2 = dx2 >> 1;
        if point0.y != point1.y {
            // Not a flat top.
                // If there is a flat top, skip to 'next
            e1 = dx1 >> 1;

            let mut i = 0;
            'outer: while i < dx1 {
                t1xp = 0;   t2xp = 0;
                if t1x < t2x    { min_x = t1x;    max_x = t2x; }
                else { min_x = t2x;    max_x = t1x }
                

                // process first line until y value is about to change
                'start: while i < dx1 {
                    i += 1;
                    e1 += dy1;

                    while e1 >= dx1 {
                        e1 -= dx1;
                        if changed1 { t1xp = sign_x1 as isize; }
                        else { break 'start; } // goto next1
                    }

                    if changed1 { break; }
                    else { t1x += sign_x1 as isize; }
                }

                // process second line until y value is about to change
                'next1: loop {
                    e2 += dy2;

                    while e2 >= dx2 {
                        e2 -= dx2;
                        if changed2 { t2xp = sign_x2 as isize; }
                        else { break 'next1 } // goto next 2
                    }

                    if changed2 { break; }
                    else { t2x += sign_x2 as isize; }
                }

                '_next2: {
                    if min_x > t1x { min_x = t1x; } if min_x > t2x { min_x = t2x; }
                    if max_x < t1x { max_x = t1x; } if max_x < t2x { max_x = t2x }

                    //  Draw line from min x to max x on y
                    self.draw_line(
                        Point{x: min_x as usize, y: y as usize}, 
                        Point{x: max_x as usize, y: y as usize}, 
                        color
                    );

                    // Now increase y
                    if !changed1 { t1x += sign_x1 as isize; }
                    t1x += t1xp;

                    if !changed2 { t2x += sign_x2 as isize; }
                    t2x += t2xp;

                    y += 1;
                    if y == point1.y as isize { break 'outer; }
                }
            }
        }
        
        // else point0.y == point1.y
        '_next: {
            // Second half
            dx1 = point2.x as isize - point1.x as isize;
            if dx1 < 0 { dx1 = -dx1;    sign_x1 = -1; }
            else { sign_x1 = 1; }

            dy1 = point2.y as isize - point1.y as isize;
            t1x = point1.x as isize;


            if dy1 > dx1 {
                (dy1, dx1) = (dx1, dy1);
                changed1 = true;
            }
            else { changed1 = false; }

            e1 = dx1 >> 1;

            let mut i = 0;
            while i <= dx1 {
                t1xp = 0;   t2xp = 0;

                if t1x < t2x { min_x = t1x; max_x = t2x; }
                else { min_x = t2x; max_x = t1x; }

                // Process first line until y is about to change
                'first: while i < dx1 {
                    e1 += dy1;

                    while e1 >= dx1 {
                        e1 -= dx1;
                        if changed1 { t1xp = sign_x1 as isize; break;} 
                        else {  break 'first; } // goto next3
                    }
                    
                    if changed1 { break; }
                    else { t1x += sign_x1 as isize; }

                    if i < dx1 { i += 1};
                    
                }

                'next3: {
                    // process second line until y value is about to change
                    while t2x != point2.x as isize {
                        e2 += dy2;
                        while e2 >= dx2 {
                            e2 -= dx2;
                            if changed2 { t2xp = sign_x2 as isize; }
                            else { break 'next3; } // goto next4
                        }
                        if changed2 { break; }
                        else { t2x += sign_x2 as isize; }
                    }
                }

                '_next4: {
                    if min_x > t1x { min_x = t1x }  if min_x > t2x { min_x = t2x; }
                    if max_x < t1x { max_x = t1x }  if max_x < t2x { max_x = t2x; }

                    //  Draw line from min x to max x on y
                    self.draw_line(
                        Point{x: min_x as usize, y: y as usize}, 
                        Point{x: max_x as usize, y: y as usize}, 
                        color
                    );

                    if !changed1 { t1x += sign_x1 as isize; }
                    t1x += t1xp;

                    if !changed2 { t2x += sign_x2 as isize; }
                    t2x += t2xp;

                    y += 1;

                    if y > point2.y as isize { return; }
                }

                i += 1;
            }
        }
    }

    pub fn render(&mut self) -> Result<()> {     
        // Reset cursor position to top left instead of letting terminal scroll
        queue!(
            self.stdout,
            cursor::MoveTo(0, 0),
        )?;

        // Loop through screen buffer
        for y in ( 0..self.out_h).step_by(2) {
            for x in 0..self.out_w {
                let top = self.screen_buffer[x][y];

                let bottom = if y + 1 < self.out_h {
                    self.screen_buffer[x][y + 1]
                } else {
                    RGB::default()
                };

                let old_top = self.previous_buffer[x][y];
                let old_bottom = if y + 1 < self.out_h {
                    self.previous_buffer[x][y + 1]
                } else {
                    RGB::default()
                };

                // Nothing changed.
                if !self.first_frame && top == old_top && bottom == old_bottom {
                    continue;
                }

                // Add draw pixel to queue with color
                queue!(
                    self.stdout,
                    cursor::MoveTo(x as u16, (y / 2) as u16),
                    SetForegroundColor(bottom.to_crossterm()),
                    SetBackgroundColor(top.to_crossterm()),
                    Print(self.draw_pixel),
                    ResetColor,
                )?;
            }
        }

        // Clear color settings
        queue!(
            self.stdout,
            ResetColor,
        )?;
        // Flush queue to screen
        self.stdout.flush()?;

        // Swap the frame buffer at the end of each frame
        std::mem::swap(
            &mut self.screen_buffer,
            &mut self.previous_buffer,
        );

        self.first_frame = false;

        Ok(())
    }

}

pub fn ready_terminal(renderer: &mut Renderer) -> Result<()> {
    renderer.stdout.execute(EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    renderer.stdout.execute(cursor::Hide)?;
    Ok(())
}

pub fn restore_terminal(renderer: &mut Renderer) -> Result<()>  {
    renderer.stdout.execute(cursor::Show)?;
    terminal::disable_raw_mode()?;
    renderer.stdout.execute(LeaveAlternateScreen)?;
    Ok(())
}

pub fn run(renderer: &mut Renderer, input_handler: &mut InputHandler) -> Result<()> {
    let _ = ready_terminal(renderer);

    loop {
        let _ = input_handler.update();
        if input_handler.exit {break}


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

        renderer.render()?;

        thread::sleep(Duration::from_millis( (1000/renderer.target_framerate) as u64));
    }

    // Restore terminal state.
    let _ = restore_terminal(renderer);

    Ok(())
}
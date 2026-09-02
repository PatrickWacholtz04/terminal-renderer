const OUT_W:usize = 26;
const OUT_H:usize = 26;

#[derive(Default, Clone, Copy, Debug)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

struct Renderer {
    out_w: usize,
    out_h: usize,
    screen_buffer: Vec<Vec<RGB>>,
    draw_pixel: char,
}

impl Renderer {
    fn new() -> Self {
        // Create a screen buffer the size of the output containing an RGB value for each pixel
        Self {
            out_w: OUT_W,
            out_h: OUT_H,
            screen_buffer: vec![vec![RGB::default(); OUT_H]; OUT_W],
            draw_pixel: '▄',
        }
        
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: RGB) {
        self.screen_buffer[x][y] = color;
    }

    fn draw_line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: RGB) {
        // Generate line using Bresenham's Line Algorithm
            // https://www.youtube.com/watch?v=CceepU1vIKo

        let dx = x1 as isize - x0 as isize;
        let dy = y1 as isize - y0 as isize;

        if dx.abs() > dy.abs() {
            self.draw_line_horizontal(x0, y0, x1, y1, color);
        }
        else {
            self.draw_line_vertical(x0, y0, x1, y1, color);
        }
    }
    

    fn draw_line_horizontal(&mut self, mut x0: usize, mut y0: usize, mut x1: usize, mut y1: usize, color: RGB) {
        // Generate line using Bresenham's Line Algorithm
            // https://www.youtube.com/watch?v=CceepU1vIKo
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
                self.set_pixel(x0 + i as usize, y as usize, color);

                if p >= 0 {
                    y += dir;
                    p = p - 2*dx;
                }
                p = p + 2*dy;
            }
        }
    }

    fn draw_line_vertical(&mut self, mut x0: usize, mut y0: usize, mut x1: usize, mut y1: usize, color: RGB) {
        // Generate line using Bresenham's Line Algorithm
            // https://www.youtube.com/watch?v=CceepU1vIKo
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
                self.set_pixel(x as usize, y0 + i as usize, color);

                if p >= 0 {
                    x += dir;
                    p = p - 2*dy;
                }
                p = p + 2*dx;
            }
        }
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


fn main() {
    let mut renderer = Renderer::new();

    renderer.set_pixel(13, 13, RGB{r: 255, g: 0, b: 0});

    renderer.draw_line(13,13, 25, 18, RGB{r: 255, g: 0, b: 0});
    renderer.draw_line(13,13, 18, 25, RGB{r: 0, g: 255, b: 0});

    renderer.draw_line(13,13, 9, 25, RGB{r: 0, g: 0, b: 255});
    renderer.draw_line(13,13, 0, 18, RGB{r: 255, g: 0, b: 0});

    renderer.draw_line(13,13, 25, 9, RGB{r: 0, g: 0, b: 255});
    renderer.draw_line(13,13, 18, 0, RGB{r: 0, g: 255, b: 0});

    renderer.draw_line(13,13, 0, 9, RGB{r: 0, g: 255, b: 0});
    renderer.draw_line(13,13, 9, 0, RGB{r: 255, g: 0, b: 0});
   
    
    renderer.render();
}

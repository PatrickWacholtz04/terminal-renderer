use std::io::{BufRead, BufReader, Result};
use std::{thread, time::Duration, time::Instant};
use std::fs::File;

use crate::renderer_2d::{
    Point,
    RGB
};

mod renderer_2d;

#[derive(Default, Clone, Copy)]
struct Vec3d {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Default, Clone, Copy)]
struct Triangle {
    p: [Vec3d; 3],
    color: RGB,
}

#[derive(Default, Clone)]
struct Mesh {
    tris: Vec<Triangle>,
}

#[derive(Default, Clone, Copy)]
struct Mat4x4 {
    m: [[f32; 4]; 4],
}

struct Renderer {
    mesh_cube: Mesh,
    mat_proj: Mat4x4,
    v_camera: Vec3d,
}

impl Mesh {
    fn load_from_obj(&mut self, file_name: &str) -> Result<()> {
        let file = File::open(file_name)?;
        let reader = BufReader::new(file);

        let color = RGB {r: 255, g: 0, b: 255};

        let mut verts: Vec<Vec3d> = Vec::new();
        for line in reader.lines() {
            let line = line?;
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            
            if parts[0] == "v" {
                let x: f32 = parts[1].parse().unwrap();
                let y: f32 = parts[2].parse().unwrap();
                let z: f32 = parts[3].parse().unwrap();
                verts.push(
                    Vec3d { x, y, z }
                );
            }

            else if parts[0] == "f" {
                let p0: usize = parts[1].parse().unwrap();
                let p1: usize = parts[2].parse().unwrap();
                let p2: usize = parts[3].parse().unwrap();

                let p = [verts[p0 - 1], verts[p1 - 1], verts[p2 - 1]];
                // let color = RGB::default();

                self.tris.push(
                    Triangle { p, color }
                );
            }

        }

        Ok(())
    }


}

impl Renderer {
    fn new() -> Self {
        let mesh_cube = Mesh::default();
        
        let mat_proj = Mat4x4::default();

        Self { 
            mesh_cube,
            mat_proj,
            v_camera: Vec3d::default(),
        }

    }

    fn multiply_matrix_vector(i: Vec3d, m: Mat4x4) -> Vec3d {
        let mut o = Vec3d::default();

        o.x = i.x * m.m[0][0] + i.y * m.m[1][0] + i.z * m.m[2][0] + m.m[3][0];
        o.y = i.x * m.m[0][1] + i.y * m.m[1][1] + i.z * m.m[2][1] + m.m[3][1];
        o.z = i.x * m.m[0][2] + i.y * m.m[1][2] + i.z * m.m[2][2] + m.m[3][2];
        let w = i.x * m.m[0][3] + i.y * m.m[1][3] + i.z * m.m[2][3] + m.m[3][3];

        if w != 0.0 {
            o.x /= w;
            o.y /= w;
            o.z /= w;
        }
        return o;
    }

    fn get_color(input_color: RGB, mut factor: f32) -> RGB {
        let num_shades = 4.0;
        let level = (factor * num_shades).round() / num_shades;
        factor = level * 0.8;

        let r_new = input_color.r as f32 + (255.0 - input_color.r as f32) * factor;
        let g_new = input_color.g as f32 + (255.0 - input_color.g as f32) * factor;
        let b_new = input_color.b as f32 + (255.0 - input_color.b as f32) * factor;

        return RGB { r: r_new as u8, g: g_new as u8, b: b_new as u8 };
    }



}




fn main() -> Result<()> {
    let mut renderer_2d = renderer_2d::Renderer::new();
    let mut input_handler = renderer_2d::InputHandler::new();
    

    let mut renderer = Renderer::new();

    // Load mesh from obj file
    renderer.mesh_cube.load_from_obj("src/VideoShip.obj")?;
    

    // Projection Matrix
    let f_near = 0.1f32;
    let f_far = 1000.0f32;
    let f_fov = 90.0f32;
    let f_aspect_ratio = renderer_2d.out_h as f32 / renderer_2d.out_w as f32;
    let f_fov_rad = 1.0f32 / (f_fov * 0.5 / 180.0 * 3.14159).tan();

    renderer.mat_proj.m[0][0] = f_aspect_ratio * f_fov_rad;
    renderer.mat_proj.m[1][1] = f_fov_rad;
    renderer.mat_proj.m[2][2] = f_far / (f_far - f_near);
    renderer.mat_proj.m[3][2] = (-f_far * f_near) / (f_far - f_near);
    renderer.mat_proj.m[2][3] = 1.0f32;
    renderer.mat_proj.m[3][3] = 0.0f32;


    let _ = renderer_2d::ready_terminal(&mut renderer_2d);

    let start_time = Instant::now();
    let target_frame_time = Duration::from_secs_f32(1.0 / renderer_2d.target_framerate as f32);

    loop {
        let frame_start = Instant::now();
        let elapsed_time = frame_start - start_time;

        let _ = input_handler.update();
        if input_handler.exit {
            break;
        }
        if input_handler.terminal_resized {
            renderer_2d.first_frame = true;
        }

        // Clear the logical framebuffer.
        renderer_2d.screen_buffer.fill(
            vec![RGB::default(); renderer_2d.out_h]
        );
        

        // Draw Logic
        let mut mat_rot_z = Mat4x4::default();
        let mut mat_rot_x = Mat4x4::default();

        let f_theta = elapsed_time.as_secs_f32();

        // Rotation Z
        mat_rot_z.m[0][0] = f_theta.cos();
        mat_rot_z.m[0][1] = f_theta.sin();
        mat_rot_z.m[1][0] = -f_theta.sin();
        mat_rot_z.m[1][1] = f_theta.cos();
        mat_rot_z.m[2][2] = 1.0;
        mat_rot_z.m[3][3] = 1.0;

        // Rotation X
        mat_rot_x.m[0][0] = 1.0;
        mat_rot_x.m[1][1] = (0.5 * f_theta).cos();
        mat_rot_x.m[1][2] = (0.5 * f_theta).sin();
        mat_rot_x.m[2][1] = -(0.5 * f_theta).sin();
        mat_rot_x.m[2][2] = (0.5 * f_theta).cos();
        mat_rot_x.m[3][3] = 1.0;

        for tri in &renderer.mesh_cube.tris {
            let mut projected = Triangle::default();
            let mut rotated_z = Triangle::default();
            let mut rotated_zx = Triangle::default();

            // Rotate along Z axis
            rotated_z.p[0] = Renderer::multiply_matrix_vector(tri.p[0], mat_rot_z);
            rotated_z.p[1] = Renderer::multiply_matrix_vector(tri.p[1], mat_rot_z);
            rotated_z.p[2] = Renderer::multiply_matrix_vector(tri.p[2], mat_rot_z);

            // Rotate along X Axis
            rotated_zx.p[0] = Renderer::multiply_matrix_vector(rotated_z.p[0], mat_rot_x);
            rotated_zx.p[1] = Renderer::multiply_matrix_vector(rotated_z.p[1], mat_rot_x);
            rotated_zx.p[2] = Renderer::multiply_matrix_vector(rotated_z.p[2], mat_rot_x);

            // Offset onto screen
            let mut translated = rotated_zx;
            translated.p[0].z = rotated_zx.p[0].z + 8.0;
            translated.p[1].z = rotated_zx.p[1].z + 8.0;
            translated.p[2].z = rotated_zx.p[2].z + 8.0;

            let mut normal = Vec3d::default();
            let mut line1 = Vec3d::default();
            let mut line2 = Vec3d::default();

            line1.x = translated.p[1].x - translated.p[0].x;
            line1.y = translated.p[1].y - translated.p[0].y;
            line1.z = translated.p[1].z - translated.p[0].z;

            line2.x = translated.p[2].x - translated.p[0].x;
            line2.y = translated.p[2].y - translated.p[0].y;
            line2.z = translated.p[2].z - translated.p[0].z;

            normal.x = line1.y * line2.z - line1.z * line2.y;
            normal.y = line1.z * line2.x - line1.x * line2.z;
            normal.z = line1.x * line2.y - line1.y * line2.x;

            let l = (normal.x*normal.x + normal.y*normal.y + normal.z*normal.z).sqrt();
            normal.x /= l; normal.y /= l; normal.z /= l;
            

            if (normal.x * translated.p[0].x - renderer.v_camera.x) +
                (normal.y * translated.p[0].y - renderer.v_camera.y) +
                (normal.z * translated.p[0].z - renderer.v_camera.z) < 0.0 {

                // Illumination
                let mut light_direction = Vec3d{x: 0.0, y: 0.0, z: -1.0};
                let l = (light_direction.x *light_direction.x + light_direction.y*light_direction.y + light_direction.z*light_direction.z).sqrt();
                light_direction.x /= l;  light_direction.y /= l;    light_direction.z /= l;

                let dp = normal.x*light_direction.x + normal.y*light_direction.y + normal.z*light_direction.z;
            
                translated.color = Renderer::get_color(tri.color, dp);

                projected.p[0] = Renderer::multiply_matrix_vector(translated.p[0], renderer.mat_proj);
                projected.p[1] = Renderer::multiply_matrix_vector(translated.p[1], renderer.mat_proj);
                projected.p[2] = Renderer::multiply_matrix_vector(translated.p[2], renderer.mat_proj);
                projected.color = translated.color;

                // Scale into view
                projected.p[0].x += 1.0;    projected.p[0].y += 1.0;
                projected.p[1].x += 1.0;    projected.p[1].y += 1.0;
                projected.p[2].x += 1.0;    projected.p[2].y += 1.0;

                projected.p[0].x *= 0.5 * renderer_2d.out_w as f32;
                projected.p[0].y *= 0.5 * renderer_2d.out_h as f32;
                projected.p[1].x *= 0.5 * renderer_2d.out_w as f32;
                projected.p[1].y *= 0.5 * renderer_2d.out_h as f32;
                projected.p[2].x *= 0.5 * renderer_2d.out_w as f32;
                projected.p[2].y *= 0.5 * renderer_2d.out_h as f32;


                // Rasterize triangle
                
                renderer_2d.fill_triangle(Point{x: projected.p[0].x as usize, y: projected.p[0].y as usize}, 
                                        Point{x: projected.p[1].x as usize, y: projected.p[1].y as usize},
                                        Point{x: projected.p[2].x as usize, y: projected.p[2].y as usize},
                                        projected.color
                );

                // renderer_2d.draw_triangle(Point{x: projected.p[0].x as usize, y: projected.p[0].y as usize}, 
                //                         Point{x: projected.p[1].x as usize, y: projected.p[1].y as usize},
                //                         Point{x: projected.p[2].x as usize, y: projected.p[2].y as usize},
                //                         RGB { r: 255, g: 255, b: 255 }
                // );



            }   

        }

        renderer_2d.render()?;

        let frame_time = frame_start.elapsed();
        if frame_time < target_frame_time {
            thread::sleep(target_frame_time - frame_time);
        }

    }


    let result = renderer_2d::run(&mut renderer_2d, &mut input_handler);
    
    let _ = renderer_2d::restore_terminal(&mut renderer_2d);

    result
}
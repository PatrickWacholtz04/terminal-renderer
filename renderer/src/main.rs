use std::env;
use std::io::{BufRead, BufReader, Result};
use std::{thread, time::Duration, time::Instant};
use std::fs::File;

use crossterm::event::KeyCode;

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

impl Vec3d {
    fn add(v1: Vec3d, v2: Vec3d) -> Vec3d {
        return Vec3d {
            x: v1.x + v2.x,
            y: v1.y + v2.y,
            z: v1.z + v2.z
        }
    }

    fn sub(v1: Vec3d, v2: Vec3d) -> Vec3d {
        return Vec3d {
            x: v1.x - v2.x,
            y: v1.y - v2.y,
            z: v1.z - v2.z
        }
    }

    fn mult(v: Vec3d, k: f32) -> Vec3d {
        return Vec3d {
            x: v.x * k,
            y: v.y * k,
            z: v.z * k
        }
    }

    fn div(v: Vec3d, k: f32) -> Vec3d {
        return Vec3d {
            x: v.x / k,
            y: v.y / k,
            z: v.z / k
        }
    }

    fn dot_product(v1: Vec3d, v2: Vec3d) -> f32 {
        return v1.x * v2.x + v1.y * v2.y + v1.z * v2.z;
    }

    fn length(v: Vec3d) -> f32 {
        return Vec3d::dot_product(v, v).sqrt();

    }

    fn normal(v: Vec3d) -> Vec3d {
        let l = Vec3d::length(v);
        return Vec3d {
            x: v.x / l,
            y: v.y / l,
            z: v.z / l
        }
    }

    fn cross_product(v1: Vec3d, v2: Vec3d) -> Vec3d {
        return Vec3d {
            x: v1.y * v2.z - v1.z * v2.y,
            y: v1.z * v2.x - v1.x * v2.z,
            z: v1.x * v2.y - v1.y * v2.x
        } 
    }

}

impl Mat4x4 {
    fn make_identity() -> Mat4x4 {
        let mut identity = Mat4x4::default();
        identity.m[0][0] = 1.0;
        identity.m[1][1] = 1.0;
        identity.m[2][2] = 1.0;
        identity.m[3][3] = 1.0;
        return identity;
    }

    fn make_rotation_x(f_theta: f32) -> Mat4x4 {
        let mut mat_rot_x = Mat4x4::default();

        mat_rot_x.m[0][0] = 1.0;
        mat_rot_x.m[1][1] = (0.5 * f_theta).cos();
        mat_rot_x.m[1][2] = (0.5 * f_theta).sin();
        mat_rot_x.m[2][1] = -(0.5 * f_theta).sin();
        mat_rot_x.m[2][2] = (0.5 * f_theta).cos();
        mat_rot_x.m[3][3] = 1.0;

        return mat_rot_x;
    }

    fn make_rotation_y(f_theta: f32) -> Mat4x4 {
        let mut mat_rot_y = Mat4x4::default();

        mat_rot_y.m[0][0] = f_theta.cos();
        mat_rot_y.m[0][2] = f_theta.sin();
        mat_rot_y.m[2][0] = -f_theta.sin();
        mat_rot_y.m[1][1] = 1.0;
        mat_rot_y.m[2][2] = f_theta.cos();
        mat_rot_y.m[3][3] = 1.0;

        return mat_rot_y;
    }

    fn make_rotation_z(f_theta: f32) -> Mat4x4 {
        let mut mat_rot_z = Mat4x4::default();

        mat_rot_z.m[0][0] = f_theta.cos();
        mat_rot_z.m[0][1] = f_theta.sin();
        mat_rot_z.m[1][0] = -f_theta.sin();
        mat_rot_z.m[1][1] = f_theta.cos();
        mat_rot_z.m[2][2] = 1.0;
        mat_rot_z.m[3][3] = 1.0;

        return mat_rot_z
    }

    fn make_translation(x: f32, y: f32, z: f32) -> Mat4x4 {
        let mut translation = Mat4x4::default();

        translation.m[0][0] = 1.0;
        translation.m[1][1] = 1.0;
        translation.m[2][2] = 1.0;
        translation.m[3][3] = 1.0;
        translation.m[3][0] = x;
        translation.m[3][1] = y;
        translation.m[3][2] = z;

        return translation;
    }

    fn make_projection(f_fov: f32, f_aspect_ratio: f32, f_far: f32, f_near: f32) -> Mat4x4 {
        let f_fov_rad = 1.0 / (f_fov * 0.5 / 180.0 * 3.14159).tan();
        let mut projected = Mat4x4::default();

        projected.m[0][0] = f_aspect_ratio * f_fov_rad;
        projected.m[1][1] = f_fov_rad;
        projected.m[2][2] = f_far / (f_far - f_near);
        projected.m[3][2] = (-f_far * f_near) / (f_far - f_near);
        projected.m[2][3] = 1.0;
        projected.m[3][3] = 0.0;

        return projected;
    }


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
            if parts.len() == 0 { continue; }
            
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

    fn multiply_matrix_matrix(m1: Mat4x4, m2: Mat4x4) -> Mat4x4 {
        let mut multiplied = Mat4x4::default();

        for c in 0..4 {
            for r in 0..4 {
                multiplied.m[r][c] = m1.m[r][0] * m2.m[0][c] + m1.m[r][1] * m2.m[1][c] + m1.m[r][2] * m2.m[2][c] + m1.m[r][3] * m2.m[3][c];
            }
        }

        return multiplied;
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
    let binding = &"src/VideoShip.obj".to_string();

    let args: Vec<String> = env::args().collect();
    let file_path: &String = args
        .get(1)
        .unwrap_or(binding);
    
    renderer.mesh_cube.load_from_obj(file_path)?;
    

    // Projection Matrix
    let f_near: f32 = 0.1;
    let f_far: f32 = 1000.0;
    let f_fov: f32 = 90.0;
    let f_aspect_ratio: f32 = renderer_2d.out_h as f32 / renderer_2d.out_w as f32;

    renderer.mat_proj = Mat4x4::make_projection(f_fov, f_aspect_ratio, f_far, f_near);


    let _ = renderer_2d::ready_terminal(&mut renderer_2d);

    let start_time = Instant::now();
    let target_frame_time = Duration::from_secs_f32(1.0 / renderer_2d.target_framerate as f32);

    let mut rot_x = 0.0;    let mut rot_y = 0.0;    let mut rot_z = 0.0;
    let mut trans_z: f32= 5.0;

    loop {
        let frame_start = Instant::now();
        let elapsed_time = frame_start - start_time;

        let mut rot_x_input = 0.0;  let mut rot_y_input = 0.0;  let mut rot_z_input = 0.0;
        let mut trans_z_input: f32 = 0.0;

        if let Some(key) = input_handler.update()? {
            match key {
                KeyCode::Char('w') => {
                    rot_x_input = 1.0;
                }
                KeyCode::Char('s') => {
                    rot_x_input = -1.0;
                }
                KeyCode::Char('d') => {
                    rot_y_input = 1.0;
                }
                KeyCode::Char('a') => {
                    rot_y_input = -1.0;
                }
                KeyCode::Char('e') => {
                    rot_z_input = 1.0;
                }
                KeyCode::Char('q') => {
                    rot_z_input = -1.0;
                }
                KeyCode::Char('r') => {
                    trans_z_input = 1.0;
                }
                KeyCode::Char('f') => {
                    trans_z_input = -1.0;
                }
                _ => {}
            }
        }



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

        let f_theta = elapsed_time.as_secs_f32();

        // Rotation matrices
        let rotation_speed = 0.02;
        rot_x += rot_x_input * f_theta * rotation_speed;
        rot_y += rot_y_input * f_theta * rotation_speed;
        rot_z += rot_z_input * f_theta * rotation_speed;

        let mat_rot_x = Mat4x4::make_rotation_x(rot_x);
        let mat_rot_y = Mat4x4::make_rotation_y(rot_y);
        let mat_rot_z = Mat4x4::make_rotation_z(rot_z);
     
        // Translation matrix
        let translation_speed = 0.02;
        trans_z += trans_z_input * f_theta * translation_speed;
        let translation = Mat4x4::make_translation(0.0, 0.0, trans_z);

        let mut world = Mat4x4::make_identity();
        world = Renderer::multiply_matrix_matrix(mat_rot_x, mat_rot_y);
        world = Renderer::multiply_matrix_matrix(world, mat_rot_z);
        world = Renderer::multiply_matrix_matrix(world, translation);

        let mut triangles_to_raster:Vec<Triangle> = Vec::new();

        for tri in &renderer.mesh_cube.tris {
            let mut projected = Triangle::default();
            let mut transformed = Triangle::default();

            transformed.p[0] = Renderer::multiply_matrix_vector(tri.p[0], world);
            transformed.p[1] = Renderer::multiply_matrix_vector(tri.p[1], world);
            transformed.p[2] = Renderer::multiply_matrix_vector(tri.p[2], world);

            // Calculate triangle normal
            let mut normal = Vec3d::default();
            let mut line1 = Vec3d::default();
            let mut line2 = Vec3d::default();

            line1 = Vec3d::sub(transformed.p[1], transformed.p[0]);
            line2 = Vec3d::sub(transformed.p[2], transformed.p[0]);

            normal = Vec3d::cross_product(line1, line2);
            normal = Vec3d::normal(normal);

            let v_camera_ray = Vec3d::sub(transformed.p[0], renderer.v_camera);
            

            if Vec3d::dot_product(normal, v_camera_ray) < 0.0 {

                // Illumination
                let mut light_direction = Vec3d{x: 0.0, y: 0.0, z: -1.0};
                light_direction = Vec3d::normal(light_direction);

                let dp = normal.x*light_direction.x + normal.y*light_direction.y + normal.z*light_direction.z;
            
                transformed.color = Renderer::get_color(tri.color, dp);

                projected.p[0] = Renderer::multiply_matrix_vector(transformed.p[0], renderer.mat_proj);
                projected.p[1] = Renderer::multiply_matrix_vector(transformed.p[1], renderer.mat_proj);
                projected.p[2] = Renderer::multiply_matrix_vector(transformed.p[2], renderer.mat_proj);
                projected.color = transformed.color;

                // Scale into view
                let v_offset_view = Vec3d { x: 1.0, y: 1.0, z: 0.0 };
                projected.p[0] = Vec3d::add(projected.p[0], v_offset_view);
                projected.p[1] = Vec3d::add(projected.p[1], v_offset_view);
                projected.p[2] = Vec3d::add(projected.p[2], v_offset_view);

                projected.p[0].x *= 0.5 * renderer_2d.out_w as f32;
                projected.p[0].y *= 0.5 * renderer_2d.out_h as f32;
                projected.p[1].x *= 0.5 * renderer_2d.out_w as f32;
                projected.p[1].y *= 0.5 * renderer_2d.out_h as f32;
                projected.p[2].x *= 0.5 * renderer_2d.out_w as f32;
                projected.p[2].y *= 0.5 * renderer_2d.out_h as f32;


                // Store triangles for sorting
                triangles_to_raster.push(projected);
                
                // renderer_2d.fill_triangle(Point{x: projected.p[0].x as usize, y: projected.p[0].y as usize}, 
                //                         Point{x: projected.p[1].x as usize, y: projected.p[1].y as usize},
                //                         Point{x: projected.p[2].x as usize, y: projected.p[2].y as usize},
                //                         projected.color
                // );

                // renderer_2d.draw_triangle(Point{x: projected.p[0].x as usize, y: projected.p[0].y as usize}, 
                //                         Point{x: projected.p[1].x as usize, y: projected.p[1].y as usize},
                //                         Point{x: projected.p[2].x as usize, y: projected.p[2].y as usize},
                //                         RGB { r: 255, g: 255, b: 255 }
                // );



            }   
        }

        // Sort triangles before drawing
        triangles_to_raster.sort_by(|t1, t2| {
            let z1 = (t1.p[0].z + t1.p[1].z + t1.p[2].z) / 3.0;
            let z2 = (t2.p[0].z + t2.p[1].z + t2.p[2].z) / 3.0;

            z2.partial_cmp(&z1).unwrap()
        });


        // Draw triangles to screen
        for tri in triangles_to_raster {
            renderer_2d.fill_triangle(Point{x: tri.p[0].x as usize, y: tri.p[0].y as usize}, 
                                    Point{x: tri.p[1].x as usize, y: tri.p[1].y as usize},
                                    Point{x: tri.p[2].x as usize, y: tri.p[2].y as usize},
                                    tri.color
            );
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
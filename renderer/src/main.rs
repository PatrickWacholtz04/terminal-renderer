use std::io::Result;
use std::{thread, time::Duration, time::Instant};

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
}

impl Renderer {
    fn new() -> Self {
        let mesh_cube = Mesh {
            tris: vec![
                // Front face
                Triangle {
                    p: [
                        Vec3d { x: 0.0, y: 0.0, z: 0.0 },
                        Vec3d { x: 0.0, y: 1.0, z: 0.0 },
                        Vec3d { x: 1.0, y: 1.0, z: 0.0 },
                    ],
                },
                Triangle {
                    p: [
                        Vec3d { x: 0.0, y: 0.0, z: 0.0 },
                        Vec3d { x: 1.0, y: 1.0, z: 0.0 },
                        Vec3d { x: 1.0, y: 0.0, z: 0.0 },
                    ],
                },

                // Back face
                Triangle {
                    p: [
                        Vec3d { x: 0.0, y: 0.0, z: 1.0 },
                        Vec3d { x: 1.0, y: 0.0, z: 1.0 },
                        Vec3d { x: 1.0, y: 1.0, z: 1.0 },
                    ],
                },
                Triangle {
                    p: [
                        Vec3d { x: 0.0, y: 0.0, z: 1.0 },
                        Vec3d { x: 1.0, y: 1.0, z: 1.0 },
                        Vec3d { x: 0.0, y: 1.0, z: 1.0 },
                    ],
                },

                // Left face
                Triangle {
                    p: [
                        Vec3d { x: 0.0, y: 0.0, z: 0.0 },
                        Vec3d { x: 0.0, y: 0.0, z: 1.0 },
                        Vec3d { x: 0.0, y: 1.0, z: 1.0 },
                    ],
                },
                Triangle {
                    p: [
                        Vec3d { x: 0.0, y: 0.0, z: 0.0 },
                        Vec3d { x: 0.0, y: 1.0, z: 1.0 },
                        Vec3d { x: 0.0, y: 1.0, z: 0.0 },
                    ],
                },

                // Right face
                Triangle {
                    p: [
                        Vec3d { x: 1.0, y: 0.0, z: 0.0 },
                        Vec3d { x: 1.0, y: 1.0, z: 0.0 },
                        Vec3d { x: 1.0, y: 1.0, z: 1.0 },
                    ],
                },
                Triangle {
                    p: [
                        Vec3d { x: 1.0, y: 0.0, z: 0.0 },
                        Vec3d { x: 1.0, y: 1.0, z: 1.0 },
                        Vec3d { x: 1.0, y: 0.0, z: 1.0 },
                    ],
                },

                // Top face
                Triangle {
                    p: [
                        Vec3d { x: 0.0, y: 1.0, z: 0.0 },
                        Vec3d { x: 0.0, y: 1.0, z: 1.0 },
                        Vec3d { x: 1.0, y: 1.0, z: 1.0 },
                    ],
                },
                Triangle {
                    p: [
                        Vec3d { x: 0.0, y: 1.0, z: 0.0 },
                        Vec3d { x: 1.0, y: 1.0, z: 1.0 },
                        Vec3d { x: 1.0, y: 1.0, z: 0.0 },
                    ],
                },

                // Bottom face
                Triangle {
                    p: [
                        Vec3d { x: 0.0, y: 0.0, z: 0.0 },
                        Vec3d { x: 1.0, y: 0.0, z: 0.0 },
                        Vec3d { x: 1.0, y: 0.0, z: 1.0 },
                    ],
                },
                Triangle {
                    p: [
                        Vec3d { x: 0.0, y: 0.0, z: 0.0 },
                        Vec3d { x: 1.0, y: 0.0, z: 1.0 },
                        Vec3d { x: 0.0, y: 0.0, z: 1.0 },
                    ],
                },
            ],
        };

        let mat_proj = Mat4x4::default();

        Self { 
            mesh_cube,
            mat_proj,
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

}




fn main() -> Result<()> {
    let mut renderer_2d = renderer_2d::Renderer::new();
    let mut input_handler = renderer_2d::InputHandler::new();
    

    let mut renderer = Renderer::new();

    // Projection Matrix
    let fNear = 0.1f32;
    let fFar = 1000.0f32;
    let fFov = 90.0f32;
    let fAspectRatio = renderer_2d.out_h as f32 / renderer_2d.out_w as f32;
    let fFovRad = 1.0f32 / (fFov * 0.5 / 180.0 * 3.14159).tan();

    renderer.mat_proj.m[0][0] = fAspectRatio * fFovRad;
    renderer.mat_proj.m[1][1] = fFovRad;
    renderer.mat_proj.m[2][2] = fFar / (fFar - fNear);
    renderer.mat_proj.m[3][2] = (-fFar * fNear) / (fFar - fNear);
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


            rotated_z.p[0] = Renderer::multiply_matrix_vector(tri.p[0], mat_rot_z);
            rotated_z.p[1] = Renderer::multiply_matrix_vector(tri.p[1], mat_rot_z);
            rotated_z.p[2] = Renderer::multiply_matrix_vector(tri.p[2], mat_rot_z);

            rotated_zx.p[0] = Renderer::multiply_matrix_vector(rotated_z.p[0], mat_rot_x);
            rotated_zx.p[1] = Renderer::multiply_matrix_vector(rotated_z.p[1], mat_rot_x);
            rotated_zx.p[2] = Renderer::multiply_matrix_vector(rotated_z.p[2], mat_rot_x);

            let mut translated = rotated_zx;
            translated.p[0].z = rotated_zx.p[0].z + 3.0;
            translated.p[1].z = rotated_zx.p[1].z + 3.0;
            translated.p[2].z = rotated_zx.p[2].z + 3.0;
            
            projected.p[0] = Renderer::multiply_matrix_vector(translated.p[0], renderer.mat_proj);
            projected.p[1] = Renderer::multiply_matrix_vector(translated.p[1], renderer.mat_proj);
            projected.p[2] = Renderer::multiply_matrix_vector(translated.p[2], renderer.mat_proj);

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

            renderer_2d.draw_triangle(Point{x: projected.p[0].x as usize, y: projected.p[0].y as usize}, 
                                    Point{x: projected.p[1].x as usize, y: projected.p[1].y as usize},
                                    Point{x: projected.p[2].x as usize, y: projected.p[2].y as usize},
                                    RGB{r: 255, g: 255, b:255}
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
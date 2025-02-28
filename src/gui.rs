use std::sync::{Arc, RwLock};
use gl::COLOR_BUFFER_BIT;
use glfw::{fail_on_errors, Context, WindowMode};
use crate::Status;

pub fn run_gui(display_state: Arc<RwLock<[[usize;64];32]>>, pressed_key: Arc<RwLock<[bool;16]>>, status: Arc<RwLock<Status>>) {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    let (mut window, events) = glfw.create_window(1280, 640, "A Rusty Chip8 Emulator", WindowMode::Windowed).unwrap();

    window.make_current();
    window.set_key_polling(true);

    gl::load_with(|ptr|window.get_proc_address(&ptr));
    let shader_program = unsafe {create_shader_program()};
    let mut vao = 0;
    let mut vbo = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::ClearColor(0.5, 0.5, 0.75, 1.0);
    }

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Space, _a, glfw::Action::Press, _c) => {
                    toggle_pause(&status);
                }
                glfw::WindowEvent::Key(key, _a, action, _c) => {
                    update_pressed_key(key, action, pressed_key.clone());
                }
                _ => {}
            }
        }
        unsafe {
            gl::Viewport(0, 0, window.get_size().0, window.get_size().1);
            gl::Clear(COLOR_BUFFER_BIT);
            draw_screen(display_state.clone(), vao);
            gl::UseProgram(shader_program);
        }
        window.swap_buffers();
    }

    unsafe {gl::DeleteProgram(shader_program)}
}
fn toggle_pause(status: &Arc<RwLock<Status>>) {
    let mut write_status = status.write().unwrap();
    match *write_status {
        Status::Paused => {
            *write_status = Status::Running;
            unsafe {gl::ClearColor(0.5, 0.5, 0.75, 1.0);}
        }
        Status::Running => {
            *write_status = Status::Paused;
            unsafe {gl::ClearColor(0.75, 0.5, 0.5, 1.0);}
        }
        _ => {}
    }
}
fn update_pressed_key(key: glfw::Key, action: glfw::Action, lock: Arc<RwLock<[bool;16]>>) {
    match key {
        glfw::Key::Num1 => {map_key_press(1, action, lock);}
        glfw::Key::Num2 => {map_key_press(2, action, lock);}
        glfw::Key::Num3 => {map_key_press(3, action, lock);}
        glfw::Key::Num4 => {map_key_press(0xC, action, lock);}

        glfw::Key::Q => {map_key_press(4, action, lock);}
        glfw::Key::W => {map_key_press(5, action, lock);}
        glfw::Key::E => {map_key_press(6, action, lock);}
        glfw::Key::R => {map_key_press(0xD, action, lock);}

        glfw::Key::A => {map_key_press(7, action, lock);}
        glfw::Key::S => {map_key_press(8, action, lock);}
        glfw::Key::D => {map_key_press(9, action, lock);}
        glfw::Key::F => {map_key_press(0xE, action, lock);}

        glfw::Key::Z => {map_key_press(0xA, action, lock);}
        glfw::Key::X => {map_key_press(0, action, lock);}
        glfw::Key::C => {map_key_press(0xB, action, lock);}
        glfw::Key::V => {map_key_press(0xF, action, lock);}

        _=>{}
    }
}

fn map_key_press(value: usize, action: glfw::Action, lock: Arc<RwLock<[bool;16]>>) {
    match action {
        glfw::Action::Press => {
            let mut write_lock = lock.write().unwrap();
            write_lock[value] = true;
            // println!("Wrote key press for value: {}", value);
        }
        glfw::Action::Release => {
            let mut write_lock = lock.write().unwrap();
            write_lock[value] = false;
        }
        _ => {}
    }
}

fn draw_screen(display_state: Arc<RwLock<[[usize;64];32]>>, vao: u32) {
    let (display_vector, vertex_count) = convert_state_to_vertices(display_state);
    if vertex_count > 0 {
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(&*display_vector) as isize,
                display_vector.as_ptr().cast(),
                gl::STATIC_DRAW);


            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 8, 0 as *const _);
            gl::EnableVertexAttribArray(0);

            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, vertex_count);
            gl::DisableVertexAttribArray(0);
        }
    }
}

fn convert_state_to_vertices(display_state: Arc<RwLock<[[usize;64];32]>>) -> (Vec<f32>, i32) {
    let mut vertex_count = 0;
    //1. convert the display_state into an array of vertices and colors
    //2. The max size the display_vector can have is: (64x32)x6 = 12_288
    let mut display_vector = Vec::with_capacity(12_288);
    //The display vector looks like this:
    // [color, v1.x, v1.y, v2.x, v2.y, ..., vn.x, nv.y]
    let read_display = display_state.read().unwrap();
    for row in 0.. read_display.len() {
        for col in 0.. read_display[row].len() {
            if read_display[row][col] > 0 {
                vertex_count += 6;

                let top_left = (-1.0 + col as f32/32.0, 1.0 - row as f32/16.0);
                let bottom_left = (-1.0 + col as f32/32.0, 1.0 - (row+1) as f32/16.0);
                let bottom_right = (-1.0 + (col+1) as f32/32.0, 1.0 - (row+1) as f32/16.0);
                let top_right = (-1.0 + (col+1) as f32/32.0, 1.0 - row as f32/16.0);

                display_vector.push(bottom_left.0);
                display_vector.push(bottom_left.1);
                display_vector.push(bottom_right.0);
                display_vector.push(bottom_right.1);
                display_vector.push(top_left.0);
                display_vector.push(top_left.1);
                display_vector.push(bottom_right.0);
                display_vector.push(bottom_right.1);
                display_vector.push(top_right.0);
                display_vector.push(top_right.1);
                display_vector.push(top_left.0);
                display_vector.push(top_left.1);
            }
        }
    }
    (display_vector, vertex_count)
}

unsafe fn create_shader_program() -> u32 {
    //1. create vertex shader
    let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
    let vertex_source = VERTEX_SHADER_SOURCE.to_string();
    gl::ShaderSource(vertex_shader, 1, &vertex_source.as_bytes().as_ptr().cast(), &vertex_source.len().try_into().unwrap());
    gl::CompileShader(vertex_shader);
    //2. create fragment shader
    let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
    let fragment_source = FRAGMENT_SHADER_SOURCE.to_string();
    gl::ShaderSource(fragment_shader, 1, &fragment_source.as_bytes().as_ptr().cast(), &fragment_source.len().try_into().unwrap());
    gl::CompileShader(fragment_shader);

    //3. link the shaders into program
    let shader_program = gl::CreateProgram();
    gl::AttachShader(shader_program, vertex_shader);
    gl::AttachShader(shader_program, fragment_shader);
    gl::LinkProgram(shader_program);
    gl::DeleteShader(vertex_shader);
    gl::DeleteShader(fragment_shader);
    shader_program
}

const VERTEX_SHADER_SOURCE: &str = "\
#version 330 core
layout (location=0) in vec2 vertexPosition;

out vec3 fragmentColor;

void main() {
    gl_Position = vec4(vertexPosition, 1.0, 1.0);
    fragmentColor = (1.0, 1.0, 1.0);
}
";

const FRAGMENT_SHADER_SOURCE: &str = "\
#version 330 core
in vec3 fragmentColor;
out vec4 screenColor;

void main() {
screenColor = vec4(fragmentColor, 1.0);
}
";
extern crate osvr;
extern crate gl;
use std::f32;
use std::io::prelude::*;
use std::fs::File;
use std::ffi::CString;

mod gl1x;

#[derive(Debug)]
enum SpecType { O, B, A, F, G, K, M }

impl std::fmt::Display for SpecType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct Star {
    hd_num: i64,
    ra_hours: i8,
    ra_mins: i8,
    ra_seconds: f32,
    dec_deg: i8,
    dec_mins: i8,
    dec_seconds: i8,
    visual_mag: f32,
    spec_type: SpecType
}

impl Star {
    fn equatorial_position_as_radians(&self) -> (f32, f32) {
        (f32::consts::PI * 2.0 * ((self.ra_hours as f32 / 24.0f32) +
                                  (self.ra_mins as f32 / (60.0*24.0)) +
                                  (self.ra_seconds / (3600.0*24.0))),
         f32::consts::PI * 0.5 * ((self.dec_deg as f32 / 90.0) +
                                  (self.dec_mins as f32 / (90.0*60.0)) +
                                  (self.dec_seconds as f32 / (90.0*3600.0)))
        )
    }
}

fn load_catalogue() -> Vec<Star> {
    let mut f = File::open("bsc5.dat").expect("bsc5.dat (yale bright star catalogue) not found");

    let mut contents = String::new();
    f.read_to_string(&mut contents).expect("Could not read from bsc5.dat");

    fn parse_catalogue_line(line: &str) -> Option<Star> {
        // ignore if there is no position
        let dec_sign: i8 = if line.get(83..84).unwrap() == "+" { 1 } else { -1 };
        let _ra_hours = line.get(75..77).unwrap().parse::<i8>();
        let _vis_mag = line.get(103..107).unwrap().parse::<f32>();

        if _ra_hours.is_ok() && _vis_mag.is_ok() {
            Some(Star {
                hd_num: line.get(25..31).unwrap().parse::<i64>().unwrap_or(0),
                ra_hours: _ra_hours.unwrap(),
                ra_mins: line.get(77..79).unwrap().parse::<i8>().unwrap(),
                ra_seconds: line.get(79..83).unwrap().parse::<f32>().unwrap(),
                dec_deg: line.get(84..86).unwrap().parse::<i8>().unwrap() * dec_sign,
                dec_mins: line.get(86..88).unwrap().parse::<i8>().unwrap() * dec_sign,
                dec_seconds: line.get(88..90).unwrap().parse::<i8>().unwrap() * dec_sign,
                visual_mag: _vis_mag.unwrap(),
                spec_type: match line.get(129..130).unwrap() {
                    "O" => SpecType::O,
                    "B" => SpecType::B,
                    "A" => SpecType::A,
                    "F" => SpecType::F,
                    "G" => SpecType::G,
                    "K" => SpecType::K,
                    "M" => SpecType::M,
                    /* not really correct */
                    "S" => SpecType::M,
                    "N" => SpecType::M,
                    "C" => SpecType::M,
                    "W" => SpecType::O,
                    "p" => SpecType::O, /* eta carinae */
                    _ => panic!("Unexpected spectral type in bsc5.dat")
                }
            })
        } else {
            None
        }
    }

    contents.lines().filter_map(parse_catalogue_line).collect::<Vec<Star>>()
}

mod glazy {
    use gl;
    use std;
    use std::ffi::CString;

    pub struct Shader(u32);

    impl Shader {
        pub fn id(&self) -> u32 {
            let Shader(id) = *self;
            id
        }

        pub unsafe fn new(vert_src: &str, frag_src: &str) -> Shader {
            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vertex_shader, 1, &CString::new(vert_src).unwrap().as_ptr(), std::ptr::null());
            gl::ShaderSource(fragment_shader, 1, &CString::new(frag_src).unwrap().as_ptr(), std::ptr::null());
            gl::CompileShader(vertex_shader);
            gl::CompileShader(fragment_shader);

            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::LinkProgram(shader_program);
            Shader(shader_program)
        }

        pub unsafe fn getUniformLocation(&self, name: &str) -> i32 {
            gl::GetUniformLocation(self.id(), CString::new(name).unwrap().as_ptr())
        }
    }
}

fn main() {
    let context = osvr::Context::new("Rust OSVR example");
    let mut render = osvr::RenderManager::new(&context).unwrap();
    gl1x::init();
    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
        gl::Enable(gl::PROGRAM_POINT_SIZE);
        gl::BlendFunc(gl::ONE, gl::ONE);
    };
    context.update();
    render.register_buffers();

    let catalogue = load_catalogue();

    let (star_vbo, shader_program, proj_matrix, view_matrix) = {

        let mut star_data: Vec<f32> = Vec::new();

        for s in &catalogue {
            let (ra, dec) = s.equatorial_position_as_radians();

            star_data.push(-10.0 * ra.sin() * dec.cos());
            star_data.push( 10.0 * dec.sin());
            star_data.push(-10.0 * ra.cos() * dec.cos());

            let a = ((6.0 - s.visual_mag)/7.0).max(0.0);

            for color_component in match s.spec_type {
                // [r, g, b]
                SpecType::O => [a, a, a],
                SpecType::B => [a, a, a],
                SpecType::A => [a, a, a],
                SpecType::F => [a, a, a],
                SpecType::G => [a, a, a*0.9],
                SpecType::K => [a, a*0.9, a*0.75],
                SpecType::M => [a, a*0.75, a*0.5],
            }.iter() {
                star_data.push(*color_component)
            }
        }

        unsafe {
            let mut star_vbo = 0;

            gl::GenBuffers(1, &mut star_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, star_vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (star_data.len()*4) as isize, std::mem::transmute(&star_data[0]), gl::STATIC_DRAW);
            gl::BindBuffer(gl::ARRAY_BUFFER, star_vbo);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 6*4, std::mem::transmute(0u64));
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 6*4, std::mem::transmute(12u64));
            gl::EnableVertexAttribArray(1);
            gl::BindBuffer(gl::ARRAY_BUFFER, star_vbo);

            let shader = glazy::Shader::new(
                r#"
                #version 130 // Specify which version of GLSL we are using.

                uniform mat4 view_matrix, proj_matrix;
                in vec3 in_Position;
                in vec3 in_Color;
                varying vec4 color;

                void main() 
                {
                    gl_PointSize = max(1.0, in_Color.r * 3.0);
                    color = vec4(in_Color.r, in_Color.g, in_Color.b, 1.0);
                    gl_Position = proj_matrix * view_matrix * vec4(in_Position.x, in_Position.y, in_Position.z, 1.0);
                }
            "#, r#"
                #version 130 // Specify which version of GLSL we are using.
                varying vec4 color;

                void main() 
                {
                    gl_FragColor = color;
                }
            "#);
            gl::BindAttribLocation(shader.id(), 0, CString::new("in_Position").unwrap().as_ptr());
            gl::BindAttribLocation(shader.id(), 1, CString::new("in_Color").unwrap().as_ptr());
            let proj_matrix = shader.getUniformLocation("proj_matrix");
            let view_matrix = shader.getUniformLocation("view_matrix");

            let e = gl::GetError();
            if e != 0 { panic!("GL ERROR {}", e) };
            
            (star_vbo, shader.id(), proj_matrix, view_matrix)
        }
    };

    loop {
        context.update();
        render.render_eyes(|render_info, frame_buffer, color_buffer, depth_buffer| {
            osvr::glutil::bind_buffers(frame_buffer, color_buffer, depth_buffer);
            osvr::glutil::set_viewport(render_info);

            let projection = osvr::glutil::get_projection(render_info);
            let modelview = osvr::glutil::get_modelview(render_info);

            unsafe {
                let mut _projection: [f32; 16] = std::mem::zeroed();
                let mut _modelview: [f32; 16] = std::mem::zeroed();
                for i in 0..16 {
                    _projection[i] = projection[i] as f32;
                    _modelview[i] = modelview[i] as f32;
                }

                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                // draw stars
                gl::EnableVertexAttribArray(0);
                gl::EnableVertexAttribArray(1);
                gl::BindBuffer(gl::ARRAY_BUFFER, star_vbo);
                gl::UseProgram(shader_program);
                gl::UniformMatrix4fv(proj_matrix, 1, 0, &_projection[0]);
                gl::UniformMatrix4fv(view_matrix, 1, 0, &_modelview[0]);
                gl::DrawArrays(gl1x::POINTS, 0, catalogue.len() as i32);
            }
        });
    }
}

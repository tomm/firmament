extern crate osvr;
extern crate gl;
use std::f32;
use std::io::prelude::*;
use std::fs::File;

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

fn main() {
    let catalogue = load_catalogue();

    let context = osvr::Context::new("Rust OSVR example");
    let mut render = osvr::RenderManager::new(&context).unwrap();
    gl1x::init();
    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
        gl::Enable(gl1x::POINT_SMOOTH);
        gl::BlendFunc(gl::ONE, gl::ONE);
    };
    context.update();
    render.register_buffers();

    loop {
        context.update();
        render.render_eyes(|render_info, frame_buffer, color_buffer, depth_buffer| {
            osvr::glutil::bind_buffers(frame_buffer, color_buffer, depth_buffer);
            osvr::glutil::set_viewport(render_info);

            let projection = osvr::glutil::get_projection(render_info);
            let modelview = osvr::glutil::get_modelview(render_info);

            unsafe {
                gl1x::MatrixMode(gl1x::PROJECTION);
                gl1x::LoadIdentity();
                gl1x::MultMatrixd(&projection);

                gl1x::MatrixMode(gl1x::MODELVIEW);
                gl1x::LoadIdentity();
                gl1x::MultMatrixd(&modelview);

                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                draw_stars(&catalogue);
            }
        });
    }
}

unsafe fn draw_stars(stars: &Vec<Star>) {
    gl1x::PointSize(2.0);
    gl1x::Begin(gl1x::POINTS);
    
    for s in stars {
        let r: f32 = 10.0;
        let (ra, dec) = s.equatorial_position_as_radians();
        let brightness = ((6.0 - s.visual_mag)/7.0).min(1.0).max(0.0);
        let col: [f32; 3] = [brightness, brightness, brightness];
        gl1x::Color3fv(&col);
        gl1x::Vertex3f(-r * ra.sin() * dec.cos(),
                       r * dec.sin(),
                       -r * ra.cos() * dec.cos());

    }

    gl1x::End();
}

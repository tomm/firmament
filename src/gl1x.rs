// wrap some GL 1.x stuff that this demo needs, but gl-rs doesn't wrap
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
extern crate osvr;
use std;

pub const PROJECTION: u32 = 0x1701;
pub const MODELVIEW: u32 = 0x1700;
pub const POLYGON: u32 = 0x0009;
pub const POINTS: u32 = 0x0000;
pub const POINT_SMOOTH: u32 = 0x0B10;

pub fn init() {
    unsafe {
        glBegin = Some(std::mem::transmute(osvr::glutil::get_proc_address("glBegin")));
        glEnd = Some(std::mem::transmute(osvr::glutil::get_proc_address("glEnd")));
        glPushMatrix = Some(std::mem::transmute(osvr::glutil::get_proc_address("glPushMatrix")));
        glPopMatrix = Some(std::mem::transmute(osvr::glutil::get_proc_address("glPopMatrix")));
        glLoadIdentity = Some(std::mem::transmute(osvr::glutil::get_proc_address("glLoadIdentity")));
        glMultMatrixd = Some(std::mem::transmute(osvr::glutil::get_proc_address("glMultMatrixd")));
        glMatrixMode = Some(std::mem::transmute(osvr::glutil::get_proc_address("glMatrixMode")));
        glVertex3f = Some(std::mem::transmute(osvr::glutil::get_proc_address("glVertex3f")));
        glNormal3f = Some(std::mem::transmute(osvr::glutil::get_proc_address("glNormal3f")));
        glColor3fv = Some(std::mem::transmute(osvr::glutil::get_proc_address("glColor3fv")));
        glScaled = Some(std::mem::transmute(osvr::glutil::get_proc_address("glScaled")));
        glPointSize = Some(std::mem::transmute(osvr::glutil::get_proc_address("glPointSize")));
    }
}

pub fn LoadIdentity() { unsafe { (glLoadIdentity.unwrap())() } }
pub fn Begin(mode: u32) { unsafe { (glBegin.unwrap())(mode) } }
pub fn End() { unsafe { (glEnd.unwrap())() } }
pub fn PushMatrix() { unsafe { (glPushMatrix.unwrap())() } }
pub fn PopMatrix() { unsafe { (glPopMatrix.unwrap())() } }
pub fn MultMatrixd(matrix: &[f64]) {
    assert!(matrix.len() == 16);
    unsafe { (glMultMatrixd.unwrap())(&matrix[0] as *const f64) }
}
pub fn MatrixMode(mode: u32) { unsafe { (glMatrixMode.unwrap())(mode) } }
pub fn PointSize(size: f32) { unsafe { (glPointSize.unwrap())(size) } }
pub fn Vertex3f(x: f32, y: f32, z: f32) { unsafe { (glVertex3f.unwrap())(x, y, z) } }
pub fn Normal3f(x: f32, y: f32, z: f32) { unsafe { (glNormal3f.unwrap())(x, y, z) } }
pub fn Color3fv(color: &[f32]) {
    assert!(color.len() == 3);
    unsafe { (glColor3fv.unwrap())(&color[0] as *const f32) }
}
pub fn Scaled(x: f64, y: f64, z: f64) { unsafe { (glScaled.unwrap())(x, y, z) } }

static mut glBegin: Option<extern "C" fn(mode: u32)> = None;
static mut glEnd: Option<extern "C" fn()> = None;
static mut glPushMatrix: Option<extern "C" fn()> = None;
static mut glPopMatrix: Option<extern "C" fn()> = None;
static mut glLoadIdentity: Option<extern "C" fn()> = None;
static mut glMultMatrixd: Option<extern "C" fn(matrix: *const f64)> = None;
static mut glMatrixMode: Option<extern "C" fn(mode: u32)> = None;
static mut glVertex3f: Option<extern "C" fn(x: f32, y: f32, z: f32)> = None;
static mut glNormal3f: Option<extern "C" fn(x: f32, y: f32, z: f32)> = None;
static mut glColor3fv: Option<extern "C" fn(color: *const f32)> = None;
static mut glScaled: Option<extern "C" fn(x: f64, y: f64, z: f64)> = None;
static mut glPointSize: Option<extern "C" fn(s: f32)> = None;

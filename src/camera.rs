use cgmath::{Angle, InnerSpace, Matrix4, Point3, Rad, Vector3};

use crate::{HEIGHT, WIDTH};

pub struct Camera {
    pos: Point3<f32>,
    dir: Vector3<f32>,
    up: Vector3<f32>,

    move_speed: f32,
    look_sensitivity: f64,

    current_x: f64,
    current_y: f64,
    azimut: f64,
    zenit: f64,

    changed: bool,

    view_matrix: Matrix4<f32>,
}

impl Camera {
    pub fn new(pos: Point3<f32>, move_speed: f32, look_sensitivity: f64) -> Self {
        Self {
            pos,
            dir: Vector3::new(0., 0., -1.),
            up: Vector3::new(0., 1., 0.),
            move_speed,
            look_sensitivity,
            current_x: WIDTH as f64 / 2.,
            current_y: HEIGHT as f64 / 2.,
            azimut: 0.,
            zenit: 0.,
            changed: true,
            view_matrix: Matrix4::from_translation(Vector3::new(0., 0., 0.)),
        }
    }

    pub fn get_view_mat(&mut self) -> Matrix4<f32> {
        if self.changed {
            self.changed = false;
            self.view_matrix = Matrix4::look_to_rh(self.pos, self.dir, self.up);
        }

        self.view_matrix
    }

    pub fn move_forward(&mut self, d: f32) {
        self.pos += self.dir * d * self.move_speed;
        self.changed = true;
    }

    pub fn move_backward(&mut self, d: f32) {
        self.move_forward(-d);
    }

    pub fn strafe_right(&mut self, d: f32) {
        let dir = self.dir.cross(self.up).normalize();
        self.pos += dir * d * self.move_speed;
        self.changed = true;
    }

    pub fn strafe_left(&mut self, d: f32) {
        self.strafe_right(-d);
    }

    pub fn adjust_look(&mut self, new_x: f64, new_y: f64) {
        let dx = self.current_x - new_x;
        let dy = self.current_y - new_y;

        let x_offset = dx * self.look_sensitivity;
        let y_offset = dy * self.look_sensitivity;

        self.azimut -= x_offset;
        self.zenit += y_offset;

        self.adjust_dir();
    }

    fn adjust_dir(&mut self) {
        let rad_azimut = Rad(270. + self.azimut);
        let rad_zenit = Rad(self.zenit);

        let x = Angle::cos(rad_azimut) * Angle::cos(rad_zenit);
        let y = Angle::sin(rad_zenit);
        let z = Angle::sin(rad_azimut) * Angle::cos(rad_zenit);

        self.dir = Vector3::new(x as f32, y as f32, z as f32);
        self.changed = true;
    }
}

use bytemuck::{Pod, Zeroable};
use cgmath::{Angle, Basis3, InnerSpace, Rad, Rotation, Rotation3, Vector3};

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    position: Vector3<f32>,
    forwards: Vector3<f32>,
    focus_dist: f32,
    defocus_angle: f32,
}

impl Camera {
    pub fn new(
        position: [f32; 3],
        forwards: [f32; 3],
        focus_dist: f32,
        defocus_angle: f32,
    ) -> Camera {
        Camera {
            position: position.into(),
            forwards: forwards.into(),
            focus_dist,
            defocus_angle,
        }
    }

    pub fn into_uniform(&self) -> CameraUniform {
        let true_up = Vector3::unit_z();
        let right = self.forwards.cross(true_up).normalize();
        let up = right.cross(self.forwards).normalize();

        CameraUniform::new(
            self.position.into(),
            self.forwards.into(),
            right.into(),
            up.into(),
            self.focus_dist,
            self.defocus_angle,
        )
    }

    pub fn forwards(&mut self) {
        self.position += 0.1 * self.forwards;
    }

    pub fn backwards(&mut self) {
        self.position -= 0.1 * self.forwards;
    }

    pub fn rightwards(&mut self) {
        let true_up = Vector3::unit_z();
        let right = self.forwards.cross(true_up).normalize();
        self.position += 0.1 * right;
    }

    pub fn leftwards(&mut self) {
        let true_up = Vector3::unit_z();
        let right = self.forwards.cross(true_up).normalize();
        self.position -= 0.1 * right;
    }

    pub fn rotate_rightwards(&mut self, speed: f32) {
        let angle: Rad<f32> = Rad::full_turn() * speed;
        self.forwards = Basis3::from_angle_z(-angle).rotate_vector(self.forwards);
    }

    pub fn rotate_upwards(&mut self, speed: f32) {
        let angle: Rad<f32> = Rad::full_turn() * speed;
        let axis = self.forwards.cross(Vector3::unit_z()).normalize();
        self.forwards = Basis3::from_axis_angle(axis, angle).rotate_vector(self.forwards);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    position: [f32; 3],
    focus_dist: f32,
    forwards: [f32; 3],
    defocus_angle: f32,
    right: [f32; 3],
    _right_padding: f32,
    up: [f32; 3],
    _up_padding: f32,
}

impl Default for Camera {
    fn default() -> Camera {
        Camera::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 5.0, 0.3)
    }
}

impl CameraUniform {
    pub const fn new(
        position: [f32; 3],
        forwards: [f32; 3],
        right: [f32; 3],
        up: [f32; 3],
        focus_dist: f32,
        defocus_angle: f32,
    ) -> CameraUniform {
        CameraUniform {
            position,
            forwards,
            right,
            up,
            focus_dist,
            defocus_angle,

            _right_padding: 0.0,
            _up_padding: 0.0,
        }
    }
}

use glam::{Mat4, Quat, Vec2, Vec3, Vec4, Vec4Swizzles};

pub trait Camera3D: Sync + Send {
    fn update(&mut self, ui: &egui::Ui, response: Option<&egui::Response>, delta: f32);

    /// Calculate the view matrix
    fn calculate_matrix(&mut self) -> Mat4;

    fn zoom(&self) -> f32;

    fn position(&mut self) -> Vec3 {
        (self.calculate_matrix() * Vec4::ONE).xyz()
    }

    fn rotation(&self) -> Quat;

    fn focus_on_point(&mut self, point: Vec3, dist_scale: f32);

    fn focus_offset(&mut self, _dist_scale: f32) -> Vec3 {
        Vec3::ZERO
    }
}

fn zoom_factor(zoom_level: f32) -> f32 {
    2.0f32.powf(zoom_level * std::f32::consts::LN_2) - 0.9
}

#[derive(Clone)]
pub struct ArcBallCamera {
    pivot: Vec3,
    orientation: Vec2,
    zoom: f32,
    log_zoom: bool,

    camera_inv: Mat4,
}

impl ArcBallCamera {
    pub fn new(pivot: Vec3, orientation: Vec2, zoom: f32, log_zoom: bool) -> Self {
        ArcBallCamera {
            pivot,
            orientation,
            zoom,
            log_zoom,
            camera_inv: Mat4::IDENTITY,
        }
    }

    // fn eye(&self) -> Vec3 {
    //     (self.camera_inv * Vec4::W).xyz()
    // }

    fn dir(&self) -> Vec3 {
        (self.camera_inv * Vec4::Z).xyz().normalize()
    }

    fn up(&self) -> Vec3 {
        (self.camera_inv * Vec4::Y).xyz().normalize()
    }

    fn right(&self) -> Vec3 {
        self.dir().cross(self.up())
    }
}

impl Default for ArcBallCamera {
    fn default() -> Self {
        ArcBallCamera {
            pivot: Vec3::ZERO,
            orientation: Vec2::new(30., 140.),
            zoom: 5.0,
            log_zoom: true,
            camera_inv: Mat4::IDENTITY,
        }
    }
}

impl Camera3D for ArcBallCamera {
    fn update(&mut self, ui: &egui::Ui, response: Option<&egui::Response>, _delta: f32) {
        if let Some(multi_touch) = ui.ctx().multi_touch() {
            self.zoom += -(multi_touch.zoom_delta - 1.0);
        } else if let Some(response) = &response {
            if response.dragged_by(egui::PointerButton::Primary)
                || response.dragged_by(egui::PointerButton::Middle)
            {
                let mouse_delta = response.drag_delta();
                self.orientation += Vec2::new(mouse_delta.y * 0.8, mouse_delta.x) * 0.15;
            }
        }

        if let Some(response) = &response {
            if response.hover_pos().is_some() {
                self.zoom += -ui.input(|i| i.scroll_delta).y * 0.005;
            }

            if response.dragged_by(egui::PointerButton::Secondary) {
                self.pivot += (-self.up() * response.drag_delta().y * 0.003) * self.zoom();
                self.pivot += (self.right() * response.drag_delta().x * 0.003) * self.zoom();
            }
        }

        // Front view
        if ui.input(|i| i.key_pressed(egui::Key::Num1)) {
            self.orientation = Vec2::new(0., 0.);
        }

        // Right view
        if ui.input(|i| i.key_pressed(egui::Key::Num3)) {
            self.orientation = Vec2::new(0., 90.);
        }

        // Top view
        if ui.input(|i| i.key_pressed(egui::Key::Num7)) {
            self.orientation = Vec2::new(90., 0.);
        }

        // Rotate right
        if ui.input(|i| i.key_pressed(egui::Key::Num6)) {
            self.orientation.y -= 90. / 6.;
        }

        // Rotate left
        if ui.input(|i| i.key_pressed(egui::Key::Num4)) {
            self.orientation.y += 90. / 6.;
        }

        // Rotate up
        if ui.input(|i| i.key_pressed(egui::Key::Num8)) {
            self.orientation.x += 90. / 6.;
        }

        // Rotate down
        if ui.input(|i| i.key_pressed(egui::Key::Num2)) {
            self.orientation.x -= 90. / 6.;
        }

        self.zoom = self.zoom.clamp(0.00, 250.0);
        self.orientation.x = self.orientation.x.clamp(-90., 90.);
    }

    fn calculate_matrix(&mut self) -> Mat4 {
        let rotation = Mat4::from_quat(
            glam::Quat::from_rotation_x((self.orientation.x).to_radians())
                * glam::Quat::from_rotation_y((-self.orientation.y).to_radians()),
        );

        let translation = Mat4::from_translation(self.pivot);
        let zoom = Mat4::from_translation(glam::vec3(
            0.0,
            0.0,
            if self.log_zoom {
                -zoom_factor(self.zoom)
            } else {
                -self.zoom
            },
        ));

        let view = zoom * rotation * translation;
        self.camera_inv = view.inverse();

        view
    }

    fn rotation(&self) -> Quat {
        // FIXME(cohae): This still isn't right, billboards are flipped horizontally
        glam::Quat::from_rotation_y(self.orientation.y.to_radians())
            * glam::Quat::from_rotation_x(-self.orientation.x.to_radians())
    }

    fn zoom(&self) -> f32 {
        if self.log_zoom {
            zoom_factor(self.zoom)
        } else {
            self.zoom
        }
    }

    fn focus_on_point(&mut self, point: Vec3, _dist_scale: f32) {
        self.pivot = point;
        self.pivot.z = -self.pivot.z;
        self.pivot.y = -self.pivot.y;
    }
}

#[derive(Clone)]
pub struct FpsCamera {
    orientation: Vec2,
    pub front: Vec3,
    pub right: Vec3,
    pub position: Vec3,
    pub speed_mul: f32,
}

impl Default for FpsCamera {
    fn default() -> Self {
        Self {
            front: Vec3::Y,
            right: Vec3::Z,
            position: Vec3::ZERO,
            orientation: Vec2::ZERO,
            speed_mul: 1.0,
        }
    }
}

impl FpsCamera {
    fn update_vectors(&mut self) {
        let mut front = Vec3::ZERO;
        front.x = self.orientation.x.to_radians().cos() * self.orientation.y.to_radians().sin();
        front.y = -self.orientation.x.to_radians().sin();
        front.z = self.orientation.x.to_radians().cos() * self.orientation.y.to_radians().cos();

        self.front = front.normalize();
        self.right = -self.front.cross(Vec3::Y).normalize();
    }
}

impl Camera3D for FpsCamera {
    fn update(&mut self, ui: &egui::Ui, response: Option<&egui::Response>, delta: f32) {
        if let Some(response) = response {
            if response.hover_pos().is_some() {
                let scroll = ui.input(|i| i.scroll_delta);
                self.speed_mul = (self.speed_mul + scroll.y * 0.005).clamp(0.0, 5.0);
            }

            let mouse_delta = response.drag_delta();
            self.orientation += Vec2::new(mouse_delta.y * 0.8, mouse_delta.x) * 0.15;
        }

        let mut speed = delta * zoom_factor(self.speed_mul) * 10.0;
        if ui.input(|i| i.modifiers.shift) {
            speed *= 2.0;
        }
        if ui.input(|i| i.modifiers.ctrl) {
            speed /= 2.0;
        }

        let mut direction = Vec3::ZERO;
        if ui.input(|i| i.key_down(egui::Key::W)) {
            direction += self.front;
        }
        if ui.input(|i| i.key_down(egui::Key::S)) {
            direction -= self.front;
        }

        if ui.input(|i| i.key_down(egui::Key::D)) {
            direction += self.right;
        }
        if ui.input(|i| i.key_down(egui::Key::A)) {
            direction -= self.right;
        }

        if ui.input(|i| i.key_down(egui::Key::Q)) {
            direction -= Vec3::Y;
        }
        if ui.input(|i| i.key_down(egui::Key::E)) {
            direction += Vec3::Y;
        }

        self.position += direction * speed;

        self.orientation.x = self.orientation.x.clamp(-89.9, 89.9);

        self.update_vectors();
    }

    fn calculate_matrix(&mut self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.front, Vec3::Y)
    }

    fn rotation(&self) -> Quat {
        glam::Quat::from_rotation_y(self.orientation.y.to_radians())
            * glam::Quat::from_rotation_x(self.orientation.x.to_radians())
    }

    // Abusing this to get the speed value for the status bar
    fn zoom(&self) -> f32 {
        self.speed_mul
    }

    fn position(&mut self) -> Vec3 {
        self.position
    }

    fn focus_on_point(&mut self, point: Vec3, dist_scale: f32) {
        self.position = point;
        // self.position.x = -self.position.x;
        self.position -= self.front * (2.5 * dist_scale)
    }

    fn focus_offset(&mut self, dist_scale: f32) -> Vec3 {
        self.front * (2.5 * dist_scale)
    }
}

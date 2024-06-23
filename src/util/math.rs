#[derive(Clone, Copy, PartialEq)]
pub struct Rect {
    pub top_left: glam::Vec2,
    pub size: glam::Vec2
}

impl From<(f32, f32, f32, f32)> for Rect {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Self::new_vals(value.0, value.1, value.2, value.3)
    }
}

impl From<[f32;4]> for Rect {
    fn from(value: [f32;4]) -> Self {
        Self::new_vals(value[0], value[1], value[2], value[3])
    }
}

impl Into<[f32; 4]> for Rect {
    fn into(self) -> [f32; 4] {
        [self.x(), self.y(), self.w(), self.h()]
    }
}

impl Rect {
    pub fn new(top_left: glam::Vec2, size: glam::Vec2) -> Self {
        Self { top_left, size }
    }

    pub fn new_vals(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { top_left: glam::Vec2::new(x, y), size: glam::Vec2::new(w, h) }
    }

    #[inline(always)]
    pub fn x(&self) -> f32 {
        self.top_left.x
    }

    #[inline(always)]
    pub fn y(&self) -> f32 {
        self.top_left.y
    }

    #[inline(always)]
    pub fn w(&self) -> f32 {
        self.size.x
    }

    #[inline(always)]
    pub fn h(&self) -> f32 {
        self.size.y
    }

    pub fn center(&self) -> glam::Vec2 {
        glam::Vec2::new(self.x()+self.w()/2.0, self.y()+self.h()/2.0)
    }
}
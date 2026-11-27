use serde::{Deserialize, Serialize};

use crate::utils::{merge_optional_values, tween_float, tween_float_vec2};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct ImageSchema {
    pub path: String,
    pub transform: ImageTransformSchema,
}

impl Default for ImageSchema {
    fn default() -> Self {
        Self {
            path: String::new(),
            transform: ImageTransformSchema::default(),
        }
    }
}

impl ImageSchema {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            transform: ImageTransformSchema::default(),
        }
    }

    pub fn with_transform(self, transform: ImageTransformSchema) -> Self {
        Self { transform, ..self }
    }

    pub fn position(self, pos: [f32; 2]) -> Self {
        Self {
            transform: self.transform.position(pos),
            ..self
        }
    }

    pub fn scale(self, scale: f32) -> Self {
        Self {
            transform: self.transform.scale(scale),
            ..self
        }
    }

    pub fn scale_xy(self, scale: [f32; 2]) -> Self {
        Self {
            transform: self.transform.scale_xy(scale),
            ..self
        }
    }

    // TODO: Other transforms
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct ImageTransformSchema {
    pub position: Option<[f32; 2]>,
    /// Uniform scale factor, negative values flip
    pub scale: Option<[f32; 2]>,
    /// Rotation in radians
    pub rotation: Option<f32>,
    /// Pivot point for rotation
    pub pivot: Option<[f32; 2]>,
    // TODO: Add tinting
    pub opacity: Option<f32>,
    /// Ordering to render images in (higher values are rendered on top)
    pub z_index: Option<i32>,
}

// Alias ImageTransformSchema::default() as transform()
pub fn transform() -> ImageTransformSchema {
    ImageTransformSchema::default()
}

impl Default for ImageTransformSchema {
    fn default() -> Self {
        Self {
            position: None,
            scale: None,
            rotation: None,
            pivot: None,
            opacity: None,
            z_index: None,
        }
    }
}

impl ImageTransformSchema {
    pub fn merge(&self, other: &ImageTransformSchema) -> Self {
        // Add positions
        let position = merge_optional_values(other.position, self.position, |a, b| {
            [a[0] + b[0], a[1] + b[1]]
        });
        // Multiply scales
        let scale =
            merge_optional_values(other.scale, self.scale, |a, b| [a[0] * b[0], a[1] * b[1]]);
        // Add rotations
        let rotation = merge_optional_values(other.rotation, self.rotation, |a, b| a + b);
        // TODO: Combine pivot
        // Multiply opacities
        let opacity = merge_optional_values(other.opacity, self.opacity, |a, _| a);

        Self {
            position,
            scale,
            rotation,
            pivot: other.pivot.or(self.pivot),
            opacity,
            z_index: other.z_index.or(self.z_index), // Override
        }
    }

    /// Interpolates the transform between two transforms.
    /// t is a value between 0 and 1.
    pub fn tween(&self, other: &ImageTransformSchema, t: f32) -> Self {
        Self {
            position: tween_float_vec2(self.position, other.position, t),
            scale: tween_float_vec2(self.scale, other.scale, t),
            rotation: tween_float(self.rotation, other.rotation, t),
            pivot: tween_float_vec2(self.pivot, other.pivot, t),
            opacity: tween_float(self.opacity, other.opacity, t),
            z_index: tween_float(
                self.z_index.map(|z| z as f32),
                other.z_index.map(|z| z as f32),
                t,
            )
            .map(|z| z as i32),
        }
    }

    // TODO: Add preset locations? Relative positioning?!!
    // pub fn at()

    pub fn position(self, pos: [f32; 2]) -> Self {
        Self {
            position: Some(pos),
            ..self
        }
    }

    pub fn scale(self, scale: f32) -> Self {
        Self {
            scale: Some([scale, scale]),
            ..self
        }
    }

    pub fn scale_xy(self, scale: [f32; 2]) -> Self {
        Self {
            scale: Some(scale),
            ..self
        }
    }

    pub fn rotation(self, rotation: f32) -> Self {
        Self {
            rotation: Some(rotation),
            ..self
        }
    }

    pub fn pivot(self, pivot: [f32; 2]) -> Self {
        Self {
            pivot: Some(pivot),
            ..self
        }
    }

    pub fn opacity(self, opacity: f32) -> Self {
        Self {
            opacity: Some(opacity),
            ..self
        }
    }

    pub fn z_index(self, z_index: i32) -> Self {
        Self {
            z_index: Some(z_index),
            ..self
        }
    }
}

pub fn position(pos: [f32; 2]) -> ImageTransformSchema {
    ImageTransformSchema::default().position(pos)
}

pub fn scale(scale: f32) -> ImageTransformSchema {
    ImageTransformSchema::default().scale(scale)
}

pub fn scale_xy(scale: [f32; 2]) -> ImageTransformSchema {
    ImageTransformSchema::default().scale_xy(scale)
}

pub fn rotation(rotation: f32) -> ImageTransformSchema {
    ImageTransformSchema::default().rotation(rotation)
}

pub fn pivot(pivot: [f32; 2]) -> ImageTransformSchema {
    ImageTransformSchema::default().pivot(pivot)
}

pub fn opacity(opacity: f32) -> ImageTransformSchema {
    ImageTransformSchema::default().opacity(opacity)
}

pub fn z_index(z_index: i32) -> ImageTransformSchema {
    ImageTransformSchema::default().z_index(z_index)
}

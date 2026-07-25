use egui::{Ui, Vec2, emath::TSTransform};
use rand::seq::SliceRandom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EffectType {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
    FadeIn,
    ZoomOut,
    ZoomIn,
    Bounce,
    Pop,
    Corner,
}

impl EffectType {
    pub const ALL: [EffectType; 10] = [
        EffectType::LeftToRight,
        EffectType::RightToLeft,
        EffectType::TopToBottom,
        EffectType::BottomToTop,
        EffectType::FadeIn,
        EffectType::ZoomOut,
        EffectType::ZoomIn,
        EffectType::Bounce,
        EffectType::Pop,
        EffectType::Corner,
    ];

    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        *Self::ALL.choose(&mut rng).unwrap()
    }
}

/// Which axis a slide effect moves along, and in which direction the
/// content enters from.
#[derive(Clone, Copy)]
enum SlideFrom {
    Left,
    Right,
    Top,
    Bottom,
}

/// Eases towards 1.0 but overshoots slightly past it before settling back,
/// like a spring. Input/output both range roughly over [0.0, 1.0].
fn ease_out_back(t: f32) -> f32 {
    const OVERSHOOT: f32 = 1.70158;
    let t = t - 1.0;
    1.0 + (OVERSHOOT + 1.0) * t.powi(3) + OVERSHOOT * t.powi(2)
}

pub struct Transition {
    is_ongoing: bool,
    progress: f32, // 0.0 (start) to 100.0 (finished)
    speed: f32,    // progress units per second
    effect_type: EffectType,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            is_ongoing: false,
            progress: 0.0,
            speed: 240.0,
            effect_type: EffectType::LeftToRight,
        }
    }
}

impl Transition {
    pub fn start(&mut self) {
        self.is_ongoing = true;
        self.progress = 0.0;
        self.effect_type = EffectType::random();
    }

    pub fn animate(&mut self, ui: &mut Ui) {
        if !self.is_ongoing {
            return;
        }
        match self.effect_type {
            EffectType::LeftToRight => self.slide(ui, SlideFrom::Right),
            EffectType::RightToLeft => self.slide(ui, SlideFrom::Left),
            EffectType::TopToBottom => self.slide(ui, SlideFrom::Bottom),
            EffectType::BottomToTop => self.slide(ui, SlideFrom::Top),
            EffectType::FadeIn => self.fade_in(ui),
            EffectType::ZoomOut => self.zoom(ui, 1.5),
            EffectType::ZoomIn => self.zoom(ui, 0.5),
            EffectType::Bounce => self.bounce(ui),
            EffectType::Pop => self.pop(ui),
            EffectType::Corner => self.corner(ui),
        }
    }

    /// Advances `progress` by `speed * dt`, stops the transition once it
    /// reaches 100.0, and requests another frame while still animating.
    /// Returns the normalized progress in the range [0.0, 1.0].
    fn advance(&mut self, ui: &Ui) -> f32 {
        let dt = ui.ctx().input(|i| i.stable_dt);
        self.progress += self.speed * dt;

        if self.progress >= 100.0 {
            self.progress = 100.0;
            self.is_ongoing = false;
        } else {
            ui.ctx().request_repaint();
        }

        self.progress / 100.0
    }

    /// Sets the current layer's transform for this frame.
    fn set_transform(&self, ui: &Ui, transform: TSTransform) {
        ui.ctx().set_transform_layer(ui.layer_id(), transform);
    }

    /// Slides the content in from one edge of the screen to its resting position.
    fn slide(&mut self, ui: &Ui, from: SlideFrom) {
        let content_rect = ui.ctx().content_rect();

        let distance = match from {
            SlideFrom::Left | SlideFrom::Right => content_rect.width(),
            SlideFrom::Top | SlideFrom::Bottom => content_rect.height(),
        };

        let t = self.advance(ui);
        let remaining = distance * (1.0 - t);

        let offset = match from {
            SlideFrom::Left => -remaining,
            SlideFrom::Right => remaining,
            SlideFrom::Top => -remaining,
            SlideFrom::Bottom => remaining,
        };

        let translation = match from {
            SlideFrom::Left | SlideFrom::Right => Vec2::new(offset, 0.0),
            SlideFrom::Top | SlideFrom::Bottom => Vec2::new(0.0, offset),
        };

        self.set_transform(ui, TSTransform::from_translation(translation));
    }

    /// Fades the whole page in from transparent to fully opaque.
    fn fade_in(&mut self, ui: &mut Ui) {
        let t = self.advance(ui);
        ui.set_opacity(t);
    }

    /// Scales the content from `start_scale` down/up to its normal size (1.0x),
    /// keeping the content centered on screen.
    fn zoom(&mut self, ui: &Ui, start_scale: f32) {
        let center = ui.ctx().content_rect().center().to_vec2();
        let t = self.advance(ui);
        let scaling = start_scale + (1.0 - start_scale) * t;

        // Keep the center fixed: center * scaling + translation == center.
        let translation = center - center * scaling;

        self.set_transform(ui, TSTransform::new(translation, scaling));
    }

    /// Slides in from the right like `RightToLeft`, but overshoots past its
    /// resting position and springs back into place.
    fn bounce(&mut self, ui: &Ui) {
        let width = ui.ctx().content_rect().width();
        let t = self.advance(ui);
        let eased = ease_out_back(t);
        let shift = width * (1.0 - eased);

        self.set_transform(ui, TSTransform::from_translation(Vec2::new(shift, 0.0)));
    }

    /// Scales up past normal size then settles back, like a bubble popping
    /// into view, combined with a fade-in.
    fn pop(&mut self, ui: &mut Ui) {
        let center = ui.ctx().content_rect().center().to_vec2();
        let t = self.advance(ui);
        let eased = ease_out_back(t).max(0.0);

        let translation = center - center * eased;
        ui.set_opacity(t);
        self.set_transform(ui, TSTransform::new(translation, eased));
    }

    /// Flies in diagonally from the top-right corner while zooming in,
    /// like a notification card dropping into place.
    fn corner(&mut self, ui: &Ui) {
        const START_SCALE: f32 = 0.6;

        let content_rect = ui.ctx().content_rect();
        let center = content_rect.center().to_vec2();
        let t = self.advance(ui);
        let scaling = START_SCALE + (1.0 - START_SCALE) * t;

        // The zoom-centering translation, plus a diagonal offset from the
        // top-right corner that shrinks to zero as the transition finishes.
        let centering = center - center * scaling;
        let corner_offset =
            Vec2::new(content_rect.width(), -content_rect.height()) * (1.0 - t) * 0.5;

        self.set_transform(ui, TSTransform::new(centering + corner_offset, scaling));
    }
}

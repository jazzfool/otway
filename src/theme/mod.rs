//! UI theme API.
//!
//! This API aims to be very generalized, hence the stringly-typed semantics.
//! However, there are some predefined string values should be handled (see `painters` and `colors`).
//!
//! Themes can be extended upon be implementing a new theme type which uses composition and delegation to extend an existing theme.

#[cfg(feature = "themes")]
pub mod flat;

use {crate::ui, reclutch::display as gfx, thiserror::Error};

#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("failed to load theme resource: {0}")]
    ResourceError(#[from] reclutch::error::ResourceError),
    #[error("failed to load theme font: {0}")]
    FontError(#[from] reclutch::error::FontError),
}

pub struct Painter<O: 'static, T: 'static>(
    Option<Box<dyn AnyPainter<T>>>,
    std::marker::PhantomData<O>,
);

pub trait TypedPainter<T: 'static>: AnyPainter<T> {
    type Object: 'static;

    fn paint(&mut self, obj: &mut Self::Object, aux: &mut ui::Aux<T>) -> Vec<gfx::DisplayCommand>;
    fn size_hint(&mut self, obj: &mut Self::Object) -> gfx::Size;
    fn metrics(&self, _obj: &Self::Object, _metric: &'static str) -> Option<f32> {
        None
    }
}

pub trait AnyPainter<T: 'static>: as_any::AsAny {
    fn paint(
        &mut self,
        obj: &mut dyn std::any::Any,
        aux: &mut ui::Aux<T>,
    ) -> Vec<gfx::DisplayCommand>;
    fn size_hint(&mut self, obj: &mut dyn std::any::Any) -> gfx::Size;
    fn metrics(&self, obj: &dyn std::any::Any, metrics: &'static str) -> Option<f32>;
}

impl<T: 'static, P: TypedPainter<T>> AnyPainter<T> for P {
    #[inline]
    fn paint(
        &mut self,
        obj: &mut dyn std::any::Any,
        aux: &mut ui::Aux<T>,
    ) -> Vec<gfx::DisplayCommand> {
        TypedPainter::paint(self, obj.downcast_mut::<P::Object>().unwrap(), aux)
    }

    #[inline]
    fn size_hint(&mut self, obj: &mut dyn std::any::Any) -> gfx::Size {
        TypedPainter::size_hint(self, obj.downcast_mut::<P::Object>().unwrap())
    }

    #[inline]
    fn metrics(&self, obj: &dyn std::any::Any, metric: &'static str) -> Option<f32> {
        TypedPainter::metrics(self, obj.downcast_ref::<P::Object>().unwrap(), metric)
    }
}

impl<T: 'static> as_any::Downcast for dyn AnyPainter<T> {}

#[cfg(feature = "kit")]
pub struct Standards {
    pub label_size: f32,
    pub button_text_alignment: ui::layout::Alignment,
}

pub trait Theme<T: 'static> {
    fn painter(&self, p: &'static str) -> Box<dyn AnyPainter<T>>;
    fn color(&self, c: &'static str) -> gfx::Color;

    #[cfg(feature = "kit")]
    fn standards(&self) -> Standards;
}

pub fn get_painter<O: 'static, T: 'static>(theme: &dyn Theme<T>, p: &'static str) -> Painter<O, T> {
    Painter(Some(theme.painter(p)), Default::default())
}

pub fn paint<O: 'static, T: 'static>(
    obj: &mut O,
    p: impl Fn(&mut O) -> &mut Painter<O, T>,
    aux: &mut ui::Aux<T>,
) -> Vec<gfx::DisplayCommand> {
    let mut painter = p(obj).0.take().unwrap();
    let out = AnyPainter::paint(&mut *painter, obj, aux);
    p(obj).0 = Some(painter);
    out
}

pub fn size_hint<O: 'static, T: 'static>(
    obj: &mut O,
    p: impl Fn(&mut O) -> &mut Painter<O, T>,
) -> gfx::Size {
    let mut painter = p(obj).0.take().unwrap();
    let out = AnyPainter::size_hint(&mut *painter, obj);
    p(obj).0 = Some(painter);
    out
}

pub fn metrics<O: 'static, T: 'static>(
    obj: &mut O,
    metric: &'static str,
    p: impl Fn(&mut O) -> &mut Painter<O, T>,
) -> Option<f32> {
    let painter = p(obj).0.take().unwrap();
    let out = AnyPainter::metrics(&*painter, obj, metric);
    p(obj).0 = Some(painter);
    out
}

pub fn multi_metrics<O: 'static, T: 'static>(
    obj: &mut O,
    metric: &[&'static str],
    p: impl Fn(&mut O) -> &mut Painter<O, T>,
) -> Vec<Option<f32>> {
    let painter = p(obj).0.take().unwrap();
    let mut out = Vec::new();
    for m in metric {
        out.push(AnyPainter::metrics(&*painter, obj, m));
    }
    p(obj).0 = Some(painter);
    out
}

pub mod painters {
    //! Standard painter definitions used by `kit`.
    //! For a theme to support `kit`, it must implement all of these.

    pub const BUTTON: &str = "button";
    pub const LABEL: &str = "label";
    pub const TEXT_BOX: &str = "text_box";
    pub const CHECK_MARK_BOX: &str = "check_mark_box";
    pub const COMBO_BOX: &str = "combo_box";
    pub const COMBO_LIST: &str = "combo_list";
    pub const COMBO_LIST_ITEM: &str = "combo_list_item";
}

pub mod metrics {
    //! Standard visual metrics definitions used by `kit`.
    //! For a theme to support `kit`, it must implement all of these.

    pub const PADDING_X: &str = "padding_x";
    pub const PADDING_Y: &str = "padding_y";
    pub const CHECK_MARK_SPACING: &str = "spacing";
}

pub mod colors {
    //! Standard color definitions used by `kit`.
    //! For a theme to support `kit`, it must implement all of these.

    /// Color used by text and other foreground elements.
    pub const FOREGROUND: &str = "foreground";
    /// Color used to fill general background elements.
    pub const BACKGROUND: &str = "background";
    /// A less contrasting version of the foreground.
    pub const WEAK_FOREGROUND: &str = "weak_foreground";
    /// A background element in the foreground. For example, the color of a button.
    pub const STRONG_BACKGROUND: &str = "strong_background";
    /// Color used by text-based controls (text boxes, combo-boxes, etc).
    pub const TEXT_CONTROL: &str = "text_control";
    /// An element that is "activated".
    pub const ACTIVE: &str = "active";
}

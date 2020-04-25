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
}

pub trait AnyPainter<T: 'static> {
    fn paint(
        &mut self,
        obj: &mut dyn std::any::Any,
        aux: &mut ui::Aux<T>,
    ) -> Vec<gfx::DisplayCommand>;
    fn size_hint(&mut self, obj: &mut dyn std::any::Any) -> gfx::Size;
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
}

pub trait Theme<T: 'static> {
    fn painter(&self, p: &'static str) -> Box<dyn AnyPainter<T>>;
    fn color(&self, c: &'static str) -> gfx::Color;
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

pub mod painters {
    //! Standard painter definitions used by `kit`.
    //! For a theme to support `kit`, it must implement all of these.

    pub const BUTTON: &str = "button";
    pub const LABEL: &str = "label";
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
    /// A less contrasting version of the background.
    pub const STRONG_FOREGROUND: &str = "strong_foreground";
    /// A background element in the foreground. For example, the color of a button.
    pub const STRONG_BACKGROUND: &str = "strong_background";
}

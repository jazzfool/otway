#[cfg(feature = "themes")]
pub mod flat;

use {crate::ui, reclutch::display as gfx};

pub struct Painter<O: 'static, T: 'static>(
    Option<Box<dyn AnyPainter<T>>>,
    std::marker::PhantomData<O>,
);

pub trait TypedPainter<T: 'static>: AnyPainter<T> {
    type Object: 'static;

    fn paint(
        &mut self,
        obj: &mut Self::Object,
        display: &mut dyn gfx::GraphicsDisplay,
        aux: &mut ui::Aux<T>,
    );
}

pub trait AnyPainter<T: 'static> {
    fn paint(
        &mut self,
        obj: &mut dyn std::any::Any,
        display: &mut dyn gfx::GraphicsDisplay,
        aux: &mut ui::Aux<T>,
    );
}

impl<T: 'static, P: TypedPainter<T>> AnyPainter<T> for P {
    fn paint(
        &mut self,
        obj: &mut dyn std::any::Any,
        display: &mut dyn gfx::GraphicsDisplay,
        aux: &mut ui::Aux<T>,
    ) {
        TypedPainter::paint(self, obj.downcast_mut::<P::Object>().unwrap(), display, aux);
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
    display: &mut dyn gfx::GraphicsDisplay,
    aux: &mut ui::Aux<T>,
) {
    let mut painter = p(obj).0.take().unwrap();
    AnyPainter::paint(&mut *painter, obj, display, aux);
    p(obj).0 = Some(painter);
}

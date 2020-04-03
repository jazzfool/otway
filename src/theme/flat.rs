use {
    crate::{kit, theme::*, ui},
    reclutch::display as gfx,
};

pub struct FlatTheme {}

impl FlatTheme {
    pub fn new() -> Self {
        FlatTheme {}
    }
}

impl<T: 'static> Theme<T> for FlatTheme {
    fn painter(&self, p: &'static str) -> Box<dyn AnyPainter<T>> {
        match p {
            painters::BUTTON => Box::new(ButtonPainter),
            _ => unimplemented!(),
        }
    }

    fn color(&self, c: &'static str) -> gfx::Color {
        match c {
            _ => unimplemented!(),
        }
    }
}

struct ButtonPainter;

impl<T: 'static> TypedPainter<T> for ButtonPainter {
    type Object = kit::Button<T>;

    fn paint(
        &mut self,
        _obj: &mut kit::Button<T>,
        _display: &mut dyn gfx::GraphicsDisplay,
        aux: &mut ui::Aux<T>,
    ) {
        aux.theme.color("foreground");
    }
}

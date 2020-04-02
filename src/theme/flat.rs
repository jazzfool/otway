use {
    crate::{kit, theme, ui},
    reclutch::display as gfx,
};

pub struct FlatTheme {}

impl<T: 'static> theme::Theme<T> for FlatTheme {
    fn painter(&self, p: &'static str) -> Box<dyn theme::AnyPainter<T>> {
        match p {
            "button" => Box::new(ButtonPainter),
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

impl<T: 'static> theme::TypedPainter<T> for ButtonPainter {
    type Object = kit::Button<T>;

    fn paint(
        &mut self,
        _obj: &mut kit::Button<T>,
        _display: &mut dyn gfx::GraphicsDisplay,
        _aux: &mut ui::Aux<T>,
    ) {
    }
}

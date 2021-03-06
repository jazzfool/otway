use {
    crate::{prelude::*, theme, ui},
    reclutch::display as gfx,
};

/// Displays text.
pub struct Label<T: 'static> {
    text: gfx::DisplayText,
    size: f32,
    max_width: Option<f32>,
    color: gfx::Color,

    painter: theme::Painter<Self>,
    common: ui::CommonRef,
}

impl<T: 'static> Label<T> {
    pub fn new(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> Self {
        Label {
            text: gfx::DisplayText::Simple(Default::default()),
            max_width: None,
            size: aux.theme.standards().label_size,
            color: aux.theme.color(theme::colors::FOREGROUND),
            painter: theme::get_painter(aux.theme.as_ref(), theme::painters::LABEL),
            common: ui::CommonRef::new(parent),
        }
    }

    pub fn set_text(&mut self, text: impl Into<gfx::DisplayText>) {
        self.text = text.into();
        self.repaint_and_resize();
    }

    #[inline]
    pub fn text(&self) -> &gfx::DisplayText {
        &self.text
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size;
        self.repaint_and_resize();
    }

    #[inline]
    pub fn size(&self) -> f32 {
        self.size
    }

    #[inline]
    pub fn set_max_width(&mut self, max_width: impl Into<Option<f32>>) {
        self.max_width = max_width.into();
        self.repaint_and_resize();
    }

    #[inline]
    pub fn max_width(&self) -> Option<f32> {
        self.max_width
    }

    pub fn set_color(&mut self, color: gfx::Color) {
        self.color = color;
        self.repaint();
    }

    #[inline]
    pub fn color(&self) -> gfx::Color {
        self.color
    }

    fn repaint_and_resize(&mut self) {
        self.repaint();
        let size = theme::size_hint(self, |x| &mut x.painter);
        ElementMixin::set_size(self, size);
    }
}

impl<T: 'static> ui::Element for Label<T> {
    type Aux = T;

    #[inline]
    fn common(&self) -> &ui::CommonRef {
        &self.common
    }

    #[inline]
    fn draw(&mut self, display: &mut dyn gfx::GraphicsDisplay, aux: &mut ui::Aux<T>) {
        ui::draw(
            self,
            |o, aux| theme::paint(o, |o| &mut o.painter, aux),
            display,
            aux,
            None,
        );
    }
}

impl<T: 'static> ui::WidgetChildren<T> for Label<T> {}

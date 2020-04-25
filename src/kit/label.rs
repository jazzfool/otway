use {
    crate::{theme, ui},
    reclutch::{display as gfx, widget::Widget},
};

/// Displays text.
#[derive(WidgetChildren)]
#[widget_children_trait(ui::WidgetChildren)]
pub struct Label<T: 'static> {
    text: gfx::DisplayText,
    size: f32,
    painter: theme::Painter<Self, T>,
    common: ui::CommonRef,
}

impl<T: 'static> Label<T> {
    pub fn new(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> Self {
        Label {
            text: gfx::DisplayText::Simple(Default::default()),
            size: aux.theme.standards().label_size,
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

    fn repaint_and_resize(&mut self) {
        self.common.with(|x| x.command_group_mut().repaint());
        let size = theme::size_hint(self, |x| &mut x.painter);
        self.common.with(|x| x.set_size(size));
    }
}

impl<T: 'static> ui::Element for Label<T> {
    #[inline]
    fn common(&self) -> &ui::CommonRef {
        &self.common
    }
}

impl<T: 'static> Widget for Label<T> {
    type UpdateAux = ui::Aux<T>;
    type GraphicalAux = ui::Aux<T>;
    type DisplayObject = gfx::DisplayCommand;

    #[inline]
    fn bounds(&self) -> gfx::Rect {
        self.common.with(|x| x.rect())
    }

    #[inline]
    fn draw(&mut self, display: &mut dyn gfx::GraphicsDisplay, aux: &mut ui::Aux<T>) {
        ui::draw(
            self,
            |o, aux| theme::paint(o, |o| &mut o.painter, aux),
            display,
            aux,
        );
    }
}

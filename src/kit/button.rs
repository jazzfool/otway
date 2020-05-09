use {
    crate::{kit, prelude::*, theme, ui},
    reclutch::display as gfx,
};

/// Simple labelled button control which emits interaction events.
pub struct Button<T: 'static> {
    label: kit::Label<T>,

    alignment: ui::layout::Alignment,
    painter: theme::Painter<Self, T>,
    common: ui::CommonRef,
    listener: ui::Listener<Self, ui::Aux<T>>,
}

impl<T: 'static> Button<T> {
    pub fn new(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> Self {
        let listener = kit::interaction_handler(aux, kit::interaction_forwarder(None), None);

        let common = ui::CommonRef::new(parent);

        Button {
            label: kit::Label::new(common.clone(), aux),
            alignment: aux.theme.standards().button_text_alignment,
            painter: theme::get_painter(aux.theme.as_ref(), theme::painters::BUTTON),
            common,
            listener,
        }
    }

    pub fn set_text(&mut self, text: impl Into<gfx::DisplayText>) {
        self.label.set_text(text);
        self.update_label();
    }

    #[inline]
    pub fn text(&self) -> &gfx::DisplayText {
        self.label.text()
    }

    pub fn set_alignment(&mut self, alignment: ui::layout::Alignment) {
        self.alignment = alignment;
        self.update_label();
    }

    #[inline]
    pub fn alignment(&self) -> ui::layout::Alignment {
        self.alignment
    }

    fn update_label(&mut self) {
        let label_bounds = self.label.bounds();
        let padding = theme::size_hint(self, |x| &mut x.painter);
        self.common
            .with(|x| x.set_size(label_bounds.size + padding));
        let bounds = self.bounds();
        let y = gfx::center_vertically(label_bounds, bounds).y - 1.0;
        let x = ui::layout::align_x(label_bounds, bounds, self.alignment, padding.width / 2.0);

        self.label
            .common()
            .with(|c| c.set_position(gfx::Point::new(x, y)));
    }
}

impl<T: 'static> ui::Element for Button<T> {
    type Aux = T;

    #[inline]
    fn common(&self) -> &ui::CommonRef {
        &self.common
    }

    #[inline]
    fn update(&mut self, aux: &mut ui::Aux<T>) {
        ui::propagate_repaint(self);
        ui::dispatch(self, aux, |x| &mut x.listener);
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

impl<T: 'static> ui::WidgetChildren<T> for Button<T> {
    crate::children![for <T>; label];
}

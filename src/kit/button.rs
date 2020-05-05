use {
    crate::{kit, prelude::*, theme, ui},
    reclutch::display as gfx,
};

/// Simple labelled button control which emits interaction events.
pub struct Button<T: 'static> {
    label: kit::Label<T>,

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
            painter: theme::get_painter(aux.theme.as_ref(), theme::painters::BUTTON),
            common,
            listener,
        }
    }

    pub fn set_text(&mut self, text: impl Into<gfx::DisplayText>) {
        self.label.set_text(text);
        let padding = theme::size_hint(self, |x| &mut x.painter);
        self.common
            .with(|x| x.set_size(self.label.bounds().size + padding));
    }

    #[inline]
    pub fn text(&self) -> &gfx::DisplayText {
        self.label.text()
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

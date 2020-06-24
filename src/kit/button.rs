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
    listeners: ui::ListenerList<Self, ui::Aux<T>>,
}

impl<T: 'static> Button<T> {
    pub fn new(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> Self {
        let common = ui::CommonRef::new(parent);

        let interaction_listener =
            kit::interaction_handler(aux, kit::interaction_forwarder(None), None, None);
        let focus_listener = kit::focus_handler(
            aux,
            kit::focus_forwarder(),
            kit::FocusConfig {
                mouse_trigger: Default::default(),
                interaction_handler: common.with(|x| x.id()),
            },
        );

        Button {
            label: kit::Label::new(common.clone(), aux),
            alignment: aux.theme.standards().button_text_alignment,
            painter: theme::get_painter(aux.theme.as_ref(), theme::painters::BUTTON),
            common,
            listeners: ui::ListenerList::new(vec![interaction_listener, focus_listener]),
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
        self.set_size(label_bounds.size + padding);
        let bounds = self.rect();
        let y = ui::layout::align_y(label_bounds, bounds, ui::layout::Alignment::Middle, 0.) - 1.;
        let x = ui::layout::align_x(label_bounds, bounds, self.alignment, padding.width / 2.0);

        self.label.set_position(gfx::Point::new(x, y));
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
        ui::dispatch_list(self, aux, |x| &mut x.listeners)
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

impl<T: 'static> ui::WidgetChildren<T> for Button<T> {
    crate::children![for <T>; label];
}

use {
    crate::{kit, prelude::*, theme, ui},
    reclutch::display as gfx,
};

/// Simple labelled button control which emits interaction events.
pub struct Button<T: 'static> {
    label: kit::Label<T>,
    alignment: ui::layout::Alignment,

    painter: theme::Painter<Self>,
    common: ui::CommonRef,
    listeners: ui::ListenerList<kit::ReadWrite<Self>>,
    components: ui::ComponentList<Self>,
}

impl<T: 'static> Button<T> {
    pub fn new(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> Self {
        let common = ui::CommonRef::new(parent);

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
            listeners: ui::ListenerList::new(vec![focus_listener]),

            components: ui::ComponentList::new().and_push(kit::InteractionState::new(
                aux,
                kit::interaction_forwarder(None),
                None,
                None,
            )),
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
        let padding = theme::multi_metrics(
            self,
            &[theme::metrics::PADDING_X, theme::metrics::PADDING_Y],
            |x| &mut x.painter,
        );
        let padding = gfx::Size::new(padding[0].unwrap(), padding[1].unwrap());
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
        ui::dispatch_components(self, aux, |x| &mut x.components).unwrap();
        ui::dispatch_list::<kit::ReadWrite<Self>, _>((self, aux), |(x, _)| &mut x.listeners);

        ui::propagate_repaint(self);
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

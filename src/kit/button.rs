use {
    crate::{kit, theme, ui},
    reclutch::{display as gfx, widget::Widget},
};

#[derive(Event, Debug, Clone, Copy, PartialEq)]
pub enum ButtonEvent {
    #[event_key(press)]
    Press(gfx::Point),
    #[event_key(release)]
    Release(gfx::Point),
}

/// Simple button control.
///
/// This button is labelled and has the following events;
/// - `press`; The button was pressed.
/// - `release`; The button was released from the press state. Always paired with a prior `press`.
#[derive(WidgetChildren)]
#[widget_children_trait(ui::WidgetChildren)]
pub struct Button<T: 'static> {
    text: gfx::DisplayText,
    painter: theme::Painter<Self, T>,
    cmds: ui::CommandGroup,
    common: ui::CommonRef,
    node: sinq::EventNode<Self, ui::Aux<T>, ButtonEvent>,
}

impl<T: 'static> Button<T> {
    pub fn new(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> Self {
        let mut node = sinq::EventNode::new(&mut aux.master);

        node.add(super::interaction_handler(
            aux,
            &|btn: &mut Self, aux, event| match event {
                kit::InteractionEvent::Press(pos) => {
                    btn.node
                        .emit_owned(ButtonEvent::Press(pos), &mut aux.master);
                }
                kit::InteractionEvent::Release(pos) => {
                    btn.node
                        .emit_owned(ButtonEvent::Release(pos), &mut aux.master);
                }
                _ => {}
            },
        ));

        Button {
            text: gfx::DisplayText::Simple(Default::default()),
            painter: theme::get_painter(aux.theme.as_ref(), theme::painters::BUTTON),
            cmds: Default::default(),
            common: ui::CommonRef::new(parent),
            node,
        }
    }

    pub fn set_text(&mut self, text: impl Into<gfx::DisplayText>) {
        self.text = text.into();
        self.cmds.repaint();

        let size = theme::size_hint(self, |x| &mut x.painter);
        self.common.get_mut().set_size(size);
    }

    #[inline]
    pub fn text(&self) -> &gfx::DisplayText {
        &self.text
    }
}

impl<T: 'static> ui::Element for Button<T> {
    #[inline]
    fn common(&self) -> &ui::CommonRef {
        &self.common
    }
}

impl<T: 'static> Widget for Button<T> {
    type UpdateAux = ui::Aux<T>;
    type GraphicalAux = ui::Aux<T>;
    type DisplayObject = gfx::DisplayCommand;

    #[inline]
    fn bounds(&self) -> gfx::Rect {
        self.common.get_ref().rect()
    }

    #[inline]
    fn update(&mut self, aux: &mut ui::Aux<T>) {
        ui::update(self, aux);
    }

    #[inline]
    fn draw(&mut self, display: &mut dyn gfx::GraphicsDisplay, aux: &mut ui::Aux<T>) {
        ui::draw(
            self,
            |o| &mut o.cmds,
            |o, aux| theme::paint(o, |o| &mut o.painter, aux),
            display,
            aux,
        );
    }
}

impl<T: 'static> ui::Node for Button<T> {
    type Event = ButtonEvent;

    #[inline]
    fn node_ref(&self) -> &sinq::EventNode<Self, ui::Aux<T>, ButtonEvent> {
        &self.node
    }

    #[inline]
    fn node_mut(&mut self) -> &mut sinq::EventNode<Self, ui::Aux<T>, ButtonEvent> {
        &mut self.node
    }
}

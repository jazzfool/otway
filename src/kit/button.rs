use {
    crate::{theme, ui},
    reclutch::{display as gfx, widget::Widget},
};

#[derive(Event, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ButtonEvent {
    #[event_key(press)]
    Press,
}

#[derive(WidgetChildren)]
#[widget_children_trait(ui::WidgetChildren)]
pub struct Button<T: 'static> {
    painter: theme::Painter<Self, T>,
    cmds: ui::CommandGroup,
    common: ui::CommonRef,
    node: sinq::EventNode<Self, ui::Aux<T>, ButtonEvent>,
}

impl<T: 'static> Button<T> {
    pub fn new(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> Self {
        Button {
            painter: theme::get_painter(aux.theme.as_ref(), theme::painters::BUTTON),
            cmds: Default::default(),
            common: ui::CommonRef::new(parent),
            node: sinq::EventNode::new(&mut aux.master),
        }
    }
}

impl<T: 'static> ui::Element for Button<T> {
    fn common(&self) -> &ui::CommonRef {
        &self.common
    }
}

impl<T: 'static> Widget for Button<T> {
    type UpdateAux = ui::Aux<T>;
    type GraphicalAux = ui::Aux<T>;
    type DisplayObject = gfx::DisplayCommand;

    fn bounds(&self) -> gfx::Rect {
        self.common.get().rect()
    }

    fn update(&mut self, aux: &mut ui::Aux<T>) {
        ui::update(self, aux);
        ui::propagate_update(self, aux);
    }

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

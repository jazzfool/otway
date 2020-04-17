use {
    crate::{theme, ui},
    reclutch::{display as gfx, widget::Widget},
};

/// Displays text.
#[derive(WidgetChildren)]
#[widget_children_trait(ui::WidgetChildren)]
pub struct Label<T: 'static> {
    text: gfx::DisplayText,
    painter: theme::Painter<Self, T>,
    common: ui::CommonRef,
    node: sinq::EventNode<Self, ui::Aux<T>, ui::NoEvent>,
}

impl<T: 'static> Label<T> {
    pub fn new(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> Self {
        Label {
            text: gfx::DisplayText::Simple(Default::default()),
            painter: theme::get_painter(aux.theme.as_ref(), theme::painters::BUTTON),
            common: ui::CommonRef::new(parent),
            node: sinq::EventNode::new(&mut aux.master),
        }
    }

    pub fn set_text(&mut self, text: impl Into<gfx::DisplayText>) {
        self.text = text.into();
        self.common.get_mut().command_group_mut().repaint();

        let size = theme::size_hint(self, |x| &mut x.painter);
        self.common.get_mut().set_size(size);
    }

    #[inline]
    pub fn text(&self) -> &gfx::DisplayText {
        &self.text
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
            |o, aux| theme::paint(o, |o| &mut o.painter, aux),
            display,
            aux,
        );
    }
}

impl<T: 'static> ui::Node for Label<T> {
    type Event = ui::NoEvent;

    #[inline]
    fn node_ref(&self) -> &sinq::EventNode<Self, ui::Aux<T>, ui::NoEvent> {
        &self.node
    }

    #[inline]
    fn node_mut(&mut self) -> &mut sinq::EventNode<Self, ui::Aux<T>, ui::NoEvent> {
        &mut self.node
    }
}

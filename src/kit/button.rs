use {
    crate::{theme, ui},
    reclutch::{display as gfx, verbgraph as vg, widget::Widget},
};

#[derive(WidgetChildren)]
#[widget_children_trait(crate::ui::WidgetChildren)]
pub struct Button<T: 'static> {
    painter: theme::Painter<Self, T>,
    common: ui::CommonRef,
    graph: vg::OptionVerbGraph<Self, ui::Aux<T>>,
}

impl<T: 'static> Button<T> {
    pub fn new(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> Self {
        let graph = vg::VerbGraph::new();

        Button {
            painter: theme::get_painter(aux.theme.as_ref(), theme::painters::BUTTON),
            common: ui::CommonRef::new(parent),
            graph: Some(graph),
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
        vg::update_all(self, aux);
    }

    fn draw(&mut self, display: &mut dyn gfx::GraphicsDisplay, aux: &mut ui::Aux<T>) {
        theme::paint(self, |x| &mut x.painter, display, aux);
    }
}

impl<T: 'static> vg::HasVerbGraph for Button<T> {
    fn verb_graph(&mut self) -> &mut vg::OptionVerbGraph<Self, ui::Aux<T>> {
        &mut self.graph
    }
}

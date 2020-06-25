use {
    crate::{kit, prelude::*, theme, ui},
    reclutch::display as gfx,
};

pub struct CheckMarkToggledEvent(pub bool);

pub struct CheckMarkBox<T: 'static> {
    checked: bool,

    painter: theme::Painter<Self, T>,
    common: ui::CommonRef,
    listeners: ui::ListenerList<Self, ui::Aux<T>>,
}

impl<T: 'static> CheckMarkBox<T> {
    pub fn new(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> Self {
        let common = ui::CommonRef::new(parent);

        let interaction_listener = kit::interaction_handler(
            aux,
            |obj: &mut Self, aux, ev| {
                if let kit::InteractionEvent::Press(_) = ev {
                    obj.toggle();
                    obj.emit(aux, CheckMarkToggledEvent(obj.checked));
                }
                kit::interaction_forwarder(None)(obj, aux, ev);
            },
            None,
            None,
        );

        let mut cm = CheckMarkBox {
            checked: false,

            painter: theme::get_painter(aux.theme.as_ref(), theme::painters::CHECK_MARK_BOX),
            common,
            listeners: ui::ListenerList::new(vec![interaction_listener]),
        };

        let size = theme::size_hint(&mut cm, |x| &mut x.painter);
        ElementMixin::set_size(&cm, size);
        
        cm
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
        self.repaint();
    }

    #[inline]
    pub fn checked(&self) -> bool {
        self.checked
    }

    pub fn toggle(&mut self) {
        self.checked = !self.checked;
        self.repaint();
    }
}

impl<T: 'static> ui::Element for CheckMarkBox<T> {
    type Aux = T;

    #[inline]
    fn common(&self) -> &ui::CommonRef {
        &self.common
    }

    #[inline]
    fn update(&mut self, aux: &mut ui::Aux<T>) {
        ui::dispatch_list(self, aux, |x| &mut x.listeners);
    }

    fn draw(&mut self, display: &mut dyn gfx::GraphicsDisplay, aux: &mut ui::Aux<T>) {
        ui::draw(
            self,
            |o, a| theme::paint(o, |o| &mut o.painter, a),
            display,
            aux,
            None,
        );
    }
}

impl<T: 'static> ui::WidgetChildren<T> for CheckMarkBox<T> {}

pub struct CheckBox<T: 'static> {
    check_mark: CheckMarkBox<T>,
    label: kit::Label<T>,

    common: ui::CommonRef,
    listeners: ui::ListenerList<Self, ui::Aux<T>>,
}

impl<T: 'static> CheckBox<T> {
    pub fn new(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> Self {
        let common = ui::CommonRef::new(parent);

        let interaction_listener = kit::interaction_handler(
            aux,
            |obj, aux, ev| {
                kit::interaction_forwarder(None)(obj, aux, ev);
            },
            None,
            None,
        );

        let mut check_mark = CheckMarkBox::new(common.clone(), aux);
        let label = kit::Label::new(common.clone(), aux);

        let mut hstack = ui::layout::HStack::new().into_node(None);
        hstack.push(&check_mark, None);
        hstack.push(
            &label,
            Some(
                (
                    theme::metrics(&mut check_mark, theme::metrics::CHECK_MARK_SPACING, |x| {
                        &mut x.painter
                    })
                    .unwrap(),
                    0.0,
                )
                    .into(),
            ),
        );
        common.with(move |x| {
            x.set_layout(hstack);
            x.set_layout_mode(ui::LayoutMode::Shrink);
        });

        CheckBox {
            check_mark,
            label,

            common,
            listeners: ui::ListenerList::new(vec![interaction_listener]),
        }
    }
}

impl<T: 'static> ui::Element for CheckBox<T> {
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
}

impl<T: 'static> ui::WidgetChildren<T> for CheckBox<T> {
    crate::children![for <T>; check_mark, label];
}

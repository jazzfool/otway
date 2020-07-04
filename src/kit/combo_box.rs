use {
    crate::{kit, prelude::*, theme, ui},
    reclutch::display as gfx,
};

pub struct ComboListItem<T: 'static> {
    label: kit::Label<T>,
    selected: bool,

    painter: theme::Painter<Self>,
    common: ui::CommonRef,
    listeners: ui::ListenerList<kit::ReadWrite<Self>>,
    components: ui::ComponentList<Self>,
}

impl<T: 'static> ComboListItem<T> {
    pub fn new(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> Self {
        let common = ui::CommonRef::new(parent);

        ComboListItem {
            label: kit::Label::new(common.clone(), aux),
            selected: false,

            painter: theme::get_painter(aux.theme.as_ref(), theme::painters::COMBO_LIST_ITEM),
            common,
            listeners: ui::ListenerList::new(vec![]),
            components: ui::ComponentList::new().and_push(kit::InteractionState::new(
                aux,
                kit::interaction_forwarder(None),
                None,
                None,
            )),
        }
    }

    pub fn set_text(&mut self, text: impl ToString) {
        self.label.set_text(text.to_string());
        self.resize();
    }

    pub fn text(&self) -> String {
        match self.label.text() {
            gfx::DisplayText::Simple(s) => s.clone(),
            _ => String::new(),
        }
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
        self.repaint();
    }

    #[inline]
    pub fn selected(&self) -> bool {
        self.selected
    }

    fn resize(&mut self) {
        self.set_size(self.label.bounds().size);
        self.repaint();
    }
}

impl<T: 'static> ui::Element for ComboListItem<T> {
    type Aux = T;

    #[inline]
    fn common(&self) -> &ui::CommonRef {
        &self.common
    }

    #[inline]
    fn update(&mut self, aux: &mut ui::Aux<Self::Aux>) {
        ui::dispatch_components(self, aux, |x| &mut x.components).unwrap();
        ui::dispatch_list::<kit::ReadWrite<Self>, _>((self, aux), |(x, _)| &mut x.listeners);
    }

    fn draw(&mut self, display: &mut dyn gfx::GraphicsDisplay, aux: &mut ui::Aux<Self::Aux>) {
        ui::draw(
            self,
            |o, a| theme::paint(o, |o| &mut o.painter, a),
            display,
            aux,
            None,
        )
    }
}

impl<T: 'static> ui::WidgetChildren<T> for ComboListItem<T> {
    crate::children![for <T>; label];
}

pub struct ComboList<T: 'static> {
    combos: Vec<String>,
    items: Vec<ComboListItem<T>>,

    painter: theme::Painter<Self>,
    common: ui::CommonRef,
    listeners: ui::ListenerList<(ui::Write<Self>, ui::Write<ui::Aux<T>>)>,
    components: ui::ComponentList<Self>,
}

impl<T: 'static> ComboList<T> {
    pub fn new(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> Self {
        let common = ui::CommonRef::new(parent);

        let focus_listener = kit::focus_handler(
            aux,
            kit::focus_forwarder(),
            kit::FocusConfig {
                interaction_handler: common.with(|x| x.id()),
                mouse_trigger: Default::default(),
            },
        );

        ComboList {
            combos: Vec::new(),
            items: Vec::new(),

            painter: theme::get_painter(aux.theme.as_ref(), theme::painters::COMBO_LIST),
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

    pub fn set_combos(&mut self, combos: &[String], aux: &mut ui::Aux<T>) {
        self.combos = combos.to_owned();
        self.update_items(aux);
    }

    #[inline]
    pub fn combos(&self) -> &[String] {
        &self.combos
    }

    fn update_items(&mut self, aux: &mut ui::Aux<T>) {
        let mut stack = ui::layout::VStack::new().into_node(None);

        self.items = Vec::with_capacity(self.combos.len());
        let w = self.size().width;
        let mut h = 0.;
        for combo in &self.combos {
            let mut item = ComboListItem::new(self.common.clone(), aux);
            item.set_text(combo);

            let item_size = item.size();
            h += item_size.height;
            item.set_size(gfx::Size::new(w, item_size.height));

            stack.push(&item, None);
            self.items.push(item);
        }
        self.set_size(gfx::Size::new(w, h));

        self.set_layout(stack);
        ui::layout::update_layout(self);
    }
}

impl<T: 'static> ui::Element for ComboList<T> {
    type Aux = T;

    #[inline]
    fn common(&self) -> &ui::CommonRef {
        &self.common
    }

    fn update(&mut self, aux: &mut ui::Aux<Self::Aux>) {
        ui::dispatch_components(self, aux, |x| &mut x.components).unwrap();
        ui::dispatch_list::<kit::ReadWrite<Self>, _>((self, aux), |(x, _)| &mut x.listeners);

        ui::propagate_repaint(self);
    }

    fn draw(&mut self, display: &mut dyn gfx::GraphicsDisplay, aux: &mut ui::Aux<Self::Aux>) {
        ui::draw(
            self,
            |o, a| theme::paint(o, |o| &mut o.painter, a),
            display,
            aux,
            None,
        )
    }
}

impl<T: 'static> ui::WidgetChildren<T> for ComboList<T> {
    fn children(&self) -> Vec<&dyn WidgetChildren<T>> {
        self.items
            .iter()
            .map(|x| x as &dyn WidgetChildren<T>)
            .collect()
    }

    fn children_mut(&mut self) -> Vec<&mut dyn WidgetChildren<T>> {
        self.items
            .iter_mut()
            .map(|x| x as &mut dyn WidgetChildren<T>)
            .collect()
    }
}

pub struct ComboBox<T: 'static> {
    combos: Vec<String>,
    label: kit::Label<T>,
    list: Option<ComboList<T>>,
    selected: Option<usize>,

    painter: theme::Painter<Self>,
    common: ui::CommonRef,
    listeners: ui::ListenerList<kit::ReadWrite<Self>>,
    components: ui::ComponentList<Self>,
}

impl<T: 'static> ComboBox<T> {
    pub fn new(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> Self {
        let common = ui::CommonRef::new(parent);

        let focus_listener = kit::focus_handler(
            aux,
            kit::focus_forwarder(),
            kit::FocusConfig {
                interaction_handler: common.with(|x| x.id()),
                mouse_trigger: Default::default(),
            },
        );

        ComboBox {
            combos: Vec::new(),
            label: kit::Label::new(common.clone(), aux),
            list: None,
            selected: None,

            painter: theme::get_painter(aux.theme.as_ref(), theme::painters::COMBO_BOX),
            common,
            listeners: ui::ListenerList::new(vec![focus_listener]),
            components: ui::ComponentList::new().and_push(kit::InteractionState::new(
                aux,
                |obj: &mut Self, aux, ev| {
                    match ev {
                        kit::InteractionEvent::Press(_) => obj.show_combo_list(aux),
                        _ => {}
                    }

                    kit::interaction_forwarder(None)(obj, aux, ev);
                },
                None,
                None,
            )),
        }
    }

    pub fn set_combos(&mut self, combos: &[String], aux: &mut ui::Aux<T>) {
        self.combos = combos.to_vec();
        self.selected = if self.combos.is_empty() {
            None
        } else {
            Some(0)
        };
        self.show_combo_list(aux);
        self.repaint();
        self.update_label();
        self.resize();
    }

    #[inline]
    pub fn combos(&self) -> &[String] {
        &self.combos
    }

    pub fn set_selected(&mut self, selected: usize) {
        self.selected = Some(selected);
        self.repaint();
        self.update_label();
        self.resize();
    }

    #[inline]
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn selected_combo(&self) -> Option<&str> {
        self.selected
            .and_then(|x| self.combos.get(x).map(|x| &x[..]))
    }

    pub fn show_combo_list(&mut self, aux: &mut ui::Aux<T>) {
        let mut list = ComboList::new(self.common.clone(), aux);
        list.set_combos(&self.combos, aux);
        self.list = Some(list);
    }

    #[inline]
    pub fn hide_combo_list(&mut self) {
        self.list = None;
    }

    #[inline]
    pub fn is_combo_list_open(&self) -> bool {
        !self.list.is_none()
    }

    fn update_label(&mut self) {
        self.label.set_text(
            self.selected_combo()
                .map(|x| x.to_string())
                .unwrap_or_default(),
        );
    }

    fn resize(&mut self) {
        let metrics = theme::multi_metrics(
            self,
            &[theme::metrics::PADDING_X, theme::metrics::PADDING_Y],
            |x| &mut x.painter,
        );

        let padding = gfx::Size::new(metrics[0].unwrap(), metrics[1].unwrap());
        let label_bounds = self.label.bounds();
        self.set_size(label_bounds.size + padding);

        let bounds = self.bounds();

        let x = ui::layout::align_x(label_bounds, bounds, ui::layout::Alignment::Begin, 6.);
        let y = ui::layout::align_y(label_bounds, bounds, ui::layout::Alignment::Middle, 0.) - 1.;

        self.label.set_position(gfx::Point::new(x, y));
    }
}

impl<T: 'static> ui::Element for ComboBox<T> {
    type Aux = T;

    #[inline]
    fn common(&self) -> &ui::CommonRef {
        &self.common
    }

    #[inline]
    fn update(&mut self, aux: &mut ui::Aux<Self::Aux>) {
        ui::dispatch_components(self, aux, |x| &mut x.components).unwrap();
        ui::dispatch_list::<kit::ReadWrite<Self>, _>((self, aux), |(x, _)| &mut x.listeners);

        ui::propagate_repaint(self);
    }

    fn draw(&mut self, display: &mut dyn gfx::GraphicsDisplay, aux: &mut ui::Aux<Self::Aux>) {
        ui::draw(
            self,
            |o, a| theme::paint(o, |o| &mut o.painter, a),
            display,
            aux,
            None,
        )
    }
}

impl<T: 'static> ui::WidgetChildren<T> for ComboBox<T> {
    fn children(&self) -> Vec<&dyn ui::WidgetChildren<T>> {
        if let Some(list) = &self.list {
            vec![&self.label, list]
        } else {
            vec![&self.label]
        }
    }

    fn children_mut(&mut self) -> Vec<&mut dyn ui::WidgetChildren<T>> {
        if let Some(list) = &mut self.list {
            vec![&mut self.label, list]
        } else {
            vec![&mut self.label]
        }
    }
}

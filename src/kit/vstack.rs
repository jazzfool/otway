use crate::{kit, ui};

pub struct VStack<T: 'static> {
    children: Vec<(ui::CommonRef, VStackConfig)>,
    dirty: bool,
    common: ui::CommonRef,
    phantom: std::marker::PhantomData<T>,
}

impl<T: 'static> VStack<T> {
    pub fn new(parent: ui::CommonRef, _aux: &mut ui::Aux<T>) -> Self {
        let common = ui::CommonRef::new(parent);
        VStack {
            children: Default::default(),
            dirty: true,
            common,
            phantom: Default::default(),
        }
    }
}

impl<T: 'static> ui::Element for VStack<T> {
    type Aux = T;

    #[inline]
    fn common(&self) -> &ui::CommonRef {
        &self.common
    }
}

impl<T: 'static> ui::Layout<T> for VStack<T> {
    type Config = Option<VStackConfig>;

    fn push(&mut self, child: ui::CommonRef, config: Option<VStackConfig>) {
        if !self.has(&child) {
            let config = config.unwrap_or_default();
            self.children.push((child, config));
            self.dirty = true;
        }
    }

    fn insert(&mut self, child: ui::CommonRef, config: Option<VStackConfig>, index: usize) {
        if !self.has(&child) {
            let config = config.unwrap_or_default();
            self.children.insert(index, (child, config));
            self.dirty = true;
        }
    }

    fn reorder(&mut self, child: &ui::CommonRef, new_index: usize) {
        if let Some(idx) = self.position(child) {
            let (child, config) = self.children.remove(idx);
            self.insert(child, Some(config), new_index);
        }
    }

    fn remove(&mut self, child: &ui::CommonRef) {
        if let Some(idx) = self.position(child) {
            self.children.remove(idx);
            self.dirty = true;
        }
    }

    fn set(&mut self, child: &ui::CommonRef, config: Option<VStackConfig>) {
        if let Some(idx) = self.position(child) {
            self.children[idx].1 = config.unwrap_or_default();
        }
    }

    #[inline]
    fn has(&self, child: &ui::CommonRef) -> bool {
        self.position(child).is_some()
    }

    #[inline]
    fn position(&self, child: &ui::CommonRef) -> Option<usize> {
        self.children.iter().position(|(c, _)| c == child)
    }

    #[inline]
    fn len(&self) -> usize {
        self.children.len()
    }
}

impl<T: 'static> ui::WidgetChildren<T> for VStack<T> {}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct VStackConfig {
    pub margin: kit::SideMargins,
    pub align: kit::Alignment,
}

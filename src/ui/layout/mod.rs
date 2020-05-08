pub mod hstack;
pub mod vstack;

use {
    crate::{prelude::*, ui},
    reclutch::display as gfx,
};

pub enum Item {
    Widget(ui::CommonRef),
    Layout(DynamicNode),
}

impl Item {
    pub fn is_widget(&self) -> bool {
        match self {
            Item::Widget(_) => true,
            Item::Layout(_) => false,
        }
    }

    pub fn set_rect(&mut self, rect: gfx::Rect) {
        match self {
            Item::Widget(w) => w.with(|x| x.set_rect(rect)),
            Item::Layout(l) => l.0.set_rect(rect),
        }
    }

    pub fn rect(&self) -> gfx::Rect {
        match self {
            Item::Widget(w) => w.with(|x| x.rect()),
            Item::Layout(l) => l.0.rect(),
        }
    }
}

impl<E: Element> From<&E> for Item {
    #[inline]
    fn from(e: &E) -> Self {
        Item::Widget(e.common().clone())
    }
}

impl<L: Layout> From<Node<L>> for Item {
    #[inline]
    fn from(l: Node<L>) -> Self {
        Item::Layout(DynamicNode(Box::new(l)))
    }
}

pub trait Layout: 'static {
    type Config: 'static;
    type Id;

    fn push(&mut self, item: impl Into<Item>, config: Self::Config) -> Self::Id;
    fn remove(&mut self, id: &Self::Id);

    fn get(&self, id: &Self::Id) -> Option<&Item>;
    fn get_mut(&mut self, id: &Self::Id) -> Option<&mut Item>;
    fn len(&self) -> usize;

    fn min_size(&self) -> gfx::Size;
    fn update(&mut self, bounds: gfx::Rect);
}

pub(crate) trait DynNode: as_any::AsAny {
    fn resize(&mut self);
    fn update(&mut self);
    fn set_rect(&mut self, rect: gfx::Rect);
    fn rect(&self) -> gfx::Rect;
}

pub struct Node<L: Layout> {
    layout: L,
    rect: gfx::Rect,
    dynamic: bool,
    layouts: Vec<L::Id>,
}

impl<L: Layout> Node<L> {
    #[inline]
    pub fn new(layout: L, size: Option<gfx::Size>) -> Self {
        Node::with_position(layout, Default::default(), size)
    }

    pub fn with_position(layout: L, position: gfx::Point, size: Option<gfx::Size>) -> Self {
        Node {
            layout,
            rect: gfx::Rect::new(position, size.unwrap_or_default()),
            dynamic: size.is_none(),
            layouts: Default::default(),
        }
    }

    pub fn push(&mut self, item: impl Into<Item>, config: L::Config) -> L::Id
    where
        L::Id: Clone,
    {
        let item = item.into();
        let is_layout = !item.is_widget();
        let id = self.layout.push(item, config);
        if is_layout {
            self.layouts.push(id.clone());
        }
        id
    }

    pub fn remove(&mut self, id: &L::Id)
    where
        L::Id: PartialEq,
    {
        if let Some(idx) = self.layouts.iter().position(|x| *x == *id) {
            self.layouts.remove(idx);
        }

        self.layout.remove(id);
    }
}

impl<L: Layout> std::ops::Deref for Node<L> {
    type Target = L;

    #[inline]
    fn deref(&self) -> &L {
        &self.layout
    }
}

impl<L: Layout> std::ops::DerefMut for Node<L> {
    #[inline]
    fn deref_mut(&mut self) -> &mut L {
        &mut self.layout
    }
}

impl<L: Layout> DynNode for Node<L> {
    fn resize(&mut self) {
        for id in &self.layouts {
            if let Some(child) = self.layout.get_mut(id) {
                match child {
                    Item::Layout(node) => node.0.resize(),
                    _ => {}
                }
            }
        }

        if self.dynamic {
            self.rect.size = self.layout.min_size();
        }
    }

    fn update(&mut self) {
        self.layout.update(self.rect);
        for id in &self.layouts {
            if let Some(child) = self.layout.get_mut(id) {
                match child {
                    Item::Layout(node) => node.0.update(),
                    _ => {}
                }
            }
        }
    }

    #[inline]
    fn set_rect(&mut self, rect: gfx::Rect) {
        self.rect = rect;
    }

    #[inline]
    fn rect(&self) -> gfx::Rect {
        self.rect
    }
}

impl as_any::Downcast for dyn DynNode {}

pub struct DynamicNode(pub(crate) Box<dyn DynNode>);

pub fn update_layout<T: 'static>(widget: &dyn WidgetChildren<T>) {
    resize_layout(widget);

    widget.common().with(|x| {
        if let Some(DynamicNode(layout)) = &mut x.layout {
            layout.update();
        }
    });

    for child in widget.children() {
        update_layout(child);
    }
}

fn resize_layout<T: 'static>(widget: &dyn WidgetChildren<T>) {
    for child in widget.children() {
        update_layout(child);
    }

    widget.common().with(|x| {
        if let Some(DynamicNode(layout)) = &mut x.layout {
            layout.resize();
        }
    });
}

pub type SideMargins = reclutch::euclid::SideOffsets2D<f32, reclutch::euclid::UnknownUnit>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Alignment {
    Begin,
    Middle,
    End,
}

impl Default for Alignment {
    #[inline]
    fn default() -> Self {
        Alignment::Middle
    }
}

pub fn align_x(inner: gfx::Rect, outer: gfx::Rect, align: Alignment, padding: f32) -> f32 {
    match align {
        Alignment::Begin => outer.origin.x + padding,
        Alignment::Middle => gfx::center_horizontally(inner, outer).x,
        Alignment::End => outer.max_x() - inner.size.width - padding,
    }
}

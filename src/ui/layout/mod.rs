pub mod hstack;
pub mod relative_box;
pub mod vfill;
pub mod vstack;

pub use {hstack::*, relative_box::*, vfill::*, vstack::*};

use {
    crate::{prelude::*, ui},
    as_any::Downcast,
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

impl From<ui::CommonRef> for Item {
    #[inline]
    fn from(c: ui::CommonRef) -> Self {
        Item::Widget(c)
    }
}

pub trait Layout: 'static {
    type Config: 'static;
    type Id: Clone;

    fn push(&mut self, item: impl Into<Item>, config: Self::Config) -> Self::Id;
    fn remove(&mut self, id: &Self::Id) -> Option<Item>;

    fn get(&self, id: &Self::Id) -> Option<&Item>;
    fn get_mut(&mut self, id: &Self::Id) -> Option<&mut Item>;
    fn len(&self) -> usize;
    fn items(&self) -> Vec<(&Item, &Self::Id)>;

    fn min_size(&self) -> gfx::Size;
    fn update(&mut self, bounds: gfx::Rect);

    #[inline]
    fn into_node(self, size: Option<gfx::Size>) -> Node<Self>
    where
        Self: Sized,
    {
        Node::new(self, size)
    }
}

/// Returns a boolean indicating whether an item should be subject to layout.
pub fn should_layout(item: &Item) -> bool {
    if let Item::Widget(c) = item {
        let v = c.with(|x| x.visible());
        v != ui::Visibility::NoLayout && v != ui::Visibility::None
    } else {
        true
    }
}

pub(crate) trait DynNode: as_any::AsAny {
    fn resize(&mut self);
    fn update(&mut self);
    fn process_detachments(&mut self);
    fn set_rect(&mut self, rect: gfx::Rect);
    fn rect(&self) -> gfx::Rect;
    fn set_size(&mut self, size: Option<gfx::Size>);
}

#[derive(Debug, Clone)]
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

    pub fn push(&mut self, item: impl Into<Item>, config: L::Config) -> L::Id {
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

    fn process_detachments(&mut self) {
        let mut removal = Vec::new();
        for (item, id) in self.layout.items().clone() {
            if let Item::Widget(widget) = item {
                if widget.with(|x| x.is_marked_for_detach()) {
                    // the layout is wrongly keeping the widget alive
                    removal.push(id.clone());
                }
            }
        }
        for id in removal {
            self.layout.remove(&id);
        }
    }

    fn set_size(&mut self, size: Option<gfx::Size>) {
        self.dynamic = size.is_none();
        self.rect.size = size.unwrap_or_default();
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

impl Downcast for dyn DynNode {}

pub struct DynamicNode(pub(crate) Box<dyn DynNode>);

impl DynamicNode {
    pub fn cast_mut<L: Layout>(&mut self) -> Option<&mut Node<L>> {
        self.0.as_mut().downcast_mut::<Node<L>>()
    }
}

pub fn update_direct_layout(common: &ui::CommonRef) {
    common.with(|x| {
        if let Some(DynamicNode(layout)) = &mut x.layout {
            layout.process_detachments();
            layout.update();
        }
    });
}

pub fn update_layout<T: 'static>(widget: &dyn WidgetChildren<T>) {
    resize_layout(widget);
    update_layout_impl(widget);
}

fn update_layout_impl<T: 'static>(widget: &dyn WidgetChildren<T>) {
    widget.common().with(|x| {
        if let Some(DynamicNode(layout)) = &mut x.layout {
            layout.process_detachments();
            layout.update();
        }
    });

    for child in widget.children() {
        update_layout(child);
    }
}

fn resize_layout<T: 'static>(widget: &dyn WidgetChildren<T>) {
    for child in widget.children() {
        resize_layout(child);
    }

    widget.common().with(|x| {
        if let Some(DynamicNode(layout)) = &mut x.layout {
            layout.resize();
            x.update_layout_size();
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
        Alignment::Begin
    }
}

pub fn align_x(inner: gfx::Rect, outer: gfx::Rect, align: Alignment, padding: f32) -> f32 {
    match align {
        Alignment::Begin => outer.origin.x + padding,
        Alignment::Middle => gfx::center_horizontally(inner, outer).x,
        Alignment::End => outer.max_x() - inner.size.width - padding,
    }
}

pub fn align_y(inner: gfx::Rect, outer: gfx::Rect, align: Alignment, padding: f32) -> f32 {
    match align {
        Alignment::Begin => outer.origin.y + padding,
        Alignment::Middle => gfx::center_vertically(inner, outer).y,
        Alignment::End => outer.max_y() - inner.size.height - padding,
    }
}

pub mod view;

use {
    crate::theme::Theme,
    reclutch::{display as gfx, widget::Widget},
    std::{
        cell::{Ref, RefCell, RefMut},
        rc::Rc,
    },
};

/// Global auxiliary type.
///
/// `T` generic is the additional data to be stored.
pub struct Aux<T: 'static> {
    pub data: T,
    pub theme: Box<dyn Theme<T>>,
}

/// Helper type to store a counted reference to a `Common`, or in other words, a reference to the core of a widget type (not the widget type itself).
#[derive(Debug, Clone, PartialEq)]
pub struct CommonRef(Rc<RefCell<Common>>);

impl CommonRef {
    /// Creates a new `CommonRef` as an implied child of a `parent`.
    #[allow(clippy::wrong_self_convention)]
    #[inline(always)]
    pub fn new(parent: CommonRef) -> Self {
        CommonRef(Rc::new(RefCell::new(Common::new(parent))))
    }

    /// Returns a `Ref` to the inner `Common`.
    #[inline(always)]
    pub fn get(&self) -> Ref<'_, Common> {
        self.0.borrow()
    }

    /// Returns a `RefMut` to the inner `Common`.
    #[inline(always)]
    pub fn get_mut(&self) -> RefMut<'_, Common> {
        self.0.borrow_mut()
    }

    /// Returns a reference to the ref-counted `Common`.
    #[inline(always)]
    pub fn get_rc(&self) -> &Rc<RefCell<Common>> {
        &self.0
    }
}

/// The core, widget-agnostic object.
/// This should be stored within widgets via `Element`.
/// It handles the widget rectangle, parent, and other fundamental things.
#[derive(Debug, Clone, PartialEq)]
pub struct Common {
    rect: gfx::Rect,
    parent: CommonRef,
}

impl Common {
    /// Creates a new `Common` as an implied child of a `parent`.
    pub fn new(parent: CommonRef) -> Self {
        Common {
            rect: Default::default(),
            parent,
        }
    }

    /// Changes the widget rectangle.
    pub fn set_rect(&mut self, rect: gfx::Rect) {
        self.rect = rect;
    }

    /// Returns the widget rectangle.
    pub fn rect(&self) -> gfx::Rect {
        self.rect
    }

    /// Returns a `Ref` to the parent `Common`.
    pub fn parent(&self) -> Ref<'_, Common> {
        self.parent.get()
    }

    /// Returns a `RefMut` to the parent `Common`.
    pub fn parent_mut(&mut self) -> RefMut<'_, Common> {
        self.parent.get_mut()
    }
}

/// UI element trait, viewed as an extension of `Widget`.
pub trait Element: Widget + AnyElement {
    fn common(&self) -> CommonRef;
}

/// Conversions for `Element`s, from `Self` to various forms of `std::any::Any`.
/// # Note
/// **Do not manually implement** this trait. It is automatically implemented for all types that implement `Element`.
/// Simply implement `Element` and this will be automatically implemented.
pub trait AnyElement {
    /// Returns `self` as an immutable dynamic `Any` reference.
    fn as_any(&self) -> &dyn std::any::Any;
    /// Returns `self` as a mutable dynamic `Any` reference.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    /// Returns a `Boxed` `self` as a `Boxed` `Any`.
    fn as_any_box(self: Box<Self>) -> Box<dyn std::any::Any>;
}

impl<T: Element + 'static> AnyElement for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any_box(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

/// Altered version of `reclutch::widget::WidgetChildren` incorporating `Element`.
///
/// Refrain from implementing manually, instead use `#[derive(WidgetChildren] #[widget_children_trait(otway::ui::WidgetChildren)]`
pub trait WidgetChildren: Element + 'static {
    /// Returns a `Vec` of dynamic immutable children.
    fn children(
        &self,
    ) -> Vec<
        &dyn WidgetChildren<
            UpdateAux = Self::UpdateAux,
            GraphicalAux = Self::GraphicalAux,
            DisplayObject = Self::DisplayObject,
        >,
    > {
        Vec::new()
    }

    /// Returns a `Vec` of dynamic mutable children.
    fn children_mut(
        &mut self,
    ) -> Vec<
        &mut dyn WidgetChildren<
            UpdateAux = Self::UpdateAux,
            GraphicalAux = Self::GraphicalAux,
            DisplayObject = Self::DisplayObject,
        >,
    > {
        Vec::new()
    }
}

/// Helper type; `WidgetChildren` and `Aux`, with a given additional data type.
///
/// This reflects the primary widget type prevalent in the API.
pub type AuxWidgetChildren<T> = dyn WidgetChildren<
    UpdateAux = Aux<T>,
    GraphicalAux = Aux<T>,
    DisplayObject = gfx::DisplayCommand,
>;

/// Layout trait for pushing and removing widgets to a layout.
pub trait Layout: WidgetChildren {
    /// Additional per-widget configuration required by the implementing layout type.
    type Config;

    /// Adds a widget to this layout.
    fn push(&mut self, child: CommonRef, config: Self::Config);
    /// Removes a widget from this layout.
    fn remove(&mut self, child: CommonRef, config: Self::Config);
    /// Returns a boolean indicating the existence of a child within this layout (i.e. if it has been `pushed` and not `removed`).
    fn has(&self, child: &CommonRef) -> bool;
}

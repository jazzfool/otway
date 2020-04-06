pub mod view;

use {
    crate::theme::Theme,
    reclutch::{display as gfx, event::RcEventQueue, verbgraph as graph, widget::Widget},
    std::{
        cell::{Ref, RefCell, RefMut},
        ops::{Deref, DerefMut},
        rc::Rc,
    },
};

/// Global auxiliary type.
///
/// `T` generic is the additional data to be stored.
pub struct Aux<T: 'static> {
    pub data: T,
    pub theme: Box<dyn Theme<T>>,
    pub evq: RcEventQueue<WindowEvent>,
    pub master: sinq::MasterNodeRecord,
}

pub trait MasterEventRecord {
    fn master_mut(&mut self) -> &mut sinq::MasterNodeRecord;
}

impl<T: 'static> MasterEventRecord for Aux<T> {
    #[inline]
    fn master_mut(&mut self) -> &mut sinq::MasterNodeRecord {
        &mut self.master
    }
}

#[derive(Clone, Event)]
pub enum WindowEvent {
    #[event_key(mouse_press)]
    MousePress,
    #[event_key(mouse_release)]
    MouseRelease,
    #[event_key(key_press)]
    KeyPress,
    #[event_key(key_release)]
    KeyRelease,
}

/// Helper type to store a counted reference to a `Common`, or in other words, a reference to the core of a widget type (not the widget type itself).
#[derive(Debug, Clone, PartialEq)]
pub struct CommonRef(Rc<RefCell<Common>>);

impl CommonRef {
    /// Creates a new `CommonRef` without any parent.
    pub fn root() -> Self {
        CommonRef(Rc::new(RefCell::new(Common::root())))
    }

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
    parent: Option<CommonRef>,
}

impl Common {
    /// Creates a new `Common` as an implied root.
    pub fn root() -> Self {
        Common {
            rect: Default::default(),
            parent: None,
        }
    }

    /// Creates a new `Common` as an implied child of a `parent`.
    pub fn new(parent: CommonRef) -> Self {
        Common {
            rect: Default::default(),
            parent: Some(parent),
        }
    }

    /// Changes the widget rectangle.
    #[inline(always)]
    pub fn set_rect(&mut self, rect: gfx::Rect) {
        self.rect = rect;
    }

    /// Returns the widget rectangle.
    #[inline(always)]
    pub fn rect(&self) -> gfx::Rect {
        self.rect
    }

    /// Returns a `Ref` to the parent `Common`, if there is one.
    #[inline(always)]
    pub fn parent(&self) -> Option<Ref<'_, Common>> {
        Some(self.parent.as_ref()?.get())
    }

    /// Returns a `RefMut` to the parent `Common`, if there is one.
    #[inline(always)]
    pub fn parent_mut(&mut self) -> Option<RefMut<'_, Common>> {
        Some(self.parent.as_mut()?.get_mut())
    }
}

pub trait Node: Widget + Sized {
    type Event: graph::Event + 'static;

    fn node_ref(&self) -> &sinq::EventNode<Self, Self::UpdateAux, Self::Event>;
    fn node_mut(&mut self) -> &mut sinq::EventNode<Self, Self::UpdateAux, Self::Event>;
}

pub trait NodeExt: Widget {
    fn update_node(
        &mut self,
        aux: &mut Self::UpdateAux,
        node: sinq::NodeId,
        current_rec: usize,
        length: usize,
    ) -> Vec<sinq::EventRecord>;
    fn index(&self) -> Vec<sinq::NodeId>;
    fn current_record(&self) -> usize;
    fn finalize(&mut self, final_rec: usize);
}

impl<N: Widget + Node + 'static> NodeExt for N
where
    N::UpdateAux: MasterEventRecord,
{
    fn update_node(
        &mut self,
        aux: &mut Self::UpdateAux,
        node: sinq::NodeId,
        current_rec: usize,
        length: usize,
    ) -> Vec<sinq::EventRecord> {
        self.node_mut().set_record(Some(current_rec));
        let mut graph = self.node_mut().take();
        graph.update_node(self, aux, node, length);
        self.node_mut().reset(graph);
        self.node_mut().set_record(None);
        aux.master_mut().record()
    }

    #[inline]
    fn index(&self) -> Vec<sinq::NodeId> {
        self.node_ref().subjects()
    }

    #[inline]
    fn current_record(&self) -> usize {
        self.node_ref().final_record()
    }

    #[inline]
    fn finalize(&mut self, final_rec: usize) {
        self.node_mut().set_final_record(final_rec);
    }
}

fn update_invoker<T: 'static>(
    widget: &mut &mut dyn WidgetChildren<
        UpdateAux = Aux<T>,
        GraphicalAux = Aux<T>,
        DisplayObject = gfx::DisplayCommand,
    >,
    aux: &mut Aux<T>,
    node: sinq::NodeId,
    current_rec: usize,
    length: usize,
) -> Vec<sinq::EventRecord> {
    widget.update_node(aux, node, current_rec, length)
}

fn update_indexer<T: 'static>(
    widget: &&mut dyn WidgetChildren<
        UpdateAux = Aux<T>,
        GraphicalAux = Aux<T>,
        DisplayObject = gfx::DisplayCommand,
    >,
) -> Vec<sinq::NodeId> {
    widget.index()
}

fn update_recorder<T: 'static>(
    widget: &&mut dyn WidgetChildren<
        UpdateAux = Aux<T>,
        GraphicalAux = Aux<T>,
        DisplayObject = gfx::DisplayCommand,
    >,
) -> usize {
    widget.current_record()
}

fn update_finalizer<T: 'static>(
    widget: &mut &mut dyn WidgetChildren<
        UpdateAux = Aux<T>,
        GraphicalAux = Aux<T>,
        DisplayObject = gfx::DisplayCommand,
    >,
    final_rec: usize,
) {
    widget.finalize(final_rec)
}

pub fn update<T: 'static>(
    widget: &mut impl WidgetChildren<
        UpdateAux = Aux<T>,
        GraphicalAux = Aux<T>,
        DisplayObject = gfx::DisplayCommand,
    >,
    aux: &mut Aux<T>,
) {
    {
        let mut items = widget.children_mut();
        let rec = aux.master.record();
        sinq::update(
            &mut items,
            aux,
            rec,
            update_invoker::<T>,
            update_indexer::<T>,
            update_recorder::<T>,
            update_finalizer::<T>,
        );
    }

    let rec = aux.master.record();
    sinq::update(
        &mut [widget
            as &mut dyn WidgetChildren<
                UpdateAux = Aux<T>,
                GraphicalAux = Aux<T>,
                DisplayObject = gfx::DisplayCommand,
            >],
        aux,
        rec,
        update_invoker::<T>,
        update_indexer::<T>,
        update_recorder::<T>,
        update_finalizer::<T>,
    );
}

pub fn propagate_update<T: 'static>(
    widget: &mut impl WidgetChildren<
        UpdateAux = Aux<T>,
        GraphicalAux = Aux<T>,
        DisplayObject = gfx::DisplayCommand,
    >,
    aux: &mut Aux<T>,
) {
    for child in widget.children_mut() {
        child.update(aux);
    }
}

/// An event that will never be emitted.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[event_key(none)]
pub struct NoEvent(
    /* to prevent instantiation */ std::marker::PhantomData<()>,
);

/// UI element trait, viewed as an extension of `Widget`.
pub trait Element: Widget + AnyElement + NodeExt {
    fn common(&self) -> &CommonRef;
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

/// `CommandGroup` compatible with the `draw` function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandGroup(Option<gfx::CommandGroup>);

impl Default for CommandGroup {
    fn default() -> Self {
        CommandGroup(Some(Default::default()))
    }
}

impl Deref for CommandGroup {
    type Target = gfx::CommandGroup;

    fn deref(&self) -> &gfx::CommandGroup {
        self.0.as_ref().unwrap()
    }
}

impl DerefMut for CommandGroup {
    fn deref_mut(&mut self) -> &mut gfx::CommandGroup {
        self.0.as_mut().unwrap()
    }
}

/// Widget drawing helper function which handles ownership.
pub fn draw<T: 'static, W: WidgetChildren>(
    obj: &mut W,
    cmds_fn: impl Fn(&mut W) -> &mut CommandGroup,
    draw_fn: impl FnOnce(&mut W, &mut Aux<T>) -> Vec<gfx::DisplayCommand>,
    display: &mut dyn gfx::GraphicsDisplay,
    aux: &mut Aux<T>,
) -> Option<()> {
    let mut cmds = cmds_fn(obj).0.take()?;
    cmds.push_with(
        display,
        || draw_fn(obj, aux),
        Default::default(),
        None,
        None,
    );
    cmds_fn(obj).0 = Some(cmds);

    Some(())
}

/// Keyboard modifier keys state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
}

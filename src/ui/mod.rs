pub mod view;

use {
    crate::theme::Theme,
    reclutch::{display as gfx, verbgraph as graph, widget::Widget},
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
    /// Additional miscellaneous data
    pub data: T,
    /// Current application theme.
    pub theme: Box<dyn Theme<T>>,
    /// Out-going only node. Emits OS events relating to the window.
    // TODO(jazzfool): use a collection to prepare for multiple window support.
    pub node: sinq::EventNode<(), (), WindowEvent>,
    /// Master event record, storing a timeline of all events emitted from within the application.
    /// This facilitates event synchronization.
    pub master: sinq::MasterNodeRecord,
}

/// Auxiliary type containing a master event record.
pub trait MasterEventRecord {
    fn master_mut(&mut self) -> &mut sinq::MasterNodeRecord;
}

impl<T: 'static> MasterEventRecord for Aux<T> {
    #[inline]
    fn master_mut(&mut self) -> &mut sinq::MasterNodeRecord {
        &mut self.master
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ConsumableEventInner<T> {
    marker: RefCell<bool>,
    data: T,
}

/// Event data that can be "consumed". This is needed for events such as clicking and typing.
/// Those kinds of events aren't typically received by multiple widgets.
///
/// As an example of this, say you have multiple buttons stacked atop each other.
/// When you click that stack of buttons, only the one on top should receive the click event,
/// as in, the event is *consumed*.
///
/// Note that this primitive isn't very strict. The consumption conditions can be bypassed
/// in case the data needs to be accessed regardless of state, and the predicate can be
/// exploited to use the data without consuming it.
///
/// Also note that the usage of "consume" is completely unrelated to the consume/move
/// semantics of Rust. In fact, nothing is actually consumed in this implementation.
#[derive(Debug, PartialEq)]
pub struct ConsumableEvent<T>(Rc<ConsumableEventInner<T>>);

impl<T> ConsumableEvent<T> {
    /// Creates a unconsumed event, initialized with `val`.
    pub fn new(val: T) -> Self {
        ConsumableEvent(Rc::new(ConsumableEventInner {
            marker: RefCell::new(true),
            data: val,
        }))
    }

    /// Returns the event data as long as **both** the following conditions are satisfied:
    /// 1. The event hasn't been consumed yet.
    /// 2. The predicate returns true.
    ///
    /// The point of the predicate is to let the caller see if the event actually applies
    /// to them before consuming needlessly.
    pub fn with<P>(&self, mut pred: P) -> Option<&T>
    where
        P: FnMut(&T) -> bool,
    {
        let mut is_consumed = self.0.marker.borrow_mut();
        if *is_consumed && pred(&self.0.data) {
            *is_consumed = false;
            Some(&self.0.data)
        } else {
            None
        }
    }

    /// Returns the inner event data regardless of consumption.
    #[inline(always)]
    pub fn get(&self) -> &T {
        &self.0.data
    }
}

impl<T> Clone for ConsumableEvent<T> {
    fn clone(&self) -> Self {
        ConsumableEvent(self.0.clone())
    }
}

/// An event emitted by the OS relating to the window.
#[derive(Clone, Event)]
pub enum WindowEvent {
    /// A mouse button was pressed down.
    #[event_key(mouse_press)]
    MousePress(ConsumableEvent<(MouseButton, gfx::Point)>),
    /// A mouse button was releasd. Always paired with a prior `MousePress`.
    #[event_key(mouse_release)]
    MouseRelease(ConsumableEvent<(MouseButton, gfx::Point)>),
    /// The mouse/cursor was moved.
    #[event_key(mouse_move)]
    MouseMove(ConsumableEvent<gfx::Point>),
    /// A keyboard key was pressed down.
    #[event_key(key_press)]
    KeyPress(ConsumableEvent<KeyInput>),
    /// A keyboard key was released. Always paired with a prior `KeyPress`.
    #[event_key(key_release)]
    KeyRelease(ConsumableEvent<KeyInput>),
    /// Printable character was typed. Related to string input.
    #[event_key(text)]
    Text(ConsumableEvent<char>),
}

/// Clickable button on a mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Other(u8),
}

macro_rules! keyboard_enum {
    ($name:ident as $other:ty {
        $($v:ident),*$(,)?
    }) => {
        #[doc = "Key on a keyboard."]
        #[repr(u32)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $($v),*
        }

        #[cfg(feature = "app")]
        impl From<$other> for $name {
            fn from(other: $other) -> $name {
                match other {
                    $(<$other>::$v => $name::$v),*
                }
            }
        }

        #[cfg(feature = "app")]
        impl Into<$other> for $name {
            fn into(self) -> $other {
                match self {
                    $($name::$v => <$other>::$v),*
                }
            }
        }
    };
}

keyboard_enum! {
    KeyInput as glutin::event::VirtualKeyCode {
        Key1,
        Key2,
        Key3,
        Key4,
        Key5,
        Key6,
        Key7,
        Key8,
        Key9,
        Key0,
        A,
        B,
        C,
        D,
        E,
        F,
        G,
        H,
        I,
        J,
        K,
        L,
        M,
        N,
        O,
        P,
        Q,
        R,
        S,
        T,
        U,
        V,
        W,
        X,
        Y,
        Z,
        Escape,
        F1,
        F2,
        F3,
        F4,
        F5,
        F6,
        F7,
        F8,
        F9,
        F10,
        F11,
        F12,
        F13,
        F14,
        F15,
        F16,
        F17,
        F18,
        F19,
        F20,
        F21,
        F22,
        F23,
        F24,
        Snapshot,
        Scroll,
        Pause,
        Insert,
        Home,
        Delete,
        End,
        PageDown,
        PageUp,
        Left,
        Up,
        Right,
        Down,
        Back,
        Return,
        Space,
        Compose,
        Caret,
        Numlock,
        Numpad0,
        Numpad1,
        Numpad2,
        Numpad3,
        Numpad4,
        Numpad5,
        Numpad6,
        Numpad7,
        Numpad8,
        Numpad9,
        AbntC1,
        AbntC2,
        Add,
        Apostrophe,
        Apps,
        At,
        Ax,
        Backslash,
        Calculator,
        Capital,
        Colon,
        Comma,
        Convert,
        Decimal,
        Divide,
        Equals,
        Grave,
        Kana,
        Kanji,
        LAlt,
        LBracket,
        LControl,
        LShift,
        LWin,
        Mail,
        MediaSelect,
        MediaStop,
        Minus,
        Multiply,
        Mute,
        MyComputer,
        NavigateForward,
        NavigateBackward,
        NextTrack,
        NoConvert,
        NumpadComma,
        NumpadEnter,
        NumpadEquals,
        OEM102,
        Period,
        PlayPause,
        Power,
        PrevTrack,
        RAlt,
        RBracket,
        RControl,
        RShift,
        RWin,
        Semicolon,
        Slash,
        Sleep,
        Stop,
        Subtract,
        Sysrq,
        Tab,
        Underline,
        Unlabeled,
        VolumeDown,
        VolumeUp,
        Wake,
        WebBack,
        WebFavorites,
        WebForward,
        WebHome,
        WebRefresh,
        WebSearch,
        WebStop,
        Yen,
        Copy,
        Paste,
        Cut,
    }
}

/// Helper type to store a counted reference to a `Common`, or in other words, a reference to the core of a widget type (not the widget type itself).
#[derive(Debug, Clone, PartialEq)]
pub struct CommonRef(Rc<RefCell<Common>>);

impl CommonRef {
    /// Creates a new `CommonRef` as an implied child of a `parent`.
    #[allow(clippy::wrong_self_convention)]
    #[inline(always)]
    pub fn new(parent: impl Into<Option<CommonRef>>) -> Self {
        CommonRef(Rc::new(RefCell::new(Common::new(parent))))
    }

    /// Returns a `Ref` to the inner `Common`.
    #[inline(always)]
    pub fn get_ref(&self) -> Ref<'_, Common> {
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

/// Contains the interaction state for a single widget.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Interaction {
    pub(crate) pressed: bool,
    pub(crate) hovered: bool,
}

impl Interaction {
    /// Returns true if the widget has been pressed down by the cursor.
    #[inline]
    pub fn pressed(&self) -> bool {
        self.pressed
    }

    /// Returns true if the widget has been hovered over by the cursor.
    #[inline]
    pub fn hovered(&self) -> bool {
        self.hovered
    }
}

/// The core, widget-agnostic object.
/// This should be stored within widgets via `Element`.
/// It handles the widget rectangle, parent, and other fundamental things.
#[derive(Debug, Clone, PartialEq)]
pub struct Common {
    pub(crate) interaction: Interaction,
    visible: bool,
    updates: bool,
    rect: gfx::Rect,
    parent: Option<CommonRef>,
    cmds: CommandGroup,
}

impl Common {
    /// Creates a new `Common`.
    /// If `None` is given to `parent`, it implies that this widget is a root widget.
    pub fn new(parent: impl Into<Option<CommonRef>>) -> Self {
        Common {
            interaction: Interaction::default(),
            visible: true,
            updates: true,
            rect: Default::default(),
            parent: parent.into(),
            cmds: Default::default(),
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

    /// Changes the widget rectangle size.
    #[inline]
    pub fn set_size(&mut self, size: gfx::Size) {
        self.rect.size = size;
    }

    /// Returns the widget rectangle size.
    #[inline]
    pub fn size(&self) -> gfx::Size {
        self.rect.size
    }

    /// Sets the visibility for this widget.
    ///
    /// If `false`, this widget will be excluded from rendering.
    #[inline]
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Returns the visibility for this widget.
    #[inline]
    pub fn visible(&self) -> bool {
        self.visible
    }

    /// Sets the updating mode for this widget.
    ///
    /// If `false`, this widget will be excluded from updates (will not be able to handle events).
    #[inline]
    pub fn set_updates(&mut self, updates: bool) {
        self.updates = updates;
    }

    /// Returns the updating mode for this widget.
    #[inline]
    pub fn updates(&self) -> bool {
        self.updates
    }

    /// Returns a `Ref` to the parent `Common`, if there is one.
    #[inline]
    pub fn parent(&self) -> Option<Ref<'_, Common>> {
        Some(self.parent.as_ref()?.get_ref())
    }

    /// Returns a `RefMut` to the parent `Common`, if there is one.
    #[inline]
    pub fn parent_mut(&mut self) -> Option<RefMut<'_, Common>> {
        Some(self.parent.as_mut()?.get_mut())
    }

    #[inline]
    pub fn command_group(&self) -> &CommandGroup {
        &self.cmds
    }

    #[inline]
    pub fn command_group_mut(&mut self) -> &mut CommandGroup {
        &mut self.cmds
    }
}

/// Widget which act as an event node, i.e. have in-going and out-going connections to other nodes.
pub trait Node: Widget + Sized {
    type Event: graph::Event + 'static;

    /// Returns an immutable reference to the widget's event node.
    fn node_ref(&self) -> &sinq::EventNode<Self, Self::UpdateAux, Self::Event>;
    /// Returns a mutable reference to the widget's event node.
    fn node_mut(&mut self) -> &mut sinq::EventNode<Self, Self::UpdateAux, Self::Event>;
}

/// Extension functions required by `update`.
///
/// **Do not implement manually**, implement `Node` instead.
/// You do not need to worry about this trait.
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
        aux.master_mut().record().to_vec()
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

/// Updates the nodes of the direct children of a widget.
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
        items.retain(|x| x.common().get_ref().updates());
        let rec = aux.master.record().to_vec();
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

    let rec = aux.master.record().to_vec();
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

/// Recursively propagate the `update` method.
pub fn propagate_update<T: 'static>(
    widget: &mut dyn WidgetChildren<
        UpdateAux = Aux<T>,
        GraphicalAux = Aux<T>,
        DisplayObject = gfx::DisplayCommand,
    >,
    aux: &mut Aux<T>,
) {
    for child in widget.children_mut() {
        propagate_update(child, aux);
    }

    widget.update(aux);
}

/// Recursively propagate the `draw` method.
pub fn propagate_draw<T: 'static>(
    widget: &mut dyn WidgetChildren<
        UpdateAux = Aux<T>,
        GraphicalAux = Aux<T>,
        DisplayObject = gfx::DisplayCommand,
    >,
    display: &mut dyn gfx::GraphicsDisplay,
    aux: &mut Aux<T>,
) {
    for child in widget.children_mut().into_iter().rev() {
        propagate_draw(child, display, aux);
    }

    if widget.common().get_ref().visible() {
        widget.draw(display, aux);
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
    draw_fn: impl FnOnce(&mut W, &mut Aux<T>) -> Vec<gfx::DisplayCommand>,
    display: &mut dyn gfx::GraphicsDisplay,
    aux: &mut Aux<T>,
) {
    let mut cmds = obj.common().get_mut().command_group_mut().0.take().unwrap();
    cmds.push_with(
        display,
        || draw_fn(obj, aux),
        Default::default(),
        None,
        None,
    );
    obj.common().get_mut().command_group_mut().0 = Some(cmds);
}

/// Keyboard modifier keys state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
}

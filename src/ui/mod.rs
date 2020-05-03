pub mod view;

use {
    crate::theme::Theme,
    reclutch::{display as gfx, widget::Widget},
    std::{
        cell::Cell,
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
    /// Queue event ID.
    pub id: u64,
    /// Global queue.
    pub queue: uniq::rc::Queue,
}

impl<T: 'static> Aux<T> {
    /// Creates a new [`Listener`](Listener).
    pub fn listen<O: 'static, A: 'static>(&self) -> Listener<O, A> {
        Listener(Some(self.queue.listen()))
    }

    pub fn emit<E: 'static>(&self, id: &impl Id, e: E) {
        self.queue.emit(id.id(), e);
    }
}

/// Listener compatible with the [`dispatch`](dispatch) function.
///
/// Created via [`listen`](Aux::listen).
pub struct Listener<O: 'static, A: 'static>(Option<uniq::rc::EventListener<O, A>>);

impl<O: 'static, A: 'static> Listener<O, A> {
    /// Adds a handler to `self` and returns `Self`.
    ///
    /// `id` marks the source ID. The type of the third parameter of the handler is the event type.
    /// Both of these will be used to match correct events.
    ///
    /// If the ID and event type are already being handled, the handler will be replaced.
    pub fn and_on<E: 'static>(
        mut self,
        id: u64,
        handler: impl FnMut(&mut O, &mut A, &E) + 'static,
    ) -> Self {
        self.0.as_mut().unwrap().on(id, handler);
        self
    }

    /// Adds a handler.
    ///
    /// `id` marks the source ID. The type of the third parameter of the handler is the event type.
    /// Both of these will be used to match correct events.
    ///
    /// If the ID and event type are already being handled, the handler will be replaced.
    pub fn on<E: 'static>(
        &mut self,
        id: u64,
        handler: impl FnMut(&mut O, &mut A, &E) + 'static,
    ) -> (u64, std::any::TypeId) {
        self.0.as_mut().unwrap().on(id, handler)
    }

    /// Removes a handler which matches a specific `id` and event type.
    pub fn remove<E: 'static>(&mut self, id: u64) -> bool {
        self.0.as_mut().unwrap().remove::<E>(id)
    }

    /// Returns `true` if there is a handler handling `id` and event type `E`.
    pub fn contains<E: 'static>(&self, id: u64) -> bool {
        self.0.as_ref().unwrap().contains::<E>(id)
    }
}

/// Dispatches the event handlers in a [`Listener`](Listener).
pub fn dispatch<O: 'static, A: 'static>(
    o: &mut O,
    a: &mut A,
    l: impl Fn(&mut O) -> &mut Listener<O, A>,
) {
    let mut ll = l(o).0.take().unwrap();
    ll.dispatch(o, a);
    l(o).0 = Some(ll);
}

#[derive(Clone, Debug, PartialEq)]
struct ConsumableEventInner<T> {
    marker: Cell<bool>,
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
            marker: Cell::new(true),
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
        if self.0.marker.get() && pred(&self.0.data) {
            self.0.marker.set(false);
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

/// A mouse button was pressed down.
pub struct MousePressEvent(pub ConsumableEvent<(MouseButton, gfx::Point)>);
/// A mouse button was releasd. Always paired with a prior `MousePressEvent`.
pub struct MouseReleaseEvent(pub ConsumableEvent<(MouseButton, gfx::Point)>);
/// The mouse/cursor was moved.
pub struct MouseMoveEvent(pub ConsumableEvent<gfx::Point>);
/// A keyboard key was pressed down.
pub struct KeyPressEvent(pub ConsumableEvent<KeyInput>);
/// A keyboard key was released. Always paired with a prior `KeyPressEvent`.
pub struct KeyReleaseEvent(pub ConsumableEvent<KeyInput>);
/// Printable character was typed. Related to string input.
pub struct TextEvent(pub ConsumableEvent<char>);

/// Clickable button on a mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Other(u8),
}

// it's either this or `mem::transmute`
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

/// Partial function application; returns a closure that fills in one additional parameter in order to
/// conform to standard widget constructor signature.
pub fn f1<
    A,
    P,
    W: WidgetChildren<UpdateAux = A, GraphicalAux = A, DisplayObject = gfx::DisplayCommand>,
>(
    a: impl FnOnce(CommonRef, &mut A, P) -> W,
    p: P,
) -> impl FnOnce(CommonRef, &mut A) -> W {
    move |x, y| a(x, y, p)
}

/// Partial function application; returns a closure that fills in two additional parameters in order to
/// conform to standard widget constructor signature.
pub fn f2<
    A,
    P1,
    P2,
    W: WidgetChildren<UpdateAux = A, GraphicalAux = A, DisplayObject = gfx::DisplayCommand>,
>(
    a: impl FnOnce(CommonRef, &mut A, P1, P2) -> W,
    p1: P1,
    p2: P2,
) -> impl FnOnce(CommonRef, &mut A) -> W {
    move |x, y| a(x, y, p1, p2)
}

/// Partial function application; returns a closure that fills in three additional parameters in order to
/// conform to standard widget constructor signature.
pub fn f3<
    A,
    P1,
    P2,
    P3,
    W: WidgetChildren<UpdateAux = A, GraphicalAux = A, DisplayObject = gfx::DisplayCommand>,
>(
    a: impl FnOnce(CommonRef, &mut A, P1, P2, P3) -> W,
    p1: P1,
    p2: P2,
    p3: P3,
) -> impl FnOnce(CommonRef, &mut A) -> W {
    move |x, y| a(x, y, p1, p2, p3)
}

/// Helper type to store a counted reference to a `Common`, or in other words, a reference to the core of a widget type (not the widget type itself).
///
/// The reference type provides `RefCell`-like semantics using `Cell`, reducing the overhead to only `Rc` instead of `Rc` + `RefCell`.
/// It does this by `take` and `replace`, inserted around accessor closures.
#[derive(Clone)]
pub struct CommonRef(Rc<Cell<Option<Common>>>);

impl CommonRef {
    /// Creates a new `CommonRef` as an implied child of a `parent`.
    #[inline]
    pub fn new(parent: impl Into<Option<CommonRef>>) -> Self {
        CommonRef(Rc::new(Cell::new(Some(Common::new(parent)))))
    }

    // Creates a new `CommonRef` as an implied child of a `parent` with some additional `info`.
    #[inline]
    pub fn with_info(
        parent: impl Into<Option<CommonRef>>,
        info: impl Into<Option<Box<dyn std::any::Any>>>,
    ) -> Self {
        CommonRef(Rc::new(Cell::new(Some(Common::with_info(parent, info)))))
    }

    /// Mutably access the inner `Common` through a closure.
    /// The return value of the closure is forwarded to the caller.
    ///
    /// This can be used to extract certain values or mutate, or both.
    pub fn with<R>(&self, f: impl FnOnce(&mut Common) -> R) -> R {
        let mut common = self.0.take().unwrap();
        let r = f(&mut common);
        self.0.replace(Some(common));
        r
    }

    /// Returns a reference to the ref-counted `Common`.
    #[inline]
    pub fn get_rc(&self) -> &Rc<Cell<Option<Common>>> {
        &self.0
    }

    /// Emits an event to the global queue on the behalf of `common.id()`.
    #[inline]
    pub fn emit<T: 'static, E: 'static>(&self, aux: &mut Aux<T>, event: E) {
        self.with(|x| x.emit(aux, event));
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
///
/// Moreover, it can also possibly contain additional information (accessed through `info()/_mut`).
/// The information is stored in an `Option<Box<dyn Any>>`. It serves the purpose of passing information
/// between arbitrary widgets without using event queues as a means of data transfer.
/// This information can be initialized (only once) by constructing `with_info`.
pub struct Common {
    pub(crate) interaction: Interaction,
    visible: bool,
    updates: bool,
    rect: gfx::Rect,
    parent: Option<CommonRef>,
    cmds: CommandGroup,
    id: u64,
    info: Option<Box<dyn std::any::Any>>,
}

impl Common {
    /// Creates a new `Common` without additional information.
    #[inline]
    pub fn new(parent: impl Into<Option<CommonRef>>) -> Self {
        Common::with_info(parent, None)
    }

    /// Creates a new `Common` with additional `info`.
    /// If `None` is given to `parent`, it implies that this widget is a root widget.
    ///
    /// If passing `None` to `info` then use [`Common::new`](Common::new) instead.
    pub fn with_info(
        parent: impl Into<Option<CommonRef>>,
        info: impl Into<Option<Box<dyn std::any::Any>>>,
    ) -> Self {
        Common {
            interaction: Interaction::default(),
            visible: true,
            updates: true,
            rect: Default::default(),
            parent: parent.into(),
            cmds: Default::default(),
            id: uniq::id::next(),
            info: info.into(),
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

    /// Returns a reference to the parent `Common`.
    ///
    /// If `None` is returned then this is the root `Common`.
    #[inline]
    pub fn parent(&self) -> Option<CommonRef> {
        self.parent.clone()
    }

    /// Returns the display command group immutably.
    #[inline]
    pub fn command_group(&self) -> &CommandGroup {
        &self.cmds
    }

    /// Returns the display command group mutably.
    #[inline]
    pub fn command_group_mut(&mut self) -> &mut CommandGroup {
        &mut self.cmds
    }

    /// Emits an event to the global queue on the behalf of [`id`](Common::id).
    #[inline]
    pub fn emit<T: 'static, E: 'static>(&self, aux: &mut Aux<T>, event: E) {
        aux.queue.emit(self.id, event);
    }

    /// Returns the possible stored information immutably.
    ///
    /// If the information has not been provided, or the downcast type mismatches, `None` is returned.
    #[inline]
    pub fn info<T: 'static>(&self) -> Option<&T> {
        self.info.as_ref()?.as_ref().downcast_ref::<T>()
    }

    /// Returns the possible stored information mutably.
    ///
    /// If the information has not been provided, or the downcast type mismatches, `None` is returned.
    #[inline]
    pub fn info_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.info.as_mut()?.as_mut().downcast_mut::<T>()
    }

    /// Returns `true` if there is additional information matching the given type, otherwise `false`.
    #[inline]
    pub fn info_is_type<T: 'static>(&self) -> bool {
        self.info
            .as_ref()
            .map(|x| x.type_id() == std::any::TypeId::of::<T>())
            .unwrap_or(false)
    }

    /// Performs an upward search of the (grand)parents using a given predicate and returns a possible match.
    /// The search will continue upwards until a match is found or the root widget (which has no parent) is reached.
    ///
    /// `max_distance` is the maximum distance that the search will go. This can be `None` or a `usize`.
    /// For example, `max_distance: 3` will only search up to 3 parents. The fourth grandparent and onwards will not be searched.
    ///
    /// Note: This does not consider `self`.
    pub fn find_parent(
        &self,
        mut pred: impl FnMut(&Common) -> bool,
        max_distance: impl Into<Option<usize>> + Copy,
    ) -> Option<CommonRef> {
        if max_distance.into().map(|x| x == 0).unwrap_or(false) {
            None
        } else {
            self.parent().and_then(move |x| {
                if x.with(|x| pred(x)) {
                    Some(x)
                } else {
                    x.with(|x| x.find_parent(pred, max_distance.into().map(|x| x - 1)))
                }
            })
        }
    }
}

impl Id for Common {
    /// Returns the unique ID assigned to this `Common`.
    /// It is unique across all `Common` and is primarily used as an event source ID for the global queue.
    #[inline]
    fn id(&self) -> u64 {
        self.id
    }
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
    if widget.common().with(|c| c.visible()) {
        widget.draw(display, aux);
    }

    for child in widget.children_mut().into_iter().rev() {
        propagate_draw(child, display, aux);
    }
}

pub trait Id {
    fn id(&self) -> u64;
}

impl Id for u64 {
    #[inline]
    fn id(&self) -> u64 {
        *self
    }
}

/// UI element trait, viewed as an extension of `Widget`.
pub trait Element: Widget + AnyElement {
    fn common(&self) -> &CommonRef;
}

impl<E: Element + ?Sized> Id for E {
    #[inline]
    fn id(&self) -> u64 {
        self.common().with(|x| x.id())
    }
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
    let mut cmds = obj
        .common()
        .with(|x| x.command_group_mut().0.take().unwrap());
    cmds.push_with(
        display,
        || draw_fn(obj, aux),
        Default::default(),
        None,
        None,
    );
    obj.common().with(|x| x.command_group_mut().0 = Some(cmds));
}

/// Keyboard modifier keys state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
}

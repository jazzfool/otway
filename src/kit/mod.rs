use {
    crate::ui::{self, layout, ElementMixin},
    reclutch::display as gfx,
};

pub mod button;
pub mod check_box;
pub mod combo_box;
pub mod label;
pub mod text_box;

pub use {button::*, check_box::*, combo_box::*, label::*, text_box::*};

/// The widget was pressed.
#[repr(transparent)]
pub struct PressEvent(pub gfx::Point);
/// The widget was released from its press ([`PressEvent`](PressEvent)).
#[repr(transparent)]
pub struct ReleaseEvent(pub gfx::Point);
/// The cursor entered the widget boundaries.
#[repr(transparent)]
pub struct BeginHoverEvent(pub gfx::Point);
/// The cursor left the widget boundaries.
#[repr(transparent)]
pub struct EndHoverEvent(pub gfx::Point);

pub struct FocusGainedEvent;
pub struct FocusLostEvent;

#[repr(transparent)]
pub struct KeyPressEvent(pub ui::KeyInput);
#[repr(transparent)]
pub struct KeyReleaseEvent(pub ui::KeyInput);
#[repr(transparent)]
pub struct TextEvent(pub char);

/// Standard set of listener read/writes: `&mut Widget` and `&mut Aux`.
pub type ReadWrite<E> = (ui::Write<E>, ui::Write<ui::Aux<<E as ui::Element>::Aux>>);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InteractionEvent {
    Press(gfx::Point),
    Release(gfx::Point),
    BeginHover(gfx::Point),
    EndHover(gfx::Point),
}

pub struct InteractionState<
    T: 'static,
    W: ui::WidgetChildren<T>,
    F: FnMut(&mut W, &mut ui::Aux<T>, InteractionEvent) + 'static,
> {
    pressed: bool,
    hovered: bool,

    listener: ui::Listener<(ui::Write<W>, ui::Write<Self>, ui::Write<ui::Aux<T>>)>,
    callback: F,
    mask: InteractionMask,
    ignore_vis: bool,

    phantom: std::marker::PhantomData<(T, W)>,
}

impl<
        T: 'static,
        W: ui::WidgetChildren<T>,
        F: FnMut(&mut W, &mut ui::Aux<T>, InteractionEvent) + 'static,
    > InteractionState<T, W, F>
{
    pub fn new(
        aux: &mut ui::Aux<T>,
        callback: F,
        mask: impl Into<Option<InteractionMask>>,
        ignore_visibility: impl Into<Option<bool>>,
    ) -> Self {
        InteractionState {
            pressed: false,
            hovered: false,

            listener: aux
                .listen::<(ui::Write<W>, ui::Write<Self>, ui::Write<ui::Aux<T>>)>()
                .and_on(aux.id, |(obj, state, aux), ev: &ui::MousePressEvent| {
                    if !state.mask.press {
                        return;
                    }
                    let v = obj.visible();
                    if !state.ignore_vis && invisible_to_input(v) {
                        return;
                    }

                    let bounds = obj.bounds();
                    if let Some(&(_, pos)) = ev
                        .0
                        .with(|&(btn, pos)| btn == ui::MouseButton::Left && bounds.contains(pos))
                    {
                        state.pressed = true;
                        (state.callback)(obj, aux, InteractionEvent::Press(pos));
                    }
                })
                .and_on(
                    aux.id,
                    move |(obj, state, aux), ev: &ui::MouseReleaseEvent| {
                        if !state.mask.release {
                            return;
                        }
                        let v = obj.visible();
                        if !state.ignore_vis && invisible_to_input(v) {
                            return;
                        }

                        // FIXME: release applies when pressed, not when mouse is in bounds

                        let bounds = obj.bounds();
                        if let Some(&(_, pos)) = ev.0.with(|&(btn, pos)| {
                            btn == ui::MouseButton::Left && bounds.contains(pos)
                        }) {
                            state.pressed = false;
                            (state.callback)(obj, aux, InteractionEvent::Release(pos));
                        }
                    },
                )
                .and_on(aux.id, move |(obj, state, aux), ev: &ui::MouseMoveEvent| {
                    if !state.mask.hover {
                        return;
                    }
                    let v = obj.visible();
                    if !state.ignore_vis && invisible_to_input(v) {
                        return;
                    }

                    let bounds = obj.bounds();
                    let was_hovered = state.hovered;
                    let pos = if let Some(&pos) = ev.0.with(|&pos| bounds.contains(pos)) {
                        state.hovered = true;
                        pos
                    } else {
                        state.hovered = false;
                        ev.0.get().clone()
                    };

                    if was_hovered != state.hovered {
                        if was_hovered {
                            (state.callback)(obj, aux, InteractionEvent::EndHover(pos));
                        } else {
                            (state.callback)(obj, aux, InteractionEvent::BeginHover(pos));
                        }
                    }
                }),
            callback,
            mask: mask.into().unwrap_or_default(),
            ignore_vis: ignore_visibility.into().unwrap_or(false),

            phantom: Default::default(),
        }
    }
}

impl<
        T: 'static,
        W: ui::WidgetChildren<T>,
        F: FnMut(&mut W, &mut ui::Aux<T>, InteractionEvent) + 'static,
    > ui::Component for InteractionState<T, W, F>
{
    type Type = T;
    type Object = W;

    fn update(&mut self, obj: &mut Self::Object, aux: &mut ui::Aux<Self::Type>) {
        ui::dispatch((obj, self, aux), |x: (_, &mut Self, _)| &mut x.1.listener);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionMask {
    pub press: bool,
    pub release: bool,
    pub hover: bool,
}

impl Default for InteractionMask {
    fn default() -> Self {
        InteractionMask {
            press: true,
            release: true,
            hover: true,
        }
    }
}

pub fn interaction_forwarder<E: ui::Element<Aux = T>, T: 'static>(
    mask: impl Into<Option<InteractionMask>>,
) -> impl Fn(&mut E, &mut ui::Aux<T>, InteractionEvent) + Copy {
    let mask = mask.into().unwrap_or(Default::default());
    move |obj, aux, event| match event {
        InteractionEvent::Press(pos) => {
            if mask.press {
                obj.emit(aux, PressEvent(pos));
            }
        }
        InteractionEvent::Release(pos) => {
            if mask.release {
                obj.emit(aux, ReleaseEvent(pos));
            }
        }
        InteractionEvent::BeginHover(pos) => {
            if mask.hover {
                obj.emit(aux, BeginHoverEvent(pos));
            }
        }
        InteractionEvent::EndHover(pos) => {
            if mask.hover {
                obj.emit(aux, EndHoverEvent(pos));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FocusEvent {
    Gained,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FocusMouseTrigger {
    Press,
    Release,
}

impl Default for FocusMouseTrigger {
    #[inline]
    fn default() -> Self {
        FocusMouseTrigger::Press
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FocusConfig {
    pub mouse_trigger: FocusMouseTrigger,
    /// The event ID of the widget which emits interaction events.
    /// This should almost always be the same ID as the the widget which is attaching the `focus_handler`.
    pub interaction_handler: u64,
}

pub fn focus_handler<T, W: ui::WidgetChildren<T>>(
    aux: &mut ui::Aux<T>,
    callback: impl Fn(&mut W, &mut ui::Aux<T>, FocusEvent) + Copy + 'static,
    focus_config: FocusConfig,
) -> ui::Listener<(ui::Write<W>, ui::Write<ui::Aux<T>>)> {
    aux.listen::<(ui::Write<W>, ui::Write<ui::Aux<T>>)>()
        .and_on(
            focus_config.interaction_handler,
            move |(obj, aux), _: &PressEvent| {
                if focus_config.mouse_trigger == FocusMouseTrigger::Press {
                    aux.grab_focus(obj.common().clone());
                }
            },
        )
        .and_on(
            focus_config.interaction_handler,
            move |(obj, aux), _: &ReleaseEvent| {
                if focus_config.mouse_trigger == FocusMouseTrigger::Release {
                    aux.grab_focus(obj.common().clone());
                }
            },
        )
        .and_on(aux.id, move |(obj, aux), evt: &ui::FocusChangedEvent| {
            if evt
                .old_focus
                .as_ref()
                .map(|x| x == obj.common())
                .unwrap_or(false)
            {
                callback(obj, aux, FocusEvent::Lost);
            } else if evt
                .new_focus
                .as_ref()
                .map(|x| x == obj.common())
                .unwrap_or(false)
            {
                callback(obj, aux, FocusEvent::Gained);
            }
        })
}

pub fn focus_forwarder<E: ui::Element<Aux = T>, T: 'static>(
) -> impl Fn(&mut E, &mut ui::Aux<T>, FocusEvent) + Copy {
    move |obj, aux, event| match event {
        FocusEvent::Gained => {
            obj.emit(aux, FocusGainedEvent);
        }
        FocusEvent::Lost => {
            obj.emit(aux, FocusLostEvent);
        }
    }
}

pub enum KeyboardEvent {
    KeyPress(ui::KeyInput),
    KeyRelease(ui::KeyInput),
    Text(char),
}

pub fn keyboard_handler<T, W: ui::WidgetChildren<T>>(
    aux: &mut ui::Aux<T>,
    callback: impl Fn(&mut W, &mut ui::Aux<T>, KeyboardEvent) + Copy + 'static,
) -> ui::Listener<(ui::Write<W>, ui::Write<ui::Aux<T>>)> {
    aux.listen::<(ui::Write<W>, ui::Write<ui::Aux<T>>)>()
        .and_on(aux.id, move |(obj, aux), event: &ui::KeyPressEvent| {
            if invisible_to_input(obj.visible()) {
                return;
            }

            if let Some(e) = event.0.with(|_| aux.has_focus(obj.common())) {
                callback(obj, aux, KeyboardEvent::KeyPress(*e));
            }
        })
        .and_on(aux.id, move |(obj, aux), event: &ui::KeyReleaseEvent| {
            if invisible_to_input(obj.visible()) {
                return;
            }

            if let Some(e) = event.0.with(|_| aux.has_focus(obj.common())) {
                callback(obj, aux, KeyboardEvent::KeyRelease(*e));
            }
        })
        .and_on(aux.id, move |(obj, aux), event: &ui::TextEvent| {
            if invisible_to_input(obj.visible()) {
                return;
            }

            if let Some(e) = event.0.with(|_| aux.has_focus(obj.common())) {
                callback(obj, aux, KeyboardEvent::Text(*e));
            }
        })
}

pub fn keyboard_forwarder<E: ui::Element<Aux = T>, T: 'static>(
) -> impl Fn(&mut E, &mut ui::Aux<T>, KeyboardEvent) + Copy {
    move |obj, aux, event| match event {
        KeyboardEvent::KeyPress(x) => obj.emit(aux, KeyPressEvent(x)),
        KeyboardEvent::KeyRelease(x) => obj.emit(aux, KeyReleaseEvent(x)),
        KeyboardEvent::Text(x) => obj.emit(aux, TextEvent(x)),
    }
}

pub fn invisible_to_input(v: ui::Visibility) -> bool {
    v == ui::Visibility::NoSelf || v == ui::Visibility::Invisible || v == ui::Visibility::None
}

/// Convenience builder-like utility around the label widget.
///
/// Ensure that `inner()` is invoked once customization is finished so
/// that the unique borrow of the view is dropped.
pub struct LabelRef<'a, T: 'static, S: 'static>(
    ui::view::ChildRef<Label<T>>,
    &'a mut ui::view::View<T, S>,
);

impl<'a, T: 'static, S: 'static> LabelRef<'a, T, S> {
    /// Consumes `self` and returns the inner [`ChildRef`](ui::view::ChildRef).
    #[inline]
    pub fn into_inner(self) -> ui::view::ChildRef<Label<T>> {
        self.0
    }

    pub fn layout<L: layout::Layout>(self, layout: &mut layout::Node<L>, config: L::Config) -> Self
    where
        L::Id: Clone,
    {
        layout.push(self.1.get(self.0).unwrap(), config);
        self
    }

    /// Sets the label text.
    #[inline]
    pub fn text(self, text: impl Into<gfx::DisplayText>) -> Self {
        self.1.get_mut(self.0).unwrap().set_text(text);
        self
    }

    #[inline]
    pub fn max_width(self, max_width: impl Into<Option<f32>>) -> Self {
        self.1.get_mut(self.0).unwrap().set_max_width(max_width);
        self
    }

    /// Sets the size of the label text.
    #[inline]
    pub fn size(self, size: f32) -> Self {
        self.1.get_mut(self.0).unwrap().set_size(size);
        self
    }
}

/// Convenience builder-like utility around the button widget.
///
/// Ensure that `inner()` is invoked once customization is finished so
/// that the unique borrow of the view is dropped.
pub struct ButtonRef<'a, T: 'static, S: 'static>(
    ui::view::ChildRef<Button<T>>,
    &'a mut ui::view::View<T, S>,
);

impl<'a, T: 'static, S: 'static> ButtonRef<'a, T, S> {
    // Consumes `self` and returns the inner [`ChildRef`](ui::view::ChildRef).
    #[inline]
    pub fn into_inner(self) -> ui::view::ChildRef<Button<T>> {
        self.0
    }

    pub fn layout<L: layout::Layout>(self, layout: &mut layout::Node<L>, config: L::Config) -> Self
    where
        L::Id: Clone,
    {
        layout.push(self.1.get(self.0).unwrap(), config);
        self
    }

    pub fn text(self, text: impl Into<gfx::DisplayText>) -> Self {
        self.1.get_mut(self.0).unwrap().set_text(text);
        self
    }

    /// Handles the button press event.
    pub fn press(
        self,
        mut handler: impl FnMut(&mut ui::view::View<T, S>, &mut ui::Aux<T>, &gfx::Point) + 'static,
    ) -> Self {
        self.1.handle(self.0, move |view, aux, event: &PressEvent| {
            handler(view, aux, &event.0);
        });
        self
    }
}

/// Convenience mix-in trait which simplifies the creation of common widgets.
pub trait ViewMixin<T: 'static, S: 'static> {
    /// Creates a button widget and returns a builder-like object.
    fn button<'a>(&'a mut self, aux: &mut ui::Aux<T>) -> ButtonRef<'a, T, S>;

    /// Creates a label widget and returns a builder-like object.
    fn label<'a>(&'a mut self, aux: &mut ui::Aux<T>) -> LabelRef<'a, T, S>;
}

impl<T: 'static, S: 'static> ViewMixin<T, S> for ui::view::View<T, S> {
    fn button<'a>(&'a mut self, aux: &mut ui::Aux<T>) -> ButtonRef<'a, T, S> {
        let child = self.child(Button::new, aux);
        ButtonRef(child, self)
    }

    fn label<'a>(&'a mut self, aux: &mut ui::Aux<T>) -> LabelRef<'a, T, S> {
        let child = self.child(Label::new, aux);
        LabelRef(child, self)
    }
}

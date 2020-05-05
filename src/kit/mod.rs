use {crate::ui, reclutch::display as gfx};

pub mod button;
pub mod label;
pub mod vstack;

pub use {button::*, label::*, vstack::*};

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

/// The widget was pressed.
pub struct PressEvent(pub gfx::Point);
/// The widget was released from its press ([`PressEvent`](PressEvent)).
pub struct ReleaseEvent(pub gfx::Point);
/// The cursor entered the widget boundaries.
pub struct BeginHoverEvent(pub gfx::Point);
/// The cursor left the widget boundaries.
pub struct EndHoverEvent(pub gfx::Point);

pub enum InteractionEvent {
    Press(gfx::Point),
    Release(gfx::Point),
    BeginHover(gfx::Point),
    EndHover(gfx::Point),
}

pub fn interaction_handler<T, W: ui::WidgetChildren<T>>(
    aux: &mut ui::Aux<T>,
    callback: impl Fn(&mut W, &mut ui::Aux<T>, InteractionEvent) + Copy + 'static,
    mask: impl Into<Option<InteractionMask>>,
) -> ui::Listener<W, ui::Aux<T>> {
    let mask = mask.into().unwrap_or(Default::default());
    aux.listen()
        .and_on(
            aux.id,
            move |obj: &mut W, aux, event: &ui::MousePressEvent| {
                if !mask.press {
                    return;
                }
                let bounds = obj.bounds();
                if let Some(&(_, pos)) = event
                    .0
                    .with(|&(btn, pos)| btn == ui::MouseButton::Left && bounds.contains(pos))
                {
                    obj.common().with(|x| x.interaction.pressed = true);
                    callback(obj, aux, InteractionEvent::Press(pos));
                }
            },
        )
        .and_on(
            aux.id,
            move |obj: &mut W, aux, event: &ui::MouseReleaseEvent| {
                if !mask.release {
                    return;
                }
                let bounds = obj.bounds();
                if let Some(&(_, pos)) = event
                    .0
                    .with(|&(btn, pos)| btn == ui::MouseButton::Left && bounds.contains(pos))
                {
                    obj.common().with(|x| x.interaction.pressed = false);
                    callback(obj, aux, InteractionEvent::Release(pos));
                }
            },
        )
        .and_on(
            aux.id,
            move |obj: &mut W, aux, event: &ui::MouseMoveEvent| {
                if !mask.begin_hover && !mask.end_hover {
                    return;
                }
                let bounds = obj.bounds();
                let was_hovered = obj.common().with(|x| x.interaction.hovered);
                let pos = if let Some(&pos) = event.0.with(|&pos| bounds.contains(pos)) {
                    obj.common().with(|x| x.interaction.hovered = true);
                    pos
                } else {
                    obj.common().with(|x| x.interaction.hovered = false);
                    event.0.get().clone()
                };

                if was_hovered != obj.common().with(|x| x.interaction.hovered) {
                    if was_hovered {
                        callback(obj, aux, InteractionEvent::EndHover(pos));
                    } else {
                        callback(obj, aux, InteractionEvent::BeginHover(pos));
                    }
                }
            },
        )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionMask {
    pub press: bool,
    pub release: bool,
    pub begin_hover: bool,
    pub end_hover: bool,
}

impl Default for InteractionMask {
    fn default() -> Self {
        InteractionMask {
            press: true,
            release: true,
            begin_hover: true,
            end_hover: true,
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
                obj.common().emit(aux, PressEvent(pos));
            }
        }
        InteractionEvent::Release(pos) => {
            if mask.release {
                obj.common().emit(aux, ReleaseEvent(pos));
            }
        }
        InteractionEvent::BeginHover(pos) => {
            if mask.begin_hover {
                obj.common().emit(aux, BeginHoverEvent(pos));
            }
        }
        InteractionEvent::EndHover(pos) => {
            if mask.end_hover {
                obj.common().emit(aux, EndHoverEvent(pos));
            }
        }
    }
}

/// Convenience builder-like utility around the label widget.
///
/// Ensure that `inner()` is invoked once customization is finished so
/// that the unique borrow of the view is dropped.
pub struct LabelExtRef<'a, T: 'static, S: 'static>(
    ui::view::ChildRef<Label<T>>,
    &'a mut ui::view::View<T, S>,
);

impl<'a, T: 'static, S: 'static> LabelExtRef<'a, T, S> {
    /// Consumes `self` and returns the inner [`ChildRef`](ui::view::ChildRef).
    #[inline]
    pub fn into_ref(self) -> ui::view::ChildRef<Label<T>> {
        self.0
    }

    /// Sets the label text.
    #[inline]
    pub fn text(self, text: impl Into<gfx::DisplayText>) -> Self {
        self.1.get_mut(self.0).unwrap().set_text(text);
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
pub struct ButtonExtRef<'a, T: 'static, S: 'static>(
    ui::view::ChildRef<Button<T>>,
    &'a mut ui::view::View<T, S>,
);

impl<'a, T: 'static, S: 'static> ButtonExtRef<'a, T, S> {
    // Consumes `self` and returns the inner [`ChildRef`](ui::view::ChildRef).
    #[inline]
    pub fn inner(self) -> ui::view::ChildRef<Button<T>> {
        self.0
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
    /// Creates a button widget with a specified label.
    fn button(
        &mut self,
        label: impl Into<gfx::DisplayText>,
        aux: &mut ui::Aux<T>,
    ) -> ui::view::ChildRef<Button<T>>;

    /// Creates a button widget layed out with a specified label.
    fn lay_button<L: ui::Layout<T>>(
        &mut self,
        label: impl Into<gfx::DisplayText>,
        layout: ui::view::ChildRef<L>,
        config: L::Config,
        aux: &mut ui::Aux<T>,
    ) -> ui::view::ChildRef<Button<T>>;

    /// Creates a button widget with a specified label and returns a specialized reference.
    fn button_ext<'a>(
        &'a mut self,
        label: impl Into<gfx::DisplayText>,
        aux: &mut ui::Aux<T>,
    ) -> ButtonExtRef<'a, T, S>;

    /// Creates a button widget layed out with a specified label and returns a specialized reference.
    fn lay_button_ext<'a, L: ui::Layout<T>>(
        &'a mut self,
        label: impl Into<gfx::DisplayText>,
        layout: ui::view::ChildRef<L>,
        config: L::Config,
        aux: &mut ui::Aux<T>,
    ) -> ButtonExtRef<'a, T, S>;

    /// Creates a label widget with specified text.
    fn label(
        &mut self,
        text: impl Into<gfx::DisplayText>,
        aux: &mut ui::Aux<T>,
    ) -> ui::view::ChildRef<Label<T>>;

    /// Creates a label widget layed out with specified text.
    fn lay_label<L: ui::Layout<T>>(
        &mut self,
        text: impl Into<gfx::DisplayText>,
        layout: ui::view::ChildRef<L>,
        config: L::Config,
        aux: &mut ui::Aux<T>,
    ) -> ui::view::ChildRef<Label<T>>;

    /// Creates a label widget with specified text and returns a specialized reference with
    /// builder-like conveniences.
    fn label_ext<'a>(
        &'a mut self,
        text: impl Into<gfx::DisplayText>,
        aux: &mut ui::Aux<T>,
    ) -> LabelExtRef<'a, T, S>;

    /// Creates a label widget layed out with specified text and returns a specialized reference.
    fn lay_label_ext<'a, L: ui::Layout<T>>(
        &'a mut self,
        text: impl Into<gfx::DisplayText>,
        layout: ui::view::ChildRef<L>,
        config: L::Config,
        aux: &mut ui::Aux<T>,
    ) -> LabelExtRef<'a, T, S>;
}

impl<T: 'static, S: 'static> ViewMixin<T, S> for ui::view::View<T, S> {
    fn button(
        &mut self,
        label: impl Into<gfx::DisplayText>,
        aux: &mut ui::Aux<T>,
    ) -> ui::view::ChildRef<Button<T>> {
        let r = self.child(Button::new, aux);
        self.get_mut(r).unwrap().set_text(label);
        r
    }

    fn lay_button<L: ui::Layout<T>>(
        &mut self,
        label: impl Into<gfx::DisplayText>,
        layout: ui::view::ChildRef<L>,
        config: L::Config,
        aux: &mut ui::Aux<T>,
    ) -> ui::view::ChildRef<Button<T>> {
        let r = self.lay(Button::new, aux, layout, config);
        self.get_mut(r).unwrap().set_text(label);
        r
    }

    fn button_ext<'a>(
        &'a mut self,
        label: impl Into<gfx::DisplayText>,
        aux: &mut ui::Aux<T>,
    ) -> ButtonExtRef<'a, T, S> {
        ButtonExtRef(self.button(label, aux), self)
    }

    fn lay_button_ext<'a, L: ui::Layout<T>>(
        &'a mut self,
        label: impl Into<gfx::DisplayText>,
        layout: ui::view::ChildRef<L>,
        config: L::Config,
        aux: &mut ui::Aux<T>,
    ) -> ButtonExtRef<'a, T, S> {
        ButtonExtRef(self.lay_button(label, layout, config, aux), self)
    }

    fn label(
        &mut self,
        text: impl Into<gfx::DisplayText>,
        aux: &mut ui::Aux<T>,
    ) -> ui::view::ChildRef<Label<T>> {
        let r = self.child(Label::new, aux);
        self.get_mut(r).unwrap().set_text(text);
        r
    }

    fn lay_label<L: ui::Layout<T>>(
        &mut self,
        text: impl Into<gfx::DisplayText>,
        layout: ui::view::ChildRef<L>,
        config: L::Config,
        aux: &mut ui::Aux<T>,
    ) -> ui::view::ChildRef<Label<T>> {
        let r = self.lay(Label::new, aux, layout, config);
        self.get_mut(r).unwrap().set_text(text);
        r
    }

    fn label_ext<'a>(
        &'a mut self,
        text: impl Into<gfx::DisplayText>,
        aux: &mut ui::Aux<T>,
    ) -> LabelExtRef<'a, T, S> {
        LabelExtRef(self.label(text, aux), self)
    }

    fn lay_label_ext<'a, L: ui::Layout<T>>(
        &'a mut self,
        text: impl Into<gfx::DisplayText>,
        layout: ui::view::ChildRef<L>,
        config: L::Config,
        aux: &mut ui::Aux<T>,
    ) -> LabelExtRef<'a, T, S> {
        LabelExtRef(self.lay_label(text, layout, config, aux), self)
    }
}

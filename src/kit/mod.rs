use {
    crate::ui::{self, layout, ElementMixin},
    reclutch::display as gfx,
};

pub mod button;
pub mod label;

pub use {button::*, label::*};

/// The widget was pressed.
pub struct PressEvent(pub gfx::Point);
/// The widget was released from its press ([`PressEvent`](PressEvent)).
pub struct ReleaseEvent(pub gfx::Point);
/// The cursor entered the widget boundaries.
pub struct BeginHoverEvent(pub gfx::Point);
/// The cursor left the widget boundaries.
pub struct EndHoverEvent(pub gfx::Point);

pub trait ComparableCommon {
    fn compare(&self, common: &ui::CommonRef) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StoredLayout<C: ComparableCommon> {
    pub children: Vec<C>,
}

impl<C: ComparableCommon> Default for StoredLayout<C> {
    fn default() -> Self {
        StoredLayout {
            children: Default::default(),
        }
    }
}

impl<C: ComparableCommon> StoredLayout<C> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn push(&mut self, child: &ui::CommonRef, data: C) -> bool {
        if !self.has(child) {
            self.children.push(data);
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn insert(&mut self, child: &ui::CommonRef, data: C, index: usize) -> bool {
        if !self.has(&child) {
            self.children.insert(index, data);
            true
        } else {
            false
        }
    }

    pub fn reorder(&mut self, child: &ui::CommonRef, new_index: usize) -> bool {
        if let Some(idx) = self.position(child) {
            let x = self.children.remove(idx);
            self.insert(child, x, new_index);
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, child: &ui::CommonRef) -> bool {
        if let Some(idx) = self.position(child) {
            self.children.remove(idx);
            true
        } else {
            false
        }
    }

    pub fn set<D>(&mut self, child: &ui::CommonRef, data: D, set: impl FnOnce(&mut C, D)) -> bool {
        if let Some(idx) = self.position(child) {
            set(&mut self.children[idx], data);
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn has(&self, child: &ui::CommonRef) -> bool {
        self.position(child).is_some()
    }

    #[inline]
    pub fn position(&self, child: &ui::CommonRef) -> Option<usize> {
        self.children.iter().position(|c| c.compare(child))
    }
}

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
                obj.emit(aux, PressEvent(pos));
            }
        }
        InteractionEvent::Release(pos) => {
            if mask.release {
                obj.emit(aux, ReleaseEvent(pos));
            }
        }
        InteractionEvent::BeginHover(pos) => {
            if mask.begin_hover {
                obj.emit(aux, BeginHoverEvent(pos));
            }
        }
        InteractionEvent::EndHover(pos) => {
            if mask.end_hover {
                obj.emit(aux, EndHoverEvent(pos));
            }
        }
    }
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

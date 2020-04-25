use {crate::ui, reclutch::display as gfx};

pub mod button;
pub mod label;

pub use {button::*, label::*};

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
    pub fn inner(self) -> ui::view::ChildRef<Label<T>> {
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
    fn lay_button<
        L: ui::Layout<
            UpdateAux = ui::Aux<T>,
            GraphicalAux = ui::Aux<T>,
            DisplayObject = gfx::DisplayCommand,
        >,
    >(
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
    fn lay_button_ext<
        'a,
        L: ui::Layout<
            UpdateAux = ui::Aux<T>,
            GraphicalAux = ui::Aux<T>,
            DisplayObject = gfx::DisplayCommand,
        >,
    >(
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
    fn lay_label<
        L: ui::Layout<
            UpdateAux = ui::Aux<T>,
            GraphicalAux = ui::Aux<T>,
            DisplayObject = gfx::DisplayCommand,
        >,
    >(
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
    fn lay_label_ext<
        'a,
        L: ui::Layout<
            UpdateAux = ui::Aux<T>,
            GraphicalAux = ui::Aux<T>,
            DisplayObject = gfx::DisplayCommand,
        >,
    >(
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

    fn lay_button<
        L: ui::Layout<
            UpdateAux = ui::Aux<T>,
            GraphicalAux = ui::Aux<T>,
            DisplayObject = gfx::DisplayCommand,
        >,
    >(
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

    fn lay_button_ext<
        'a,
        L: ui::Layout<
            UpdateAux = ui::Aux<T>,
            GraphicalAux = ui::Aux<T>,
            DisplayObject = gfx::DisplayCommand,
        >,
    >(
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

    fn lay_label<
        L: ui::Layout<
            UpdateAux = ui::Aux<T>,
            GraphicalAux = ui::Aux<T>,
            DisplayObject = gfx::DisplayCommand,
        >,
    >(
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

    fn lay_label_ext<
        'a,
        L: ui::Layout<
            UpdateAux = ui::Aux<T>,
            GraphicalAux = ui::Aux<T>,
            DisplayObject = gfx::DisplayCommand,
        >,
    >(
        &'a mut self,
        text: impl Into<gfx::DisplayText>,
        layout: ui::view::ChildRef<L>,
        config: L::Config,
        aux: &mut ui::Aux<T>,
    ) -> LabelExtRef<'a, T, S> {
        LabelExtRef(self.lay_label(text, layout, config, aux), self)
    }
}

pub enum InteractionEvent {
    Press(gfx::Point),
    Release(gfx::Point),
    BeginHover(gfx::Point),
    EndHover(gfx::Point),
}

pub fn interaction_handler<
    T: ui::WidgetChildren<
        UpdateAux = ui::Aux<A>,
        GraphicalAux = ui::Aux<A>,
        DisplayObject = gfx::DisplayCommand,
    >,
    A,
>(
    aux: &mut ui::Aux<A>,
    _callback: &'static (impl Fn(&mut T, &mut ui::Aux<A>, InteractionEvent) + 'static),
) -> ui::Listener<T, ui::Aux<A>> {
    aux.listen()
    /*sinq::QueueHandler::new(&aux.node)
    .and_on(
        "mouse_press",
        move |obj: &mut T, aux: &mut ui::Aux<A>, event| {
            let bounds = obj.bounds();
            if let Some(&(_, pos)) = event
                .unwrap_as_mouse_press()
                .unwrap()
                .with(|&(btn, pos)| btn == ui::MouseButton::Left && bounds.contains(pos))
            {
                obj.common().with(|x| x.interaction.pressed = true);
                callback(obj, aux, InteractionEvent::Press(pos));
            }
        },
    )
    .and_on(
        "mouse_release",
        move |obj: &mut T, aux: &mut ui::Aux<A>, event| {
            let bounds = obj.bounds();
            if let Some(&(_, pos)) = event
                .unwrap_as_mouse_release()
                .unwrap()
                .with(|&(btn, pos)| btn == ui::MouseButton::Left && bounds.contains(pos))
            {
                obj.common().with(|x| x.interaction.pressed = false);
                callback(obj, aux, InteractionEvent::Release(pos));
            }
        },
    )
    .and_on(
        "mouse_move",
        move |obj: &mut T, aux: &mut ui::Aux<A>, event| {
            let bounds = obj.bounds();
            let was_hovered = obj.common().with(|x| x.interaction.hovered);
            let event = event.unwrap_as_mouse_move().unwrap();
            let pos = if let Some(&pos) = event.with(|&pos| bounds.contains(pos)) {
                obj.common().with(|x| x.interaction.hovered = true);
                pos
            } else {
                obj.common().with(|x| x.interaction.hovered = false);
                event.get().clone()
            };

            if was_hovered != obj.common().with(|x| x.interaction.hovered) {
                if was_hovered {
                    callback(obj, aux, InteractionEvent::EndHover(pos));
                } else {
                    callback(obj, aux, InteractionEvent::BeginHover(pos));
                }
            }
        },
    )*/
}

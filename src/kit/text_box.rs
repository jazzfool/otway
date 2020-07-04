use {
    crate::{kit, prelude::*, theme, ui},
    reclutch::display as gfx,
};

/// Widget which can accept various forms of string-based user input.
///
/// This widget shouldn't be used on its own. It is deliberately rendered as only the text and cursor.
/// Thus, it makes it possible to create more versatile text input widgets on top of this.
///
/// If you want a simple text input widget, instead use [`TextInput`](crate::kit::TextInput).
pub struct TextBox<T: 'static> {
    text_label: kit::Label<T>,
    text: String,
    placeholder: String,
    wrap: bool,
    censor: Option<Box<dyn FnMut(&str) -> String>>,
    multi_line: bool,
    cursor: usize,

    painter: theme::Painter<Self>,
    common: ui::CommonRef,
    listeners: ui::ListenerList<kit::ReadWrite<Self>>,
    components: ui::ComponentList<Self>,
}

impl<T: 'static> TextBox<T> {
    pub fn new(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> Self {
        let common = ui::CommonRef::new(parent);

        let focus_listener = kit::focus_handler(
            aux,
            kit::focus_forwarder(),
            kit::FocusConfig {
                mouse_trigger: Default::default(),
                interaction_handler: common.with(|x| x.id()),
            },
        );

        let keyboard_listener = kit::keyboard_handler(aux, |obj: &mut Self, aux, event| {
            let mut text = obj.text().to_string();
            match event {
                kit::KeyboardEvent::Text(c) => {
                    text.insert(obj.cursor, c);
                    obj.cursor += 1;
                }
                kit::KeyboardEvent::KeyPress(key) => match key {
                    ui::KeyInput::Back if obj.cursor > 0 => {
                        obj.cursor -= 1;
                        text.remove(obj.cursor);
                    }
                    ui::KeyInput::Left if obj.cursor > 0 => {
                        obj.cursor -= 1;
                    }
                    ui::KeyInput::Right if obj.cursor < text.len() => {
                        obj.cursor += 1;
                    }
                    _ => {}
                },
                _ => {}
            }
            obj.set_text(text);

            kit::keyboard_forwarder()(obj, aux, event);
        });

        TextBox {
            text_label: kit::Label::new(common.clone(), aux),
            text: Default::default(),
            placeholder: Default::default(),
            wrap: false,
            censor: None,
            multi_line: false,
            cursor: 0,

            painter: theme::get_painter(aux.theme.as_ref(), theme::painters::TEXT_BOX),
            common,
            listeners: ui::ListenerList::new(vec![focus_listener, keyboard_listener]),
            components: ui::ComponentList::new().and_push(
                kit::InteractionState::<T, Self, _>::new(
                    aux,
                    kit::interaction_forwarder(None),
                    None,
                    None,
                ),
            ),
        }
    }

    pub fn set_text(&mut self, text: impl ToString) {
        self.text = text.to_string();
        self.cursor = self.cursor.min(self.text.len());
        self.update_label();
    }

    #[inline]
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_placeholder(&mut self, placeholder: impl ToString) {
        self.placeholder = placeholder.to_string();
        self.update_label();
    }

    #[inline]
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    /// Changes whether text which overflows the width can be rendered on a separate line.
    ///
    /// Note: This is not the same as physical newlines; it is merely a modification in rendering.
    pub fn set_wrap(&mut self, wrap: bool) {
        self.wrap = wrap;
        self.update_label();
    }

    #[inline]
    pub fn wrap(&self) -> bool {
        self.wrap
    }

    /// Changes the (optional) censor function.
    /// The censor function can take a string slice and return a new string with arbitrary censorship applied (e.g. password field).
    ///
    /// For a premade censor function for passwords, see [`password_censor`](password_censor).
    pub fn set_censor(&mut self, censor: impl FnMut(&str) -> String + 'static) {
        self.censor = Some(Box::new(censor));
        self.update_label();
    }

    /// Resets the censor function; no censor function will be applied.
    pub fn reset_censor(&mut self) {
        self.censor = None;
        self.update_label();
    }

    /// Returns the censor function, if any.
    #[inline]
    pub fn censor(&self) -> Option<&dyn FnMut(&str) -> String> {
        self.censor.as_ref().map(|x| x.as_ref())
    }

    /// Changes the multi-line ability of this textbox.
    ///
    /// This differs from the wrapping mode ([`set_wrap`](TextBox::set_wrap)), in that wrapping is
    /// the ability to overflow text without clipping, whereas multi-line is the ability to
    /// receive newline inputs to create physical newlines (e.g. user pressing enter/return key).
    pub fn set_multi_line(&mut self, multi_line: bool) {
        self.multi_line = multi_line;
        self.update_label();
    }

    /// Returns the state of the multi-line ability of this textbox.
    #[inline]
    pub fn multi_line(&self) -> bool {
        self.multi_line
    }

    #[inline]
    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    #[inline]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    fn update_label(&mut self) {
        let mut text = if self.text.is_empty() {
            self.placeholder.clone()
        } else {
            self.text.clone()
        };

        if let Some(censor) = &mut self.censor {
            text = censor(&text);
        }

        if !self.multi_line {
            text = text.replace(&['\n', '\r'][..], "");
        }

        self.text_label.set_text(text);
        self.text_label.set_max_width(if self.wrap {
            Some(self.bounds().size.width)
        } else {
            None
        });
    }
}

impl<T: 'static> ui::Element for TextBox<T> {
    type Aux = T;

    #[inline]
    fn common(&self) -> &ui::CommonRef {
        &self.common
    }

    fn update(&mut self, aux: &mut ui::Aux<T>) {
        self.text_label
            .set_color(aux.theme.color(if self.text.is_empty() {
                theme::colors::WEAK_FOREGROUND
            } else {
                theme::colors::FOREGROUND
            }));

        ui::dispatch_components(self, aux, |x| &mut x.components).unwrap();
        ui::dispatch_list::<kit::ReadWrite<Self>, _>((self, aux), |(x, _)| &mut x.listeners);

        ui::propagate_repaint(self);
    }

    fn draw(&mut self, display: &mut dyn gfx::GraphicsDisplay, aux: &mut ui::Aux<T>) {
        ui::draw(
            self,
            |o, aux| theme::paint(o, |o| &mut o.painter, aux),
            display,
            aux,
            None,
        );
    }
}

impl<T: 'static> ui::WidgetChildren<T> for TextBox<T> {
    crate::children![for <T>; text_label];
}

/// Censor function for [`TextBox`](TextBox), appropriate for password fields.
#[inline]
pub fn password_censor(s: &str) -> String {
    "•".repeat(s.len())
}

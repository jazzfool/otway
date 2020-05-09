use {
    crate::{kit, prelude::*, theme::*, ui},
    reclutch::display as gfx,
    std::rc::Rc,
};

const CORNER_RADIUS: f32 = 5.0;
const CORNER_RADII: [f32; 4] = [CORNER_RADIUS, CORNER_RADIUS, CORNER_RADIUS, CORNER_RADIUS];

pub type FontRef = (gfx::ResourceReference, gfx::FontInfo);

#[derive(Clone)]
pub struct Fonts {
    ui_regular: FontRef,
}

#[derive(Clone, Copy)]
pub struct FontSizes {
    ui: f32,
}

struct Inner {
    fonts: Fonts,
    font_sizes: FontSizes,
}

pub struct FlatTheme(Rc<Inner>);

impl FlatTheme {
    pub fn new(
        display: &mut dyn gfx::GraphicsDisplay,
        fonts: Option<Fonts>,
        font_sizes: Option<FontSizes>,
    ) -> Result<Self, ThemeError> {
        let fonts = if let Some(fonts) = fonts {
            fonts
        } else {
            Fonts {
                ui_regular: {
                    let info = gfx::FontInfo::from_name(
                        "Segoe UI", // Windows
                        &[
                            "SF Display",      // MacOS
                            "Helvetica",       // MacOS
                            "Lucida Grande",   // MacOS
                            "Noto Sans",       // Linux
                            "Liberation Sans", // Linux
                            "Cantarell",       // Linux
                        ],
                        None,
                    )?;

                    let reference = display.new_resource(gfx::ResourceDescriptor::Font(
                        gfx::ResourceData::Data(gfx::SharedData::RefCount(std::sync::Arc::new(
                            info.data().ok_or(ThemeError::ResourceError(
                                reclutch::error::ResourceError::InvalidData,
                            ))?,
                        ))),
                    ))?;

                    (reference, info)
                },
            }
        };

        let font_sizes = font_sizes.unwrap_or_else(|| FontSizes { ui: 14.0 });

        Ok(FlatTheme(Rc::new(Inner { fonts, font_sizes })))
    }
}

impl<T: 'static> Theme<T> for FlatTheme {
    fn painter(&self, p: &'static str) -> Box<dyn AnyPainter<T>> {
        match p {
            painters::BUTTON => Box::new(ButtonPainter {
                _theme: Rc::clone(&self.0),
            }),
            painters::LABEL => Box::new(LabelPainter {
                theme: Rc::clone(&self.0),
            }),
            _ => unimplemented!(),
        }
    }

    fn color(&self, c: &'static str) -> gfx::Color {
        match c {
            colors::FOREGROUND => gfx::Color::new(0.8, 0.8, 0.8, 0.8),
            colors::BACKGROUND => gfx::Color::new(0.0, 0.0, 0.0, 1.0),
            colors::WEAK_FOREGROUND => gfx::Color::new(0.5, 0.5, 0.5, 1.0),
            colors::STRONG_FOREGROUND => gfx::Color::new(1.0, 1.0, 1.0, 1.0),
            colors::STRONG_BACKGROUND => gfx::Color::new(0.2, 0.2, 0.2, 1.0),
            _ => unimplemented!(),
        }
    }

    fn standards(&self) -> Standards {
        Standards {
            label_size: self.0.font_sizes.ui,
            button_text_alignment: ui::layout::Alignment::Middle,
        }
    }
}

struct ButtonPainter {
    _theme: Rc<Inner>,
}

impl<T: 'static> TypedPainter<T> for ButtonPainter {
    type Object = kit::Button<T>;

    fn paint(
        &mut self,
        obj: &mut kit::Button<T>,
        aux: &mut ui::Aux<T>,
    ) -> Vec<gfx::DisplayCommand> {
        let mut out = gfx::DisplayListBuilder::new();

        out.push_round_rectangle(
            obj.bounds(),
            CORNER_RADII,
            gfx::GraphicsDisplayPaint::Fill(gfx::StyleColor::Color(
                aux.theme.color(colors::STRONG_BACKGROUND),
            )),
            None,
        );

        out.build()
    }

    #[inline]
    fn size_hint(&mut self, _obj: &mut kit::Button<T>) -> gfx::Size {
        gfx::Size::new(20.0, 6.0)
    }
}

struct LabelPainter {
    theme: Rc<Inner>,
}

impl LabelPainter {
    fn text_bounds(&self, text: gfx::DisplayText, size: f32) -> gfx::Size {
        gfx::TextDisplayItem {
            text,
            font: self.theme.fonts.ui_regular.0,
            font_info: self.theme.fonts.ui_regular.1.clone(),
            size,
            bottom_left: Default::default(),
            color: gfx::StyleColor::Color(Default::default()),
        }
        .bounds()
        .unwrap()
        .size
    }
}

impl<T: 'static> TypedPainter<T> for LabelPainter {
    type Object = kit::Label<T>;

    fn paint(&mut self, obj: &mut kit::Label<T>, aux: &mut ui::Aux<T>) -> Vec<gfx::DisplayCommand> {
        let mut out = gfx::DisplayListBuilder::new();

        let mut text = gfx::TextDisplayItem {
            text: obj.text().clone(),
            font: self.theme.fonts.ui_regular.0,
            font_info: self.theme.fonts.ui_regular.1.clone(),
            size: obj.size(),
            bottom_left: Default::default(),
            color: gfx::StyleColor::Color(aux.theme.color(colors::FOREGROUND)),
        };

        text.set_top_left(obj.bounds().origin);

        out.push_text(text, None);

        out.build()
    }

    #[inline]
    fn size_hint(&mut self, obj: &mut kit::Label<T>) -> gfx::Size {
        self.text_bounds(obj.text().clone(), obj.size())
    }
}

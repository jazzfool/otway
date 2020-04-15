use {
    crate::{kit, prelude::*, theme::*, ui},
    reclutch::display as gfx,
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

pub struct FlatTheme {
    fonts: Fonts,
    font_sizes: FontSizes,
}

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

        let font_sizes = font_sizes.unwrap_or_else(|| FontSizes { ui: 12.0 });

        Ok(FlatTheme { fonts, font_sizes })
    }
}

impl<T: 'static> Theme<T> for FlatTheme {
    fn painter(&self, p: &'static str) -> Box<dyn AnyPainter<T>> {
        match p {
            painters::BUTTON => Box::new(ButtonPainter {
                fonts: self.fonts.clone(),
                font_sizes: self.font_sizes,
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
            _ => unimplemented!(),
        }
    }
}

struct ButtonPainter {
    fonts: Fonts,
    font_sizes: FontSizes,
}

impl ButtonPainter {
    fn text_bounds(&self, text: gfx::DisplayText) -> gfx::Size {
        gfx::TextDisplayItem {
            text,
            bottom_left: Default::default(),
            color: gfx::StyleColor::Color(Default::default()),
            font: self.fonts.ui_regular.0,
            font_info: self.fonts.ui_regular.1.clone(),
            size: self.font_sizes.ui,
        }
        .bounds()
        .unwrap()
        .size
    }
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
            obj.common().get_ref().rect(),
            CORNER_RADII,
            gfx::GraphicsDisplayPaint::Fill(gfx::StyleColor::Color(
                aux.theme.color(colors::FOREGROUND),
            )),
            None,
        );

        out.build()
    }

    #[inline]
    fn size_hint(&mut self, obj: &mut kit::Button<T>) -> gfx::Size {
        self.text_bounds(obj.text().clone())
    }
}

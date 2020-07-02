use {
    crate::{kit, prelude::*, theme::*, ui},
    reclutch::display as gfx,
    std::rc::Rc,
};

#[inline]
fn rgba(r: u8, g: u8, b: u8, a: f32) -> gfx::Color {
    gfx::Color::new(r as f32 / 255., g as f32 / 255., b as f32 / 255., a)
}

fn with_alpha(mut c: gfx::Color, a: f32) -> gfx::Color {
    c.alpha = a;
    c
}

const CORNER_RADIUS: f32 = 5.;
const CORNER_RADII: [f32; 4] = [CORNER_RADIUS, CORNER_RADIUS, CORNER_RADIUS, CORNER_RADIUS];

const BLUR_RADIUS: f32 = 20.;
const TRANSLUCENCY: f32 = 0.8;

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
            painters::TEXT_BOX => Box::new(TextBoxPainter {
                theme: Rc::clone(&self.0),
                count: 0,
                last_cur: std::usize::MAX,
            }),
            painters::CHECK_MARK_BOX => Box::new(CheckMarkBoxPainter {
                _theme: Rc::clone(&self.0),
            }),
            painters::COMBO_BOX => Box::new(ComboBoxPainter {
                _theme: Rc::clone(&self.0),
            }),
            painters::COMBO_LIST => Box::new(ComboListPainter {
                _theme: Rc::clone(&self.0),
            }),
            painters::COMBO_LIST_ITEM => Box::new(ComboListItemPainter {
                _theme: Rc::clone(&self.0),
            }),
            _ => unimplemented!(),
        }
    }

    fn color(&self, c: &'static str) -> gfx::Color {
        match c {
            colors::FOREGROUND => rgba(180, 180, 180, 1.0),
            colors::BACKGROUND => rgba(38, 38, 38, 1.0),
            colors::WEAK_FOREGROUND => rgba(109, 109, 109, 1.0),
            colors::STRONG_BACKGROUND => rgba(58, 58, 58, 1.0),
            colors::TEXT_CONTROL => rgba(26, 26, 26, 1.0),
            colors::ACTIVE => rgba(25, 78, 197, 1.0),
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
        Default::default()
    }

    fn metrics(&self, _obj: &kit::Button<T>, metric: &'static str) -> Option<f32> {
        match metric {
            metrics::PADDING_X => Some(30.),
            metrics::PADDING_Y => Some(3.),
            _ => None,
        }
    }
}

struct LabelPainter {
    theme: Rc<Inner>,
}

impl LabelPainter {
    fn text_bounds(&self, text: gfx::DisplayText, size: f32, max_width: Option<f32>) -> gfx::Size {
        let item = gfx::TextDisplayItem {
            text,
            font: self.theme.fonts.ui_regular.0,
            font_info: self.theme.fonts.ui_regular.1.clone(),
            size,
            bottom_left: Default::default(),
            color: gfx::StyleColor::Color(Default::default()),
        };

        if let Some(max_width) = max_width {
            let height = item.bounds().unwrap().size.height;
            let items = item.linebreak(max_width, height, true).unwrap();
            gfx::Size::new(max_width, height * items.len() as f32)
        } else {
            item.bounds().unwrap().size
        }
    }
}

impl<T: 'static> TypedPainter<T> for LabelPainter {
    type Object = kit::Label<T>;

    fn paint(
        &mut self,
        obj: &mut kit::Label<T>,
        _aux: &mut ui::Aux<T>,
    ) -> Vec<gfx::DisplayCommand> {
        let mut out = gfx::DisplayListBuilder::new();

        let mut text = gfx::TextDisplayItem {
            text: obj.text().clone(),
            font: self.theme.fonts.ui_regular.0,
            font_info: self.theme.fonts.ui_regular.1.clone(),
            size: obj.size(),
            bottom_left: Default::default(),
            color: gfx::StyleColor::Color(obj.color()),
        };

        text.set_top_left(obj.bounds().origin);

        let items = if let Some(max_width) = obj.max_width() {
            let height = text.bounds().unwrap().size.height;
            text.linebreak(max_width, height, true).unwrap()
        } else {
            vec![text]
        };

        for item in items {
            out.push_text(item, None);
        }

        out.build()
    }

    #[inline]
    fn size_hint(&mut self, obj: &mut kit::Label<T>) -> gfx::Size {
        self.text_bounds(obj.text().clone(), obj.size(), obj.max_width())
    }
}

struct TextBoxPainter {
    theme: Rc<Inner>,
    count: usize,
    last_cur: usize,
}

impl<T: 'static> TypedPainter<T> for TextBoxPainter {
    type Object = kit::TextBox<T>;

    fn paint(
        &mut self,
        obj: &mut kit::TextBox<T>,
        aux: &mut ui::Aux<T>,
    ) -> Vec<gfx::DisplayCommand> {
        if !aux.has_focus(obj.common()) {
            return Default::default();
        }

        self.count += 1;

        if self.last_cur != obj.cursor() {
            self.count = 0;
        }

        self.last_cur = obj.cursor();

        if self.count > 60 {
            self.count = 0;
            return Default::default();
        } else if self.count > 30 {
            return Default::default();
        }

        let mut out = gfx::DisplayListBuilder::new();

        let cur = gfx::TextDisplayItem {
            text: obj.text().into(),
            font: self.theme.fonts.ui_regular.0,
            font_info: self.theme.fonts.ui_regular.1.clone(),
            size: self.theme.font_sizes.ui,
            bottom_left: Default::default(),
            color: gfx::StyleColor::Color(Default::default()),
        }
        .limited_bounds(obj.cursor())
        .unwrap()
        .size
        .round();

        let pos = obj.bounds().origin;
        out.push_line(
            gfx::Point::new(pos.x + cur.width, pos.y),
            gfx::Point::new(pos.x + cur.width, pos.y + cur.height),
            gfx::GraphicsDisplayStroke {
                thickness: 1.,
                color: aux.theme.color(colors::FOREGROUND).into(),
                ..Default::default()
            },
            None,
        );

        out.build()
    }

    fn size_hint(&mut self, _obj: &mut kit::TextBox<T>) -> gfx::Size {
        Default::default()
    }
}

fn check_mark(r: gfx::Rect) -> gfx::VectorPath {
    let mut path = gfx::VectorPathBuilder::new();

    path.move_to(r.origin + gfx::Size::new(r.size.width, 0.));
    path.line_to(r.origin + gfx::Size::new(r.size.width / 2., r.size.height));
    path.line_to(r.origin + gfx::Size::new(0., r.size.height / 2.));

    path.build()
}

struct CheckMarkBoxPainter {
    _theme: Rc<Inner>,
}

impl<T: 'static> TypedPainter<T> for CheckMarkBoxPainter {
    type Object = kit::CheckMarkBox<T>;

    fn paint(
        &mut self,
        obj: &mut kit::CheckMarkBox<T>,
        aux: &mut ui::Aux<T>,
    ) -> Vec<gfx::DisplayCommand> {
        let mut out = gfx::DisplayListBuilder::new();

        let color = aux.theme.color(if obj.checked() {
            colors::ACTIVE
        } else {
            colors::STRONG_BACKGROUND
        });

        let bounds = obj.bounds();

        out.push_round_rectangle(
            bounds,
            CORNER_RADII,
            gfx::GraphicsDisplayPaint::Fill(gfx::StyleColor::Color(color)),
            None,
        );

        if obj.checked() {
            out.push_path(
                check_mark(bounds.inflate(-4., -4.)),
                false,
                gfx::GraphicsDisplayPaint::Stroke(gfx::GraphicsDisplayStroke {
                    thickness: 2.,
                    color: aux.theme.color(colors::FOREGROUND).into(),
                    ..Default::default()
                }),
                None,
            )
        }

        out.build()
    }

    #[inline]
    fn size_hint(&mut self, _obj: &mut kit::CheckMarkBox<T>) -> gfx::Size {
        gfx::Size::new(20., 20.)
    }

    fn metrics(&self, _obj: &kit::CheckMarkBox<T>, metric: &'static str) -> Option<f32> {
        match metric {
            metrics::CHECK_MARK_SPACING => Some(5.0),
            _ => None,
        }
    }
}

fn up_down_arrows(rect: gfx::Rect) -> [gfx::VectorPath; 2] {
    let c = rect.center();
    let v = if rect.size.width > rect.size.height {
        rect.size.height
    } else {
        rect.size.width
    } / 3.;
    let d = v / 2.;

    let mut path1 = gfx::VectorPathBuilder::new();
    path1.move_to(c + gfx::Vector::new(-v, -v + d));
    path1.line_to(c + gfx::Vector::new(0., 2. * -v + d));
    path1.line_to(c + gfx::Vector::new(v, -v + d));

    let mut path2 = gfx::VectorPathBuilder::new();
    path2.move_to(c + gfx::Vector::new(-v, v - d));
    path2.line_to(c + gfx::Vector::new(0., 2. * v - d));
    path2.line_to(c + gfx::Vector::new(v, v - d));

    [path1.build(), path2.build()]
}

struct ComboBoxPainter {
    _theme: Rc<Inner>,
}

impl<T: 'static> TypedPainter<T> for ComboBoxPainter {
    type Object = kit::ComboBox<T>;

    fn paint(
        &mut self,
        obj: &mut kit::ComboBox<T>,
        aux: &mut ui::Aux<T>,
    ) -> Vec<gfx::DisplayCommand> {
        let mut out = gfx::DisplayListBuilder::new();

        let bounds = obj.bounds();

        out.save();
        out.push_round_rectangle_clip(bounds, CORNER_RADII);

        out.push_rectangle(
            bounds,
            gfx::GraphicsDisplayPaint::Fill(gfx::StyleColor::Color(
                aux.theme.color(colors::TEXT_CONTROL),
            )),
            None,
        );

        let mut icon_bg = bounds;
        icon_bg.size.width = 15.;
        icon_bg.origin.x = ui::layout::align_x(icon_bg, bounds, ui::layout::Alignment::End, 0.);

        out.push_rectangle(
            icon_bg,
            gfx::GraphicsDisplayPaint::Fill(gfx::StyleColor::Color(
                aux.theme.color(colors::ACTIVE),
            )),
            None,
        );

        for v in up_down_arrows(icon_bg.inflate(-1., -1.))
            .to_vec()
            .into_iter()
        {
            out.push_path(
                v,
                false,
                gfx::GraphicsDisplayPaint::Stroke(gfx::GraphicsDisplayStroke {
                    thickness: 2.,
                    color: aux.theme.color(colors::FOREGROUND).into(),
                    ..Default::default()
                }),
                None,
            );
        }

        out.restore();

        out.build()
    }

    #[inline]
    fn size_hint(&mut self, _obj: &mut kit::ComboBox<T>) -> gfx::Size {
        Default::default()
    }

    fn metrics(&self, _obj: &Self::Object, metric: &'static str) -> Option<f32> {
        match metric {
            metrics::PADDING_X => Some(30.),
            metrics::PADDING_Y => Some(3.),
            _ => None,
        }
    }
}

struct ComboListPainter {
    _theme: Rc<Inner>,
}

impl<T: 'static> TypedPainter<T> for ComboListPainter {
    type Object = kit::ComboList<T>;

    fn paint(&mut self, obj: &mut Self::Object, aux: &mut ui::Aux<T>) -> Vec<gfx::DisplayCommand> {
        let mut out = gfx::DisplayListBuilder::new();

        let bounds = obj.bounds();

        out.push_round_rectangle_backdrop(
            bounds,
            CORNER_RADII,
            gfx::Filter::Blur(BLUR_RADIUS, BLUR_RADIUS),
        );

        out.push_round_rectangle(
            bounds,
            CORNER_RADII,
            gfx::GraphicsDisplayPaint::Fill(gfx::StyleColor::Color(with_alpha(
                aux.theme.color(colors::TEXT_CONTROL),
                TRANSLUCENCY,
            ))),
            None,
        );

        out.build()
    }

    fn size_hint(&mut self, obj: &mut Self::Object) -> gfx::Size {
        Default::default()
    }
}

struct ComboListItemPainter {
    _theme: Rc<Inner>,
}

impl<T: 'static> TypedPainter<T> for ComboListItemPainter {
    type Object = kit::ComboListItem<T>;

    fn paint(&mut self, obj: &mut Self::Object, aux: &mut ui::Aux<T>) -> Vec<gfx::DisplayCommand> {
        Default::default()
    }

    fn size_hint(&mut self, obj: &mut Self::Object) -> gfx::Size {
        Default::default()
    }
}

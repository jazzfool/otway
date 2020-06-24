use {crate::ui::layout, reclutch::display as gfx};

/// A 2D position based on a relative offset and an absolute offset.
/// The `relative` offset is expressed as a fraction of the corresponding parent dimension for each component in `(x, y)`.
/// The `post_relative` offset is expressed as a fraction of the result of the `relative`, `post_relative` and `real` calculated size from `FractionSize`.
/// The `real` offset is a concrete size in DPI pixels which is added onto the offset calculated from `relative`.
///
/// For example, `FractionalPosition { relative: (0.3, 0.1), post_relative: (-0.5, 0.0), real: Vector::new(5, 30) }` for a child with a computed size of `50 x 50`
/// placed within a parent of size `100 x 100` positioned at `(50, 50)` will result in an absolute position of `(60, 90)` (or relatively; `(10, 40)`).
/// Following the calculation; `100 * 0.3 = 30, + 50 = 80, + 5 = 85, - 0.5 * 50 = 60` and `100 * 0.1 = 10, + 50 = 60, + 30 = 90`.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FractionalPosition {
    pub relative: (f32, f32),
    pub post_relative: (f32, f32),
    pub real: gfx::Vector,
}

/// A 2D size based on a relative size and an absolute size.
/// The `relative` size is expressed as a fraction of the corresponding parent dimension for each component in `(width, height)`.
/// The `post_relative` size is expressed as a fraction of the result of the `relative` and `real` calculated size.
/// The `real` size is a concrete size in DPI pixels which is added onto the size calculated from `relative`.
///
/// For example, `FractionalSize { relative: (0.5, 0.75), post_relative: (0.6, -0.2), real: Size::new(15, 10) }` placed within a parent
/// of size `100 x 100` will result in a size of `120 x 68`, because `100 * 0.5 = 50, + 15 = 75, + 0.6 * 75 = 120` and `100 * 0.75 = 75, + 10 = 85, - 0.2 * 85 = 68`.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FractionalSize {
    pub relative: (f32, f32),
    pub post_relative: (f32, f32),
    pub real: gfx::Size,
}

/// Configuration for `RelativeBox`.
///
/// If `size` is left as `None`, then the size of the child will be used.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct RelativeBoxConfig {
    pub position: FractionalPosition,
    pub size: Option<FractionalSize>,
}

impl RelativeBoxConfig {
    /// Returns a `RelativeBoxConfig` which will result in the item being centered in the parent.
    pub fn center() -> Self {
        RelativeBoxConfig {
            position: FractionalPosition {
                relative: (0.5, 0.5),
                post_relative: (-0.5, -0.5),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// Powerful layout based on quantities relative to the parent.
/// This only supports a single child.
///
/// This layout is very useful for any of the following scenarios;
/// - Centering an item.
/// - Aligning an item to a corner or side with an arbitrary offset.
/// - Filling a fraction of the parent (e.g. filling the bottom-half of the parent).
///
/// These are all scenarios which can be expressed as screen-space fractions.
pub struct RelativeBox {
    item: Option<layout::Item>,
    config: RelativeBoxConfig,
}

impl RelativeBox {
    pub fn new(config: RelativeBoxConfig) -> Self {
        RelativeBox { item: None, config }
    }
}

impl layout::Layout for RelativeBox {
    type Config = ();
    type Id = ();

    #[inline]
    fn push(&mut self, item: impl Into<layout::Item>, _: ()) {
        self.item = Some(item.into());
    }

    #[inline]
    fn remove(&mut self, _: &()) -> Option<layout::Item> {
        self.item.take()
    }

    #[inline]
    fn get(&self, _: &()) -> Option<&layout::Item> {
        self.item.as_ref()
    }

    #[inline]
    fn get_mut(&mut self, _: &()) -> Option<&mut layout::Item> {
        self.item.as_mut()
    }

    #[inline]
    fn len(&self) -> usize {
        self.item.is_some() as _
    }

    fn items(&self) -> Vec<(&layout::Item, &Self::Id)> {
        if let Some(item) = &self.item {
            vec![(item, &())]
        } else {
            Vec::new()
        }
    }

    fn min_size(&self) -> gfx::Size {
        if let Some(item) = &self.item {
            if !layout::should_layout(item) {
                Default::default()
            } else {
                item.rect().size
            }
        } else {
            Default::default()
        }
    }

    fn update(&mut self, bounds: gfx::Rect) {
        if let Some(item) = &mut self.item {
            if !layout::should_layout(item) {
                return;
            }

            let mut rect = item.rect();
            let position = self.config.position;
            rect.size = if let Some(size) = self.config.size {
                let mut new_size = gfx::Size::new(
                    bounds.size.width * size.relative.0,
                    bounds.size.height * size.relative.1,
                );

                new_size.width += size.real.width;
                new_size.height += size.real.height;

                new_size.width += new_size.width * size.post_relative.0;
                new_size.height += new_size.height * size.post_relative.1;

                new_size
            } else {
                rect.size
            };

            let mut new_position = gfx::Point::new(
                bounds.size.width * position.relative.0,
                bounds.size.height * position.relative.1,
            );

            new_position.x += position.real.x;
            new_position.y += position.real.y;

            new_position.x += rect.size.width * position.post_relative.0;
            new_position.y += rect.size.height * position.post_relative.1;

            rect.origin = new_position;

            item.set_rect(rect);
        }
    }
}

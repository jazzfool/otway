use {crate::ui::layout, reclutch::display as gfx, std::collections::BTreeMap};

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct HStackConfig {
    pub left_margin: f32,
    pub right_margin: f32,
    pub alignment: layout::Alignment,
}

impl From<(f32, f32)> for HStackConfig {
    fn from(margins: (f32, f32)) -> Self {
        HStackConfig {
            left_margin: margins.0,
            right_margin: margins.1,
            ..Default::default()
        }
    }
}

struct Item {
    config: HStackConfig,
    item: layout::Item,
}

pub struct HStack {
    entries: BTreeMap<u64, Item>,
    next_id: u64,
}

impl HStack {
    pub fn new() -> Self {
        HStack {
            entries: Default::default(),
            next_id: 0,
        }
    }
}

impl layout::Layout for HStack {
    type Config = Option<HStackConfig>;
    type Id = u64;

    fn push(&mut self, item: impl Into<layout::Item>, config: Option<HStackConfig>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.insert(
            id,
            Item {
                config: config.unwrap_or_default(),
                item: item.into(),
            },
        );
        id
    }

    #[inline]
    fn remove(&mut self, id: &u64) {
        self.entries.remove(id);
    }

    #[inline]
    fn get(&self, id: &u64) -> Option<&layout::Item> {
        Some(&self.entries.get(id)?.item)
    }

    #[inline]
    fn get_mut(&mut self, id: &u64) -> Option<&mut layout::Item> {
        Some(&mut self.entries.get_mut(id)?.item)
    }

    #[inline]
    fn len(&self) -> usize {
        self.entries.len()
    }

    fn min_size(&self) -> gfx::Size {
        let mut width = 0.0;
        let mut height = 0.0;
        for entry in self.entries.values() {
            let rect = entry.item.rect();
            if rect.size.height > height {
                height = rect.size.height;
            }
            width += rect.size.width + entry.config.left_margin + entry.config.left_margin;
        }
        gfx::Size::new(width, height)
    }

    fn update(&mut self, bounds: gfx::Rect) {
        let mut x = bounds.origin.x;
        for entry in self.entries.values_mut() {
            x += entry.config.left_margin;
            let rect = entry.item.rect();
            entry.item.set_rect(gfx::Rect::new(
                gfx::Point::new(
                    x,
                    layout::align_y(rect, bounds, entry.config.alignment, 0.0),
                ),
                rect.size,
            ));
            x += rect.size.width + entry.config.right_margin;
        }
    }
}

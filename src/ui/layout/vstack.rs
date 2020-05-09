use {crate::ui::layout, reclutch::display as gfx, std::collections::BTreeMap};

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct VStackConfig {
    pub top_margin: f32,
    pub bottom_margin: f32,
    pub alignment: layout::Alignment,
}

impl From<(f32, f32)> for VStackConfig {
    fn from(margins: (f32, f32)) -> Self {
        VStackConfig {
            top_margin: margins.0,
            bottom_margin: margins.1,
            ..Default::default()
        }
    }
}

struct Item {
    config: VStackConfig,
    item: layout::Item,
}

pub struct VStack {
    entries: BTreeMap<u64, Item>,
    next_id: u64,
}

impl VStack {
    pub fn new() -> Self {
        VStack {
            entries: Default::default(),
            next_id: 0,
        }
    }
}

impl layout::Layout for VStack {
    type Config = Option<VStackConfig>;
    type Id = u64;

    fn push(&mut self, item: impl Into<layout::Item>, config: Option<VStackConfig>) -> u64 {
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
            if rect.size.width > width {
                width = rect.size.width;
            }
            height += rect.size.height + entry.config.top_margin + entry.config.bottom_margin;
        }
        gfx::Size::new(width, height)
    }

    fn update(&mut self, bounds: gfx::Rect) {
        let mut y = bounds.origin.y;
        for entry in self.entries.values_mut() {
            y += entry.config.top_margin;
            let rect = entry.item.rect();
            entry.item.set_rect(gfx::Rect::new(
                gfx::Point::new(
                    layout::align_x(rect, bounds, entry.config.alignment, 0.0),
                    y,
                ),
                rect.size,
            ));
            y += rect.size.height + entry.config.bottom_margin;
        }
    }
}

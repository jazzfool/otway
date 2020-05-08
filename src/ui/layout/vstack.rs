use {crate::ui::layout, reclutch::display as gfx, std::collections::BTreeMap};

#[derive(Default)]
pub struct VStackConfig {
    top_margin: f32,
    bottom_margin: f32,
    alignment: layout::Alignment,
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
            height += rect.size.height;
        }
        gfx::Size::new(width, height)
    }

    fn update(&mut self, bounds: gfx::Rect) {
        let mut y = bounds.origin.y;
        for entry in self.entries.values_mut() {
            let rect = entry.item.rect();
            entry.item.set_rect(gfx::Rect::new(
                gfx::Point::new(bounds.origin.x, y),
                rect.size,
            ));
            y += rect.size.height;
        }
    }
}

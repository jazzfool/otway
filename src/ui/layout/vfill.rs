use {crate::ui::layout, reclutch::display as gfx, std::collections::BTreeMap};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VFillConfig {
    pub flex: f32,
    pub margins: layout::SideMargins,
    pub alignment: layout::Alignment,
}

impl Default for VFillConfig {
    fn default() -> Self {
        VFillConfig {
            flex: 1.0,
            margins: Default::default(),
            alignment: Default::default(),
        }
    }
}

impl From<(f32, layout::SideMargins)> for VFillConfig {
    fn from(config: (f32, layout::SideMargins)) -> Self {
        VFillConfig {
            flex: config.0,
            margins: config.1,
            ..Default::default()
        }
    }
}

struct Item {
    config: VFillConfig,
    item: layout::Item,
}

pub struct VFill {
    entries: BTreeMap<u64, Item>,
    next_id: u64,
    total_portions: f32,
}

impl VFill {
    pub fn new() -> Self {
        VFill {
            entries: Default::default(),
            next_id: 0,
            total_portions: 0.0,
        }
    }
}

impl layout::Layout for VFill {
    type Config = Option<VFillConfig>;
    type Id = u64;

    fn push(&mut self, item: impl Into<layout::Item>, config: Option<VFillConfig>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let config = config.unwrap_or_default();
        self.total_portions += config.flex;
        if let Some(prev) = self.entries.insert(
            id,
            Item {
                config,
                item: item.into(),
            },
        ) {
            self.total_portions -= prev.config.flex;
        }
        id
    }

    #[inline]
    fn remove(&mut self, id: &u64) {
        if let Some(entry) = self.entries.remove(id) {
            self.total_portions -= entry.config.flex;
        }
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
            height += rect.size.height + entry.config.margins.top + entry.config.margins.bottom;
        }
        gfx::Size::new(width, height)
    }

    fn update(&mut self, bounds: gfx::Rect) {
        let width_portion = bounds.size.width / self.total_portions;
        let mut y = bounds.origin.y;
        for entry in self.entries.values_mut() {
            y += entry.config.margins.top;
            let mut rect = entry.item.rect();
            rect.size.height = width_portion * entry.config.flex;
            entry.item.set_rect(gfx::Rect::new(
                gfx::Point::new(
                    layout::align_x(
                        rect,
                        bounds,
                        entry.config.alignment,
                        entry.config.margins.horizontal(),
                    ),
                    y,
                ),
                rect.size,
            ));
            y += rect.size.height + entry.config.margins.bottom;
        }
    }
}

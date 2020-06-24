use otway::{
    app, kit,
    prelude::*,
    reclutch::display as gfx,
    theme,
    ui::{
        self, layout,
        view::{ChildRef, View},
    },
};

struct TodoItemCompletionEvent(bool);

struct TodoItem<T: 'static> {
    completed: bool,

    label: ChildRef<kit::Label<T>>,
    btn: ChildRef<kit::Button<T>>,
}

impl<T: 'static> TodoItem<T> {
    pub fn view(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> View<T, Self> {
        let mut view = View::new(
            parent,
            aux,
            TodoItem {
                completed: false,
                label: ChildRef::null(),
                btn: ChildRef::null(),
            },
        );

        let mut hstack = layout::HStack::new().into_node(None);

        let btn = view
            .button(aux)
            .layout(&mut hstack, None)
            .press(|view, aux, _| {
                view.set_state(|x| x.completed = !x.completed);
                view.emit(aux, TodoItemCompletionEvent(view.state().completed));
            })
            .into_inner();

        let label = view
            .label(aux)
            .layout(&mut hstack, Some((5.0, 0.0).into()))
            .into_inner();

        view.set_layout(hstack);
        view.set_layout_mode(ui::LayoutMode::Shrink);

        view.state_changed(|view| {
            let (btn, completed) = (view.state().btn, view.state().completed);
            view.get_mut(btn)
                .unwrap()
                .set_text(if completed { "[X]" } else { "[-]" });
            layout::update_layout(view);
        });

        view.set_state(move |x| {
            x.label = label;
            x.btn = btn;
        });

        view
    }

    pub fn set_task(view: &mut View<T, Self>, task: String) {
        let label = view.state().label;
        view.get_mut(label).unwrap().set_text(task);
    }

    #[inline]
    pub fn is_complete(&self) -> bool {
        self.completed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ItemFilter {
    All,
    Completed,
    Incomplete,
}

impl ToString for ItemFilter {
    fn to_string(&self) -> String {
        match self {
            ItemFilter::All => String::from("All"),
            ItemFilter::Completed => String::from("Completed"),
            ItemFilter::Incomplete => String::from("Incomplete"),
        }
    }
}

struct TodoItemList<T: 'static> {
    items: Vec<ChildRef<View<T, TodoItem<T>>>>,
    filter: ItemFilter,
}

impl<T: 'static> TodoItemList<T> {
    pub fn view(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> View<T, Self> {
        let mut view = View::new(
            parent,
            aux,
            TodoItemList {
                items: Vec::new(),
                filter: ItemFilter::All,
            },
        );

        let mut vstack = layout::VStack::new().into_node(None);

        let tb = view.child(kit::TextBox::new, aux);
        vstack.push(view.get(tb).unwrap(), None);
        view.get_mut(tb).unwrap().set_rect(gfx::Rect::new(
            gfx::Point::new(0., 0.),
            gfx::Size::new(100., 30.),
        ));
        view.get_mut(tb)
            .unwrap()
            .set_placeholder("I want to...".to_string());

        view.button(aux)
            .text("Add item")
            .layout(&mut vstack, Some((0.0, 5.0).into()))
            .press(move |view, aux, _| {
                let text = view.get(tb).unwrap().text().to_string();
                if !text.is_empty() {
                    TodoItemList::add_item(view, aux, text);
                    view.get_mut(tb).unwrap().set_text("");
                }
            });

        view.button(aux)
            .text("Clear items")
            .layout(&mut vstack, Some((0.0, 5.0).into()))
            .press(|view, _, _| TodoItemList::clear_items(view));

        view.button(aux)
            .text("Next filter")
            .layout(&mut vstack, Some((0.0, 5.0).into()))
            .press(|view, _, _| {
                view.set_state(|x| match x.filter {
                    ItemFilter::All => x.filter = ItemFilter::Completed,
                    ItemFilter::Completed => x.filter = ItemFilter::Incomplete,
                    ItemFilter::Incomplete => x.filter = ItemFilter::All,
                });
                Self::filter_items(view);
            });

        let filter_label = view
            .label(aux)
            .layout(&mut vstack, Some((0.0, 5.0).into()))
            .into_inner();

        view.state_changed(move |view| {
            let label_text = format!("Current filter: {}", view.state().filter.to_string());
            view.get_mut(filter_label).unwrap().set_text(label_text);
        });

        view.set_state(|_| {});

        view.set_layout(vstack);

        let mut rb = layout::RelativeBox::new(layout::RelativeBoxConfig::center()).into_node(None);
        rb.push(&view, ());
        aux.central_widget.with(|x| x.set_layout(rb));

        view.set_layout_mode(ui::LayoutMode::Shrink);

        view
    }

    pub fn add_item(view: &mut View<T, TodoItemList<T>>, aux: &mut ui::Aux<T>, task: String) {
        let item = view.child(TodoItem::view, aux);
        TodoItem::set_task(view.get_mut(item).unwrap(), task);

        let item_c = view.get(item).unwrap().common().clone();
        view.common().with(|x| {
            x.layout_mut()
                .unwrap()
                .cast_mut::<layout::VStack>()
                .unwrap()
                .push(item_c, Some((0.0, 5.0).into()));
        });

        view.late_handle(item, |view, _, _: &TodoItemCompletionEvent| {
            Self::filter_items(view)
        });

        view.set_state(move |state| state.items.push(item));
        Self::filter_items(view);
    }

    pub fn clear_items(view: &mut View<T, TodoItemList<T>>) {
        for item in view.state().items.clone() {
            view.get(item).unwrap().mark_for_detach();
            view.remove(item);
        }
        view.set_state(|x| x.items.clear());
        layout::update_layout(view);
    }

    fn filter_items(view: &mut View<T, TodoItemList<T>>) {
        let filter = view.state().filter;
        for item in view.state().items.clone() {
            if filter == ItemFilter::All {
                view.get(item).unwrap().set_visible(ui::Visibility::All);
                ui::propagate_visibility(view.get_mut(item).unwrap());
                continue;
            }

            let completed = view.get(item).unwrap().state().is_complete();
            if (completed && filter == ItemFilter::Completed)
                || (!completed && filter == ItemFilter::Incomplete)
            {
                view.get(item).unwrap().set_visible(ui::Visibility::All);
            } else if (completed && filter == ItemFilter::Incomplete)
                || (!completed && filter == ItemFilter::Completed)
            {
                view.get(item).unwrap().set_visible(ui::Visibility::None);
            }
            ui::propagate_visibility(view.get_mut(item).unwrap());
        }
        layout::update_layout(view);
    }
}

fn main() -> Result<(), app::AppError> {
    app::run(
        TodoItemList::view,
        (),
        |display| Box::new(theme::flat::FlatTheme::new(display, None, None).unwrap()),
        Default::default(),
    )
}

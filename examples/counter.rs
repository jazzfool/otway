use otway::{
    app,
    prelude::*,
    theme,
    ui::{self, layout, view::View},
};

type AuxData = ();
type Aux = app::AppAux<AuxData>;
type AppAux = app::AppData<AuxData>;

struct IncrementEvent;
struct DecrementEvent;

fn counter(parent: ui::CommonRef, aux: &mut Aux) -> View<AppAux, i32> {
    let mut view = View::new(parent, aux, 0);

    let mut vfill = layout::VFill::new().into_node(None);
    let mut vstack = layout::VStack::new().into_node(None);
    let mut hstack = layout::HStack::new().into_node(None);

    let label = view
        .label(aux)
        .layout(&mut vstack, Some((0.0, 5.0).into()))
        .size(14.0)
        .into_inner();

    view.button(aux)
        .text("Increment")
        .layout(&mut hstack, Some((0.0, 5.0).into()))
        .press(|view, aux, _| {
            view.set_state(|x| *x += 1);
            aux.emit(view, IncrementEvent);
        });

    view.button(aux)
        .text("Decrement")
        .layout(&mut hstack, Some((0.0, 5.0).into()))
        .press(|view, aux, _| {
            view.set_state(|x| *x -= 1);
            aux.emit(view, DecrementEvent);
        });

    vstack.push(hstack, None);
    vfill.push(
        vstack,
        Some((1.0, layout::SideMargins::new_all_same(5.0)).into()),
    );
    view.set_layout(vfill);

    view.state_changed(move |view| {
        let count = *view.state();
        view.get_mut(label)
            .unwrap()
            .set_text(format!("Count: {}", count));
        layout::update_layout(view);
    });

    view.set_state(|_| {});
    view
}

fn main() -> Result<(), app::AppError> {
    app::run(
        counter,
        (),
        |display| Box::new(theme::flat::FlatTheme::new(display, None, None).unwrap()),
        Default::default(),
    )
}

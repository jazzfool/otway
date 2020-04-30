use otway::{
    app,
    prelude::*,
    theme,
    ui::{self, view::View},
};

type AuxData = ();
type Aux = app::AppAux<AuxData>;
type AppAux = app::AppData<AuxData>;

struct IncrementEvent;

fn counter(parent: ui::CommonRef, aux: &mut Aux) -> View<AppAux, i32> {
    let mut view = View::new(parent, aux, 0);

    view.button_ext("Increment", aux).press(|view, aux, _| {
        view.set_state(|x| *x += 1);
        aux.emit(view.id(), IncrementEvent);
    });

    let id = view.id();
    view.listener_mut().on(id, |_, _, _: &IncrementEvent| {
        println!("hello!");
    });

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

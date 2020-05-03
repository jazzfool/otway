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
struct DecrementEvent;

fn counter(parent: ui::CommonRef, aux: &mut Aux) -> View<AppAux, i32> {
    let mut view = View::new(parent, aux, 0);

    view.button_ext("Increment", aux).press(|view, aux, _| {
        view.set_state(|x| *x += 1);
        aux.emit(view, IncrementEvent);
    });

    view.button_ext("Decrement", aux).press(|view, aux, _| {
        view.set_state(|x| *x -= 1);
        aux.emit(view, DecrementEvent);
    });

    let label = view.label_ext("", aux).size(42.0).into_ref();

    view.state_changed(move |view| {
        let count = *view.state();
        view.get_mut(label)
            .unwrap()
            .set_text(format!("count = {}", count))
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

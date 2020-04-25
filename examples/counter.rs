use otway::{
    app, kit,
    prelude::*,
    theme,
    ui::{self, view::View},
};

type AuxData = ();
type Aux = app::AppAux<AuxData>;
type AppAux = app::AppData<AuxData>;

struct IncrementEvent(i32);

fn counter(parent: ui::CommonRef, aux: &mut Aux) -> View<AppAux, i32> {
    let mut view = View::new(parent, aux, 0);

    view.button_ext("Increment", aux).press(|view, aux, _| {
        let e = IncrementEvent(view.set_state(|x| {
            *x += 1;
            *x
        }));
        view.common().emit(aux, e);
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

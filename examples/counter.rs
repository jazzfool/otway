use otway::{
    app, kit, theme,
    ui::{self, view::View},
};

type AuxData = ();
type Aux = app::AppAux<AuxData>;
type AppAux = app::AppData<AuxData>;

fn counter(parent: ui::CommonRef, aux: &mut Aux) -> View<AppAux, i32> {
    let mut view = View::new(parent, aux, 0);

    let btn = view.child(kit::Button::new, aux);
    view.get_mut(btn).unwrap().set_text("Hello");

    view.handle(btn, "press", |view, _, _| {
        view.set_state(|x| *x += 1);
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

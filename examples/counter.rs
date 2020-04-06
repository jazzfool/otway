use otway::{
    app, theme,
    ui::{self, view::View},
};

type AuxData = ();
type Aux = app::AppAux<AuxData>;
type AppAux = app::AppData<AuxData>;

fn counter(parent: ui::CommonRef, aux: &mut Aux) -> View<AppAux, i32> {
    let view = View::new(parent, aux, 0);

    // let layout = view.child(kit::VStack::new, aux);

    // let btn = view.lay(kit::Button::new, aux, &layout, 0);

    view
}

fn main() -> Result<(), app::AppError> {
    app::run(
        counter,
        Default::default(),
        theme::flat::FlatTheme::new(),
        Default::default(),
    )
}

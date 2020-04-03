use otway::{
    app, theme,
    ui::{self, view::View},
};

type Aux = app::AppAux<()>;
type AuxType = app::AppData<()>;

fn counter(parent: ui::CommonRef, _aux: &mut Aux) -> View<AuxType, i32> {
    let view = View::new(parent, 0);

    // let layout = view.child(kit::VStack::new, aux);

    // let btn = view.lay(kit::Button::new, aux, &layout, 0);

    view
}

fn main() -> Result<(), app::AppError> {
    let mut app = app::App::new(
        counter,
        Default::default(),
        theme::flat::FlatTheme::new(),
        Default::default(),
    );
    app.run()
}

use {
    crate::{theme, ui},
    reclutch::{display as gfx, widget::Widget},
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    Context(#[from] glutin::ContextError),
    #[error("{0}")]
    Creation(#[from] glutin::CreationError),
}

#[derive(WidgetChildren, Debug, Clone, PartialEq)]
#[widget_children_trait(ui::WidgetChildren)]
pub struct Root<
    T: 'static,
    W: ui::WidgetChildren<
        UpdateAux = AppAux<T>,
        GraphicalAux = AppAux<T>,
        DisplayObject = gfx::DisplayCommand,
    >,
> {
    #[widget_child]
    child: W,
    common: ui::CommonRef,
}

impl<
        T: 'static,
        W: ui::WidgetChildren<
            UpdateAux = AppAux<T>,
            GraphicalAux = AppAux<T>,
            DisplayObject = gfx::DisplayCommand,
        >,
    > Widget for Root<T, W>
{
    type UpdateAux = AppAux<T>;
    type GraphicalAux = AppAux<T>;
    type DisplayObject = gfx::DisplayCommand;
}

impl<
        T: 'static,
        W: ui::WidgetChildren<
            UpdateAux = AppAux<T>,
            GraphicalAux = AppAux<T>,
            DisplayObject = gfx::DisplayCommand,
        >,
    > ui::Element for Root<T, W>
{
    fn common(&self) -> &ui::CommonRef {
        &self.common
    }
}

impl<
        T: 'static,
        W: ui::WidgetChildren<
            UpdateAux = AppAux<T>,
            GraphicalAux = AppAux<T>,
            DisplayObject = gfx::DisplayCommand,
        >,
    > Root<T, W>
{
    pub fn new(new: impl FnOnce(ui::CommonRef, &mut AppAux<T>) -> W, aux: &mut AppAux<T>) -> Self {
        let common = ui::CommonRef::root();
        Root {
            child: new(common.clone(), aux),
            common,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AppData<T> {
    pub data: T,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppOptions {
    window_size: gfx::Size,
}

impl Default for AppOptions {
    fn default() -> Self {
        AppOptions {
            window_size: gfx::Size::new(960.0, 540.0),
        }
    }
}

pub struct App<
    T: 'static,
    W: ui::WidgetChildren<
        UpdateAux = AppAux<T>,
        GraphicalAux = AppAux<T>,
        DisplayObject = gfx::DisplayCommand,
    >,
> {
    aux: AppAux<T>,
    root: Root<T, W>,
    options: AppOptions,
}

impl<
        T: 'static,
        W: ui::WidgetChildren<
            UpdateAux = AppAux<T>,
            GraphicalAux = AppAux<T>,
            DisplayObject = gfx::DisplayCommand,
        >,
    > App<T, W>
{
    pub fn new(
        new: impl FnOnce(ui::CommonRef, &mut AppAux<T>) -> W,
        aux: T,
        theme: impl theme::Theme<AppData<T>> + 'static,
        options: AppOptions,
    ) -> Self {
        let mut aux = ui::Aux {
            data: AppData { data: aux },
            theme: Box::new(theme),
        };

        App {
            root: Root::new(new, &mut aux),
            aux,
            options,
        }
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        let el = glutin::event_loop::EventLoop::new();

        let wb =
            glutin::window::WindowBuilder::new().with_inner_size(glutin::dpi::PhysicalSize::new(
                self.options.window_size.width,
                self.options.window_size.height,
            ));

        let ctxt = glutin::ContextBuilder::new().build_windowed(wb, &el)?;
        let ctxt = unsafe { ctxt.make_current().map_err(|(_, e)| e)? };

        el.run(|event, _window, control_flow| {});

        Ok(())
    }
}

pub type AppAux<T> = ui::Aux<AppData<T>>;

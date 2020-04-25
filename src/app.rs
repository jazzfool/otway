use {
    crate::{theme, ui},
    glutin::event::{self as winit_event, Event, WindowEvent},
    reclutch::{
        display::{self as gfx, GraphicsDisplay},
        widget::Widget,
    },
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    ContextError(#[from] glutin::ContextError),
    #[error("{0}")]
    CreationError(#[from] glutin::CreationError),
    #[error("{0}")]
    SkiaError(#[from] reclutch::error::SkiaError),
}

#[derive(WidgetChildren)]
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
    node: sinq::EventNode<Self, AppAux<T>, ui::NoEvent>,
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

    #[inline]
    fn update(&mut self, aux: &mut AppAux<T>) {
        ui::update(self, aux);
    }

    #[inline]
    fn draw(&mut self, display: &mut dyn gfx::GraphicsDisplay, aux: &mut AppAux<T>) {
        ui::draw(
            self,
            |_, aux| {
                vec![gfx::DisplayCommand::Clear(
                    aux.theme.color(theme::colors::BACKGROUND),
                )]
            },
            display,
            aux,
        )
    }
}

impl<
        T: 'static,
        W: ui::WidgetChildren<
            UpdateAux = AppAux<T>,
            GraphicalAux = AppAux<T>,
            DisplayObject = gfx::DisplayCommand,
        >,
    > ui::Node for Root<T, W>
{
    type Event = ui::NoEvent;

    #[inline]
    fn node_ref(&self) -> &sinq::EventNode<Self, Self::UpdateAux, Self::Event> {
        &self.node
    }

    #[inline]
    fn node_mut(&mut self) -> &mut sinq::EventNode<Self, Self::UpdateAux, Self::Event> {
        &mut self.node
    }
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
        let common = ui::CommonRef::new(None);
        Root {
            child: new(common.clone(), aux),
            common,
            node: sinq::EventNode::new(&mut aux.master),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppData<T> {
    pub data: T,
    cursor: gfx::Point,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppOptions {
    pub window_size: gfx::Size,
    pub background: gfx::Color,
}

impl Default for AppOptions {
    fn default() -> Self {
        AppOptions {
            window_size: gfx::Size::new(960.0, 540.0),
            background: gfx::Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

pub fn run<
    T: 'static,
    W: ui::WidgetChildren<
        UpdateAux = AppAux<T>,
        GraphicalAux = AppAux<T>,
        DisplayObject = gfx::DisplayCommand,
    >,
>(
    new: impl FnOnce(ui::CommonRef, &mut AppAux<T>) -> W,
    aux: T,
    theme: impl FnOnce(&mut dyn gfx::GraphicsDisplay) -> Box<dyn theme::Theme<AppData<T>>>,
    mut options: AppOptions,
) -> Result<(), AppError> {
    let el = glutin::event_loop::EventLoop::new();

    let wb = glutin::window::WindowBuilder::new().with_inner_size(glutin::dpi::PhysicalSize::new(
        options.window_size.width,
        options.window_size.height,
    ));
    let ctxt = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(wb, &el)?;
    let ctxt = unsafe { ctxt.make_current().map_err(|(_, e)| e)? };
    let mut scale_factor = ctxt.window().scale_factor();
    let mut display =
        gfx::skia::SkiaGraphicsDisplay::new_gl_framebuffer(&gfx::skia::SkiaOpenGlFramebuffer {
            framebuffer_id: 0,
            size: (
                options.window_size.width as _,
                options.window_size.height as _,
            ),
        })?;
    let mut master = Default::default();
    let mut aux = ui::Aux {
        data: AppData {
            data: aux,
            cursor: Default::default(),
        },
        theme: theme(&mut display),
        node: sinq::EventNode::new(&mut master),
        master,
    };
    let mut root = Root::new(new, &mut aux);
    let mut key_mods = ui::KeyModifiers {
        shift: false,
        ctrl: false,
        alt: false,
        logo: false,
    };
    let (mut cmds_a, mut cmds_b) = (gfx::CommandGroup::new(), gfx::CommandGroup::new());

    el.run(move |event, _window, control_flow| {
        *control_flow = glutin::event_loop::ControlFlow::Wait;

        match event {
            Event::MainEventsCleared => ctxt.window().request_redraw(),
            Event::RedrawRequested(_) => {
                cmds_a.push(
                    &mut display,
                    &[
                        gfx::DisplayCommand::Save,
                        gfx::DisplayCommand::Clear(options.background),
                        gfx::DisplayCommand::Scale(gfx::Vector::new(
                            scale_factor as _,
                            scale_factor as _,
                        )),
                    ],
                    gfx::ZOrder(std::i32::MIN),
                    false,
                    None,
                );

                ui::propagate_draw(&mut root, &mut display, &mut aux);

                cmds_b.push(
                    &mut display,
                    &[gfx::DisplayCommand::Restore],
                    gfx::ZOrder(std::i32::MAX),
                    false,
                    None,
                );

                display.present(None).unwrap();
                ctxt.swap_buffers().unwrap();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                }
                WindowEvent::ScaleFactorChanged {
                    scale_factor: new_scale_factor,
                    ..
                } => {
                    scale_factor = new_scale_factor;
                    let size = ctxt.window().inner_size();
                    options.window_size.width = size.width as _;
                    options.window_size.height = size.height as _;
                }
                WindowEvent::Resized(size) => {
                    options.window_size.width = size.width as _;
                    options.window_size.height = size.height as _;
                }
                WindowEvent::ModifiersChanged(key_modifiers) => {
                    key_mods.shift = key_modifiers.shift();
                    key_mods.ctrl = key_modifiers.ctrl();
                    key_mods.alt = key_modifiers.alt();
                    key_mods.logo = key_modifiers.logo();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let position = position.to_logical::<f64>(scale_factor);
                    let point = gfx::Point::new(position.x as _, position.y as _);
                    aux.node.emit_owned(
                        ui::WindowEvent::MouseMove(ui::ConsumableEvent::new(point)),
                        &mut aux.master,
                    );
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    let mouse_button = match button {
                        winit_event::MouseButton::Left => ui::MouseButton::Left,
                        winit_event::MouseButton::Middle => ui::MouseButton::Middle,
                        winit_event::MouseButton::Right => ui::MouseButton::Right,
                        winit_event::MouseButton::Other(x) => ui::MouseButton::Other(x),
                    };

                    let event = match state {
                        winit_event::ElementState::Pressed => ui::WindowEvent::MousePress(
                            ui::ConsumableEvent::new((mouse_button, aux.data.cursor)),
                        ),
                        winit_event::ElementState::Released => ui::WindowEvent::MouseRelease(
                            ui::ConsumableEvent::new((mouse_button, aux.data.cursor)),
                        ),
                    };

                    aux.node.emit_owned(event, &mut aux.master);
                }
                _ => {}
            },
            _ => return,
        }

        ui::propagate_update(&mut root, &mut aux);
    });
}

pub type AppAux<T> = ui::Aux<AppData<T>>;

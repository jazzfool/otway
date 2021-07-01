use {
    crate::{prelude::*, theme, ui},
    glutin::event::{self as winit_event, Event, WindowEvent},
    reclutch::display::{self as gfx, GraphicsDisplay},
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

type RootReadWrites<T, U> = (ui::Write<T>, ui::Write<AppAux<U>>);

pub struct Root<T: 'static, W: ui::WidgetChildren<AppData<T>>> {
    child: W,

    common: ui::CommonRef,
    listeners: ui::ListenerList<RootReadWrites<Self, T>>,
    components: ui::ComponentList<Self>,
}

impl<T: 'static, W: ui::WidgetChildren<AppData<T>>> ui::Element for Root<T, W> {
    type Aux = AppData<T>;

    #[inline]
    fn common(&self) -> &ui::CommonRef {
        &self.common
    }

    fn update(&mut self, aux: &mut AppAux<T>) {
        ui::dispatch_components(self, aux, |x| &mut x.components).unwrap();
        ui::dispatch_list::<RootReadWrites<Self, T>, _>((self, aux), |(x, _)| &mut x.listeners);
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
            None,
        )
    }
}

impl<T: 'static, W: ui::WidgetChildren<AppData<T>>> Root<T, W> {
    pub fn new(
        new: impl FnOnce(ui::CommonRef, &mut AppAux<T>) -> W,
        common: ui::CommonRef,
        aux: &mut AppAux<T>,
    ) -> Self {
        let focus_listener = crate::kit::focus_handler(
            aux,
            |_, _, _| {},
            crate::kit::FocusConfig {
                interaction_handler: common.with(|x| x.id()),
                mouse_trigger: Default::default(),
            },
        );

        Root {
            child: new(common.clone(), aux),

            common,
            listeners: ui::ListenerList::new(vec![focus_listener]),
            components: ui::ComponentList::new().and_push(crate::kit::InteractionState::new(
                aux,
                crate::kit::interaction_forwarder(None),
                None,
                None,
            )),
        }
    }
}

impl<T: 'static, W: ui::WidgetChildren<AppData<T>>> ui::WidgetChildren<AppData<T>> for Root<T, W> {
    crate::children![for <AppData<T>>; child];
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppData<T> {
    pub data: T,
    cursor: gfx::Point,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppOptions {
    pub window_title: String,
    pub window_size: gfx::Size,
}

impl Default for AppOptions {
    fn default() -> Self {
        AppOptions {
            window_title: "Otway UI".into(),
            window_size: gfx::Size::new(960.0, 540.0),
        }
    }
}

pub struct WindowResizeEvent {
    pub physical: gfx::Size,
    pub logical: gfx::Size,
}

pub fn run<T: 'static, W: ui::WidgetChildren<AppData<T>>>(
    new: impl FnOnce(ui::CommonRef, &mut AppAux<T>) -> W,
    aux: T,
    theme: impl FnOnce(&mut dyn gfx::GraphicsDisplay) -> Box<dyn theme::Theme<AppData<T>>>,
    mut options: AppOptions,
) -> Result<(), AppError> {
    let el = glutin::event_loop::EventLoop::new();

    let wb = glutin::window::WindowBuilder::new()
        .with_title(options.window_title.clone())
        .with_inner_size(glutin::dpi::PhysicalSize::new(
            options.window_size.width,
            options.window_size.height,
        ));
    let ctxt = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(wb, &el)?;
    let ctxt = unsafe { ctxt.make_current().map_err(|(_, e)| e)? };
    let mut scale_factor = ctxt.window().scale_factor();
    let mut display = gfx::skia::SkiaGraphicsDisplay::new_gl_framebuffer(
        |s| ctxt.get_proc_address(s),
        &gfx::skia::SkiaOpenGlFramebuffer {
            framebuffer_id: 0,
            size: (
                options.window_size.width as _,
                options.window_size.height as _,
            ),
        },
    )?;
    let central_widget = ui::CommonRef::new(None);
    let mut aux = ui::Aux {
        data: AppData {
            data: aux,
            cursor: Default::default(),
        },
        theme: theme(&mut display),
        id: uniq::id::next(),
        queue: Default::default(),
        central_widget: central_widget.clone(),
        focus_widget: Default::default(),
    };
    let mut root = Root::new(new, central_widget, &mut aux);
    root.set_layout_mode(ui::LayoutMode::Fill);
    let mut key_mods = ui::KeyModifiers {
        shift: false,
        ctrl: false,
        alt: false,
        logo: false,
    };
    let (mut cmds_a, mut cmds_b) = (gfx::CommandGroup::new(), gfx::CommandGroup::new());

    root.set_size({
        let logical = ctxt.window().inner_size().to_logical::<f64>(scale_factor);
        gfx::Size::new(logical.width as _, logical.height as _)
    });
    ui::layout::update_layout(&root);

    let bg_color = aux.theme.color(theme::colors::BACKGROUND);

    el.run(move |event, _window, control_flow| {
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(
            std::time::Instant::now() + std::time::Duration::from_millis(16),
        );

        match event {
            Event::MainEventsCleared => ctxt.window().request_redraw(),
            Event::RedrawRequested(_) => {
                let size = display.size();
                if options.window_size.width != size.0 as f32
                    || options.window_size.height != size.1 as f32
                {
                    display
                        .resize((
                            options.window_size.width as _,
                            options.window_size.height as _,
                        ))
                        .expect("Display error when resizing");
                }

                cmds_a.push(
                    &mut display,
                    &[
                        gfx::DisplayCommand::Save,
                        gfx::DisplayCommand::Clear(bg_color),
                        gfx::DisplayCommand::Scale(gfx::Vector::new(
                            scale_factor as _,
                            scale_factor as _,
                        )),
                    ],
                    gfx::ZOrder(std::i32::MIN),
                    false,
                    None,
                );

                root.repaint();

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

                    cmds_a.repaint();
                    cmds_b.repaint();
                    let size: glutin::dpi::LogicalSize<f64> = size.to_logical(scale_factor);
                    root.set_size(gfx::Size::new(size.width as _, size.height as _));
                    ui::layout::update_layout(&root);
                }
                WindowEvent::Resized(size) => {
                    options.window_size.width = size.width as _;
                    options.window_size.height = size.height as _;

                    let size: glutin::dpi::LogicalSize<f64> = size.to_logical(scale_factor);
                    root.set_size(gfx::Size::new(size.width as _, size.height as _));
                    ui::layout::update_layout(&root);
                    aux.emit(
                        &aux.id,
                        WindowResizeEvent {
                            physical: options.window_size,
                            logical: gfx::Size::new(size.width as _, size.height as _),
                        },
                    );
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
                    aux.data.cursor = point;
                    aux.queue
                        .emit(aux.id, ui::MouseMoveEvent(ui::ConsumableEvent::new(point)));
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    let mouse_button = match button {
                        winit_event::MouseButton::Left => ui::MouseButton::Left,
                        winit_event::MouseButton::Middle => ui::MouseButton::Middle,
                        winit_event::MouseButton::Right => ui::MouseButton::Right,
                        winit_event::MouseButton::Other(x) => ui::MouseButton::Other(x),
                    };

                    match state {
                        winit_event::ElementState::Pressed => aux.queue.emit(
                            aux.id,
                            ui::MousePressEvent(ui::ConsumableEvent::new((
                                mouse_button,
                                aux.data.cursor,
                            ))),
                        ),
                        winit_event::ElementState::Released => aux.queue.emit(
                            aux.id,
                            ui::MouseReleaseEvent(ui::ConsumableEvent::new((
                                mouse_button,
                                aux.data.cursor,
                            ))),
                        ),
                    };
                }
                WindowEvent::KeyboardInput { input, .. } => match input.state {
                    winit_event::ElementState::Pressed => aux.queue.emit(
                        aux.id,
                        ui::KeyPressEvent(ui::ConsumableEvent::new(
                            input.virtual_keycode.unwrap().into(),
                        )),
                    ),
                    winit_event::ElementState::Released if input.virtual_keycode.is_some() => {
                        aux.queue.emit(
                            aux.id,
                            ui::KeyReleaseEvent(ui::ConsumableEvent::new(
                                input.virtual_keycode.unwrap().into(),
                            )),
                        )
                    }
                    _ => {}
                },
                WindowEvent::ReceivedCharacter(c) if !c.is_control() => aux
                    .queue
                    .emit(aux.id, ui::TextEvent(ui::ConsumableEvent::new(c))),
                _ => {}
            },
            _ => return,
        }

        ui::propagate_update(&mut root, &mut aux);
    });
}

pub type AppAux<T> = ui::Aux<AppData<T>>;

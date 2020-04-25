use {crate::ui, reclutch::display as gfx};

pub mod button;
pub mod label;

pub use {button::*, label::*};

pub enum InteractionEvent {
    Press(gfx::Point),
    Release(gfx::Point),
    BeginHover(gfx::Point),
    EndHover(gfx::Point),
}

pub fn interaction_handler<
    T: ui::WidgetChildren<
        UpdateAux = ui::Aux<A>,
        GraphicalAux = ui::Aux<A>,
        DisplayObject = gfx::DisplayCommand,
    >,
    A,
>(
    aux: &mut ui::Aux<A>,
    callback: &'static (impl Fn(&mut T, &mut ui::Aux<A>, InteractionEvent) + 'static),
) -> sinq::QueueHandler<T, ui::Aux<A>, ui::WindowEvent> {
    sinq::QueueHandler::new(&aux.node)
        .and_on(
            "mouse_press",
            move |obj: &mut T, aux: &mut ui::Aux<A>, event| {
                let bounds = obj.bounds();
                if let Some(&(_, pos)) = event
                    .unwrap_as_mouse_press()
                    .unwrap()
                    .with(|&(btn, pos)| btn == ui::MouseButton::Left && bounds.contains(pos))
                {
                    obj.common().with(|x| x.interaction.pressed = true);
                    callback(obj, aux, InteractionEvent::Press(pos));
                }
            },
        )
        .and_on(
            "mouse_release",
            move |obj: &mut T, aux: &mut ui::Aux<A>, event| {
                let bounds = obj.bounds();
                if let Some(&(_, pos)) = event
                    .unwrap_as_mouse_release()
                    .unwrap()
                    .with(|&(btn, pos)| btn == ui::MouseButton::Left && bounds.contains(pos))
                {
                    obj.common().with(|x| x.interaction.pressed = false);
                    callback(obj, aux, InteractionEvent::Release(pos));
                }
            },
        )
        .and_on(
            "mouse_move",
            move |obj: &mut T, aux: &mut ui::Aux<A>, event| {
                let bounds = obj.bounds();
                let was_hovered = obj.common().with(|x| x.interaction.hovered);
                let event = event.unwrap_as_mouse_move().unwrap();
                let pos = if let Some(&pos) = event.with(|&pos| bounds.contains(pos)) {
                    obj.common().with(|x| x.interaction.hovered = true);
                    pos
                } else {
                    obj.common().with(|x| x.interaction.hovered = false);
                    event.get().clone()
                };

                if was_hovered != obj.common().with(|x| x.interaction.hovered) {
                    if was_hovered {
                        callback(obj, aux, InteractionEvent::EndHover(pos));
                    } else {
                        callback(obj, aux, InteractionEvent::BeginHover(pos));
                    }
                }
            },
        )
}

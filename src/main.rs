use masonry::properties::Background;
use std::sync::Arc;
use winit::error::EventLoopError;
use xilem::core::lens;
use xilem::style::Style;
use xilem::view::{
    Axis, FlexExt as _, FlexSpacer, MainAxisAlignment, SizedBox, button, flex,
    label, sized_box, split,
};
use xilem::{AnyWidgetView, AppState, Color, EventLoop, WidgetView, WindowOptions, Xilem};

fn panel<State, Action, V>(inner: V) -> SizedBox<V, State, Action>
where
    V: WidgetView<State, Action>,
{
    sized_box(inner)
        .background(Background::Color(Color::from_rgba8(255, 255, 255, 25)))
        .border(Color::from_rgba8(255, 255, 255, 75), 1.0)
        .corner_radius(10.0)
        .padding(10.0)
}

#[derive(Clone, Debug)]
struct SplitState {
    lhs: Box<PanelState>,
    rhs: Box<PanelState>,
    axis: Axis,
}

#[derive(Clone, Debug)]
struct HelloState {
    id: usize,
    close_requested: bool,
}

#[derive(Clone, Debug)]
enum PanelState {
    Split(SplitState),
    Hello(HelloState),
}

#[derive(Debug)]
struct State {
    panel: Option<Box<PanelState>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            panel: Some(Box::new(PanelState::Split(SplitState {
                lhs: Box::new(PanelState::Hello(HelloState {
                    id: 1,
                    close_requested: false,
                })),
                rhs: Box::new(PanelState::Hello(HelloState {
                    id: 2,
                    close_requested: false,
                })),
                axis: Axis::Horizontal,
            }))),
        }
    }
}

impl AppState for State {
    fn keep_running(&self) -> bool {
        true
    }
}

fn split_view(state: &mut SplitState) -> Arc<AnyWidgetView<SplitState>> {
    let lhs = lens(panel_view, state, |state: &mut SplitState| &mut state.lhs);
    let rhs = lens(panel_view, state, |state: &mut SplitState| &mut state.rhs);

    let split = split(lhs, rhs)
        .split_axis(state.axis)
        .solid_bar(true)
        .bar_size(0.0);

    Arc::new(split)
}

fn hello_view(state: &mut HelloState) -> Arc<AnyWidgetView<HelloState>> {
    let title = label(format!("Hello {}", state.id));

    let close = button("X", |state: &mut HelloState| {
        state.close_requested = true;
    });

    let panel = panel(
        flex((
            flex((title, FlexSpacer::Flex(1.0), close))
                .gap(0.0)
                .direction(Axis::Horizontal)
                .main_axis_alignment(MainAxisAlignment::Start),
            sized_box(label("Hello!")).expand_height().flex(1.0),
        ))
        .direction(Axis::Vertical)
        .gap(10.0),
    );

    Arc::new(panel)
}

fn panel_view(state: &mut Box<PanelState>) -> Arc<AnyWidgetView<Box<PanelState>>> {
    match &mut **state {
        PanelState::Split(..) => {
            Arc::new(lens(split_view, state, |state: &mut Box<PanelState>| {
                let PanelState::Split(state) = &mut **state else {
                    panic!("underlying view is not Split");
                };

                state
            }))
        }
        PanelState::Hello(..) => {
            Arc::new(lens(hello_view, state, |state: &mut Box<PanelState>| {
                let PanelState::Hello(hello) = &mut **state else {
                    panic!("underlying view is not Hello");
                };

                hello
            }))
        }
    }
}

fn close_panels(state: Box<PanelState>) -> Option<Box<PanelState>> {
    let close_requested = match &*state {
        PanelState::Hello(state) => state.close_requested,
        _ => false,
    };

    if close_requested {
        return None;
    }

    let PanelState::Split(SplitState { lhs, rhs, axis }) = *state else {
        return Some(state);
    };

    let lhs = close_panels(lhs);
    let rhs = close_panels(rhs);

    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) => Some(Box::new(PanelState::Split(SplitState { lhs, rhs, axis }))),
        (Some(lhs), _) => Some(lhs),
        (_, Some(rhs)) => Some(rhs),
        (_, _) => None,
    }
}

fn app_logic(state: &mut State) -> impl WidgetView<State> + use<> {
    if let Some(panel) = &state.panel {
        state.panel = close_panels(panel.clone());
    }

    let inner = if state.panel.is_some() {
        Some(
            lens(panel_view, state, |state: &mut State| {
                state.panel.as_mut().unwrap()
            })
            .flex(1.0),
        )
    } else {
        None
    };

    sized_box(flex(inner)).padding(10.0)
}

fn main() -> Result<(), EventLoopError> {
    let app = Xilem::new_simple(
        State::default(),
        app_logic,
        WindowOptions::new("Xilem Tiles"),
    );
    app.run_in(EventLoop::with_user_event())?;

    Ok(())
}

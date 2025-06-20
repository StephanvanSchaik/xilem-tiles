use masonry::properties::Background;
use std::collections::HashMap;
use std::sync::Arc;
use winit::error::EventLoopError;
use xilem::style::Style;
use xilem::view::{
    Axis, FlexExt as _, FlexSpacer, MainAxisAlignment, SizedBox, button, flex, label, sized_box,
    split,
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

#[derive(Debug)]
enum PanelState {
    Split { lhs: usize, rhs: usize, axis: Axis },
    Hello,
}

#[derive(Debug, Default)]
struct State {
    panels: HashMap<usize, PanelState>,
    panel_id: usize,
}

impl State {
    fn split(&mut self, id: usize, axis: Axis) {
        // Take out the state of the original ID, which we are going to replace with the split to
        // preserve the root ID.
        let original_state = self.panels.remove(&id).unwrap();

        // Create the left-hand side panel with the original state.
        let lhs = self.panel_id;
        self.panel_id += 1;
        self.panels.insert(lhs, original_state);

        // Create the right-hand panel with the new state.
        let rhs = self.panel_id;
        self.panel_id += 1;
        self.panels.insert(rhs, PanelState::Hello);

        // Re-use the original ID to create the split referenceing the left-hand and right-hand
        // sides.
        self.panels.insert(id, PanelState::Split { lhs, rhs, axis });
    }

    fn close(&mut self, parent_id: Option<usize>, id: usize) {
        if let Some(parent_id) = parent_id {
            // Remove the original split.
            let Some(PanelState::Split { lhs, rhs, .. }) = self.panels.remove(&parent_id) else {
                return;
            };

            // Remove the original view.
            let remaining_state = if lhs == id {
                self.panels.remove(&lhs);
                self.panels.remove(&rhs)
            } else if rhs == id {
                self.panels.remove(&rhs);
                self.panels.remove(&lhs)
            } else {
                return;
            };

            let Some(remaining_state) = remaining_state else {
                return;
            };

            // Replace the split with the remaining view.
            self.panels.insert(parent_id, remaining_state);
        } else {
            self.panels.clear();
        }
    }
}

impl AppState for State {
    fn keep_running(&self) -> bool {
        true
    }
}

fn split_view(
    state: &mut State,
    _parent_id: Option<usize>,
    id: usize,
) -> Option<Arc<AnyWidgetView<State>>> {
    let Some(PanelState::Split { lhs, rhs, axis }) = state.panels.get(&id) else {
        return None;
    };

    let lhs = *lhs;
    let rhs = *rhs;
    let axis = *axis;

    let lhs = panel_view(state, Some(id), lhs);
    let rhs = panel_view(state, Some(id), rhs);

    let split = split(flex(lhs), flex(rhs))
        .split_axis(axis)
        .solid_bar(true)
        .bar_size(0.0);

    Some(Arc::new(split))
}

fn hello_view(
    state: &mut State,
    parent_id: Option<usize>,
    id: usize,
) -> Option<Arc<AnyWidgetView<State>>> {
    let Some(PanelState::Hello) = state.panels.get(&id) else {
        return None;
    };

    let title = label(format!("Hello {}", id));

    let split_horizontally = button("H", move |state: &mut State| {
        state.split(id, Axis::Horizontal)
    });
    let split_vertically = button("V", move |state: &mut State| {
        state.split(id, Axis::Vertical)
    });
    let close = button("X", move |state: &mut State| state.close(parent_id, id));

    let panel = panel(
        flex((
            flex((
                title,
                FlexSpacer::Flex(1.0),
                split_horizontally,
                split_vertically,
                close,
            ))
            .gap(0.0)
            .direction(Axis::Horizontal)
            .main_axis_alignment(MainAxisAlignment::Start),
            sized_box(label("Hello!")).expand_height().flex(1.0),
        ))
        .direction(Axis::Vertical)
        .gap(10.0),
    );

    Some(Arc::new(panel))
}

fn panel_view(
    state: &mut State,
    parent_id: Option<usize>,
    id: usize,
) -> Option<Arc<AnyWidgetView<State>>> {
    match state.panels.get(&id)? {
        PanelState::Split { .. } => split_view(state, parent_id, id),
        PanelState::Hello => hello_view(state, parent_id, id),
    }
}

fn app_logic(state: &mut State) -> impl WidgetView<State> + use<> {
    // If there are no empty, create a default panel.
    if state.panels.is_empty() {
        state.panels.insert(0, PanelState::Hello);
        state.panel_id = 1;
    }

    let inner = panel_view(state, None, 0);

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

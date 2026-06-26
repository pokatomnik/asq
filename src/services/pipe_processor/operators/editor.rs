use iocraft::prelude::*;

use std::sync::{Arc, Mutex};

use crate::services::pipe_processor::pipe_operator::PipeOperator;

pub(crate) struct Editor;

impl Editor {
    pub fn new() -> Self {
        Self {}
    }

    fn get_input(&self, prompt: &str) -> String {
        let shared_state = Arc::new(Mutex::new(String::new()));
        let sc = shared_state.clone();
        let has_err = smol::block_on(
            element!(EditorApp(
                title: prompt,
                result_state: sc
            ))
            .fullscreen(),
        )
        .is_err();
        if has_err {
            return String::new();
        }
        match shared_state.lock() {
            Ok(mx) => mx.to_string(),
            Err(_) => String::new(),
        }
    }
}

impl PipeOperator for Editor {
    fn handle(&self, prompt: &str) -> anyhow::Result<String> {
        Ok(self.get_input(prompt))
    }
}

#[derive(Default, Props)]
struct EditorAppProps {
    title: String,
    result_state: Arc<Mutex<String>>,
}

#[component]
fn EditorApp(props: &EditorAppProps, hooks: &mut Hooks) -> impl Into<AnyElement<'static>> {
    let (w, h) = hooks.use_terminal_size();
    let mut state = hooks.use_state(|| String::new());
    let mut should_exit = hooks.use_state(|| false);

    let result_state = props.result_state.clone();
    let notify_changes = move |new_value: String| {
        if let Ok(mut mx) = result_state.lock() {
            *mx = new_value;
        }
    };

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent {
                code,
                kind,
                modifiers,
                ..
            }) if kind != KeyEventKind::Release => match (code, modifiers) {
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => should_exit.set(true),
                _ => {}
            },
            _ => {}
        }
    });

    element! {
        TitlePanel(
            title: format!("{} [Press Ctrl+C when done]", props.title),
            width: w,
            height: h
        ) {
            TextInput(
                value: state.to_string(),
                multiline: true,
                has_focus: true,
                on_change: move |new_value: String| {
                    notify_changes(new_value.clone());
                    state.set(new_value)
                }
            )
        }
    }
}

#[derive(Props, Default)]
struct PanelProps<'a> {
    title: String,
    children: Vec<AnyElement<'a>>,
    width: u16,
    height: u16,
}

#[component]
fn TitlePanel<'a>(props: &mut PanelProps<'a>) -> impl Into<AnyElement<'a>> {
    element! {
        View(
            flex_direction: FlexDirection::Column,
            width: props.width,
            height: props.height,
        ) {
            View(
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
            ) {
                View(flex_grow: 1.0, border_style: BorderStyle::Single, border_edges: Edges::Top | Edges::Left)
                Text(content: format!(" {} ", props.title), weight: Weight::Bold)
                View(flex_grow: 1.0, border_style: BorderStyle::Single, border_edges: Edges::Top | Edges::Right)
            }

            View(
                padding: 1,
                flex_grow: 1.0,
                border_style: BorderStyle::Single,
                border_edges: Edges::Left | Edges::Right | Edges::Bottom,
            ) {
                #(props.children.drain(..))
            }
        }
    }
}

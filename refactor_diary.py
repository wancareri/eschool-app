import re

with open('src/pages/diary.rs', 'r') as f:
    content = f.read()

# Lift state in render
content = content.replace('let refreshing = Signal::new(false);', 
"""let refreshing = Signal::new(false);
    let page_width = Signal::new(400.0f64);
    let strip_tx = Signal::new(-400.0f64);
    let drag_tx = strip_tx.clone();
    let drag_width = page_width.clone();
    let drag_state = state.clone();
    let show_summary_drag = show_summary.clone();
""")

# Replace week_view call
content = content.replace('move || week_view(state),', 'move || week_view(state, page_width, strip_tx),')

# Replace summary_view call
content = content.replace('move || summary_view(state),', 'move || summary_view(state, page_width, strip_tx),')

# Update week_view signature
content = re.sub(r'fn week_view\(state: AppState\) -> impl Piece \{\n    let page_width = Signal::new\(400\.0f64\);\n    let strip_tx = Signal::new\(-400\.0f64\);',
r'fn week_view(state: AppState, page_width: Signal<f64>, strip_tx: Signal<f64>) -> impl Piece {', content)

# Update summary_view signature
content = re.sub(r'fn summary_view\(state: AppState\) -> impl Piece \{\n    let page_width = Signal::new\(400\.0f64\);\n    let strip_tx = Signal::new\(-400\.0f64\);',
r'fn summary_view(state: AppState, page_width: Signal<f64>, strip_tx: Signal<f64>) -> impl Piece {', content)

# Remove on_drag from week_view
content = re.sub(r'\.background\(Color::CLEAR\)\n\s*\.on_drag\(pager_drag\(drag_tx, drag_width, drag_state\)\)', '', content)

# Remove on_drag from summary_view
content = re.sub(r'\.background\(Color::CLEAR\)\n\s*\.on_drag\(summary_drag\(drag_tx, drag_width, drag_state\)\)', '', content)

with open('src/pages/diary.rs', 'w') as f:
    f.write(content)

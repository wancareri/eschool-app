import re

with open('src/pages/diary.rs', 'r') as f:
    content = f.read()

content = content.replace("""        match drag.phase {
            DragPhase::Began => {
                axis.set(None);
                strip_tx.set(-w);
            }""",
"""        match drag.phase {
            DragPhase::Began => {
                nslog::nslog("[Drag] Began");
                axis.set(None);
                strip_tx.set(-w);
            }""")

content = content.replace("""            DragPhase::Changed => {
                let mut horiz = axis.get();
                if horiz.is_none() && (dx.abs() > SWIPE_AXIS_LOCK || dy.abs() > SWIPE_AXIS_LOCK) {
                    horiz = Some(dx.abs() >= dy.abs());
                    axis.set(horiz);
                }""",
"""            DragPhase::Changed => {
                let mut horiz = axis.get();
                if horiz.is_none() && (dx.abs() > SWIPE_AXIS_LOCK || dy.abs() > SWIPE_AXIS_LOCK) {
                    horiz = Some(dx.abs() >= dy.abs());
                    nslog::nslog(&format!("[Drag] Axis locked: horiz={:?} dx={} dy={}", horiz, dx, dy));
                    axis.set(horiz);
                }""")
                
content = content.replace("""            DragPhase::Ended => {
                let was_horiz = axis.get() == Some(true);
                axis.set(None);
                let idx = state.current_week_index.get();
                let total = state.all_weeks.get().len() as i32;
                let actual_dx = strip_tx.get() + w;
                if was_horiz && actual_dx.abs() >= SWIPE_THRESHOLD {""",
"""            DragPhase::Ended => {
                let was_horiz = axis.get() == Some(true);
                axis.set(None);
                let idx = state.current_week_index.get();
                let total = state.all_weeks.get().len() as i32;
                let actual_dx = strip_tx.get() + w;
                nslog::nslog(&format!("[Drag] Ended: was_horiz={} actual_dx={} threshold={}", was_horiz, actual_dx, SWIPE_THRESHOLD));
                if was_horiz && actual_dx.abs() >= SWIPE_THRESHOLD {""")

with open('src/pages/diary.rs', 'w') as f:
    f.write(content)

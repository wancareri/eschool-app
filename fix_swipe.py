import re

with open('src/pages/diary.rs', 'r') as f:
    content = f.read()

def replace_ended(match):
    return """DragPhase::Ended => {
                let was_horiz = axis.get() == Some(true);
                axis.set(None);
                let idx = state.current_week_index.get();
                let total = state.all_weeks.get().len() as i32;
                let actual_dx = strip_tx.get() + w;
                if was_horiz && actual_dx.abs() >= SWIPE_THRESHOLD {
                    if actual_dx < 0.0 && idx + 1 < total {
                        features::diary::load_week(state, idx + 1);
                        strip_tx.set(strip_tx.get() + w);
                        with_animation(AnimSpec::ease_out(200), || {
                            strip_tx.set(-w);
                        });
                        return;
                    }
                    if actual_dx > 0.0 && idx > 0 {
                        features::diary::load_week(state, idx - 1);
                        strip_tx.set(strip_tx.get() - w);
                        with_animation(AnimSpec::ease_out(200), || {
                            strip_tx.set(-w);
                        });
                        return;
                    }
                }
                with_animation(AnimSpec::ease_out(200), || {
                    strip_tx.set(-w);
                });
            }"""

def replace_summary_ended(match):
    return """DragPhase::Ended => {
                let was_horiz = axis.get() == Some(true);
                axis.set(None);
                let q = state.current_quarter.get();
                let actual_dx = strip_tx.get() + w;
                if was_horiz && actual_dx.abs() >= SWIPE_THRESHOLD {
                    if actual_dx < 0.0 && q + 1 <= 4 {
                        features::diary::load_quarter(state, q + 1);
                        strip_tx.set(strip_tx.get() + w);
                        with_animation(AnimSpec::ease_out(200), || {
                            strip_tx.set(-w);
                        });
                        return;
                    }
                    if actual_dx > 0.0 && q > 0 {
                        features::diary::load_quarter(state, q - 1);
                        strip_tx.set(strip_tx.get() - w);
                        with_animation(AnimSpec::ease_out(200), || {
                            strip_tx.set(-w);
                        });
                        return;
                    }
                }
                with_animation(AnimSpec::ease_out(200), || {
                    strip_tx.set(-w);
                });
            }"""

parts = content.split('DragPhase::Ended => {')

content = parts[0] + replace_ended(None) + parts[1].split('}', 1)[1]
content = content.split('DragPhase::Ended => {')[0] + replace_summary_ended(None) + content.split('DragPhase::Ended => {')[1].split('}', 1)[1]

# Need to be careful with splitting.

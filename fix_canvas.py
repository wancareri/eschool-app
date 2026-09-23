import re

with open('src/pages/diary.rs', 'r') as f:
    content = f.read()

content = content.replace("""        canvas(move |_draw, size| {
            let w = size.width;
            if w > 1.0 && (probe_width.get() - w).abs() > 0.5 {
                let old_w = probe_width.get();
                let tx = probe_tx.get();
                probe_width.set(w);
                if (tx + old_w).abs() < 2.0 {
                    probe_tx.set(-w);
                }
            }
        })
        .height(0.0)
        .grow(),""",
"""        canvas(move |_draw, size| {
            let w = size.width;
            if w > 1.0 && (probe_width.get() - w).abs() > 0.5 {
                let old_w = probe_width.get();
                let tx = probe_tx.get();
                probe_width.set(w);
                if (tx + old_w).abs() < 2.0 {
                    probe_tx.set(-w);
                }
            }
        })
        .height(0.0),""")

with open('src/pages/diary.rs', 'w') as f:
    f.write(content)

cat << 'INNER' >> src/pages/settings.rs

fn settings_tabs(current_tab: Signal<usize>) -> impl Piece {
    let t1 = current_tab;
    let t2 = current_tab;
    let t3 = current_tab;
    let t4 = current_tab;
    row((
        button(move || if t1.get() == 0 { "[Осн]" } else { "Осн" }).action(move || t1.set(0)),
        button(move || if t2.get() == 1 { "[Вид]" } else { "Вид" }).action(move || t2.set(1)),
        button(move || if t3.get() == 2 { "[Защ]" } else { "Защ" }).action(move || t3.set(2)),
        button(move || if t4.get() == 3 { "[Dev]" } else { "Dev" }).action(move || t4.set(3)),
    ))
    .spacing(8.0)
    .padding(Insets { top: 8.0, leading: 20.0, bottom: 16.0, trailing: 20.0 })
}
INNER

//! A compact wrapping row that centers each line, including the final short line.
use day::prelude::*;
use day_core::{Boundary, BuildCx, Flex, Layout, LayoutOps, PieceSeq, RNode};
use day_spec::Proposal;
use std::rc::Rc;

pub fn centered_wrap(children: Vec<AnyPiece>, spacing: f64, run_spacing: f64) -> impl Piece {
    CenteredWrap {
        children,
        spacing,
        run_spacing,
    }
}
struct CenteredWrap {
    children: Vec<AnyPiece>,
    spacing: f64,
    run_spacing: f64,
}
struct CenteredLayout {
    spacing: f64,
    run_spacing: f64,
}
struct Line {
    children: Vec<(RNode, Size)>,
    width: f64,
    height: f64,
}
impl CenteredLayout {
    fn lines(&self, cx: &mut dyn LayoutOps, children: &[RNode], width: Option<f64>) -> Vec<Line> {
        let mut lines: Vec<Line> = Vec::new();
        for &child in children {
            let size = cx.measure_child(child, Proposal::UNCONSTRAINED);
            let next = lines.last().map(|l| l.width + self.spacing + size.width);
            if lines.is_empty() || next.zip(width).is_some_and(|(next, max)| next > max + 0.5) {
                lines.push(Line {
                    children: Vec::new(),
                    width: 0.0,
                    height: 0.0,
                });
            }
            let line = lines.last_mut().unwrap();
            if !line.children.is_empty() {
                line.width += self.spacing;
            }
            line.width += size.width;
            line.height = line.height.max(size.height);
            line.children.push((child, size));
        }
        lines
    }
}
impl Layout for CenteredLayout {
    fn measure(&self, cx: &mut dyn LayoutOps, children: &[RNode], proposal: Proposal) -> Size {
        let lines = self.lines(cx, children, proposal.width);
        Size::new(
            proposal
                .width
                .unwrap_or_else(|| lines.iter().map(|l| l.width).fold(0.0, f64::max)),
            lines.iter().map(|l| l.height).sum::<f64>()
                + self.run_spacing * lines.len().saturating_sub(1) as f64,
        )
    }
    fn place(&self, cx: &mut dyn LayoutOps, children: &[RNode], bounds: Rect) {
        let mut y = 0.0;
        for line in self.lines(cx, children, Some(bounds.size.width)) {
            let mut x = ((bounds.size.width - line.width) / 2.0).max(0.0);
            for (child, size) in line.children {
                cx.place_child(
                    child,
                    Rect::new(
                        x,
                        y + (line.height - size.height) / 2.0,
                        size.width,
                        size.height,
                    ),
                );
                x += size.width + self.spacing;
            }
            y += line.height + self.run_spacing;
        }
    }
}
impl Piece for CenteredWrap {
    fn build(self, cx: &mut BuildCx) -> RNode {
        let node = cx.native(
            day_spec::kinds::CONTAINER,
            &day_spec::props::ContainerProps::default(),
            Rc::new(CenteredLayout {
                spacing: self.spacing,
                run_spacing: self.run_spacing,
            }),
            Flex::default(),
            Boundary::No,
        );
        cx.under(node, |cx| self.children.build_each(cx));
        node
    }
}

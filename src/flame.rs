use std::collections::{BTreeMap, HashMap};

struct Node {
    count: u64,
    children: BTreeMap<String, Node>,
}

impl Node {
    fn new() -> Node { Node { count: 0, children: BTreeMap::new() } }
    fn insert(&mut self, frames: &[&str], count: u64) {
        self.count += count;
        if let Some((first, rest)) = frames.split_first() {
            self.children.entry((*first).to_string()).or_insert_with(Node::new).insert(rest, count);
        }
    }
}

pub fn folded_to_text(folded: &HashMap<String, u64>) -> String {
    let mut lines: Vec<(String, u64)> = folded.iter().map(|(k, v)| (k.clone(), *v)).collect();
    lines.sort();
    lines.into_iter().map(|(k, v)| format!("{k} {v}")).collect::<Vec<_>>().join("\n")
}

pub fn folded_to_svg(folded: &HashMap<String, u64>) -> String {
    let mut root = Node::new();
    for (path, count) in folded {
        let frames: Vec<&str> = path.split(';').collect();
        root.insert(&frames, *count);
    }

    let total = root.count.max(1);
    let width = 1200.0f64;
    let row_h = 18.0f64;
    let mut max_depth = 0usize;
    measure_depth(&root, 0, &mut max_depth);
    let height = (max_depth as f64 + 1.0) * row_h + 20.0;

    let mut rects = String::new();
    let scale = width / total as f64;
    draw(&root, "मुख्य", 0.0, 0, scale, row_h, height, total, &mut rects);

    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.0}\" height=\"{h:.0}\" \
         font-family=\"monospace\" font-size=\"11\">\n\
         <rect width=\"{w:.0}\" height=\"{h:.0}\" fill=\"#1e1e1e\"/>\n\
         <text x=\"6\" y=\"14\" fill=\"#ddd\">LIPI flame graph — {total} नमूने (samples)</text>\n\
         {rects}</svg>\n",
        w = width, h = height, total = total, rects = rects
    )
}

fn measure_depth(node: &Node, depth: usize, max: &mut usize) {
    if depth > *max { *max = depth; }
    for child in node.children.values() {
        measure_depth(child, depth + 1, max);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw(node: &Node, name: &str, x: f64, depth: usize, scale: f64, row_h: f64, total_h: f64, total: u64, out: &mut String) {
    let w = node.count as f64 * scale;
    let y = total_h - (depth as f64 + 1.0) * row_h;
    let pct = node.count as f64 / total as f64 * 100.0;
    let color = frame_color(name);
    out.push_str(&format!(
        "<g><rect x=\"{x:.2}\" y=\"{y:.2}\" width=\"{w:.2}\" height=\"{rh:.2}\" fill=\"{color}\" stroke=\"#1e1e1e\" stroke-width=\"0.5\"><title>{esc} — {c} ({pct:.1}%)</title></rect>",
        x = x, y = y, w = w.max(0.1), rh = row_h - 1.0, color = color, esc = xml_escape(name), c = node.count, pct = pct
    ));
    if w > 28.0 {
        out.push_str(&format!(
            "<text x=\"{tx:.2}\" y=\"{ty:.2}\" fill=\"#000\" clip-path=\"inset(0)\">{label}</text>",
            tx = x + 2.0, ty = y + row_h - 6.0, label = xml_escape(&truncate(name, (w / 7.0) as usize))
        ));
    }
    out.push_str("</g>\n");
    let mut cx = x;
    for (cname, child) in &node.children {
        draw(child, cname, cx, depth + 1, scale, row_h, total_h, total, out);
        cx += child.count as f64 * scale;
    }
}

fn frame_color(name: &str) -> String {
    let mut h: u32 = 2166136261;
    for b in name.bytes() { h = (h ^ b as u32).wrapping_mul(16777619); }
    let r = 205 + (h % 50) as u8;
    let g = 80 + ((h >> 8) % 120) as u8;
    let b = 40 + ((h >> 16) % 60) as u8;
    format!("rgb({r},{g},{b})")
}

fn truncate(s: &str, max_chars: usize) -> String {
    if max_chars == 0 { return String::new(); }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars { s.to_string() } else { chars[..max_chars.min(chars.len())].iter().collect() }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

pub struct PickerState {
    pub items: Vec<String>,
    pub plain: Vec<String>,
    pub filter: String,
    pub cur: usize,
    pub top: usize,
    pub view: usize,
    pub visible: Vec<usize>,
}

fn strip_ansi(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            while i < bytes.len() {
                let c = bytes[i] as char;
                i += 1;
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            // copy one UTF-8 char
            let ch_len = utf8_len(bytes[i]);
            out.push_str(std::str::from_utf8(&bytes[i..i + ch_len]).unwrap_or(""));
            i += ch_len;
        }
    }
    out
}

fn utf8_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b >> 5 == 0b110 {
        2
    } else if b >> 4 == 0b1110 {
        3
    } else {
        4
    }
}

pub fn new(items: Vec<String>) -> PickerState {
    let plain = items.iter().map(|s| strip_ansi(s)).collect();
    let mut s = PickerState {
        items,
        plain,
        filter: String::new(),
        cur: 0,
        top: 0,
        view: 7,
        visible: Vec::new(),
    };
    s.rebuild_visible();
    s
}

impl PickerState {
    fn rebuild_visible(&mut self) {
        self.visible.clear();
        for i in 0..self.items.len() {
            if self.filter.is_empty() || self.plain[i].contains(&self.filter) {
                self.visible.push(i);
            }
        }
        let nv = self.visible.len();
        if self.cur >= nv && nv > 0 {
            self.cur = nv - 1;
        }
        if nv == 0 {
            self.cur = 0;
        }
        self.top = 0;
    }

    pub fn move_up(&mut self) {
        let nv = self.visible.len();
        if nv == 0 {
            return;
        }
        self.cur = (self.cur + nv - 1) % nv;
        if self.cur == nv - 1 {
            self.top = nv.saturating_sub(self.view);
        } else if self.cur < self.top {
            self.top = self.cur;
        }
    }

    pub fn move_down(&mut self) {
        let nv = self.visible.len();
        if nv == 0 {
            return;
        }
        self.cur = (self.cur + 1) % nv;
        if self.cur >= self.top + self.view {
            self.top = self.cur + 1 - self.view;
        } else if self.cur == 0 {
            self.top = 0;
        }
    }

    pub fn push_filter(&mut self, c: char) {
        self.filter.push(c);
        self.rebuild_visible();
    }

    pub fn backspace(&mut self) {
        self.filter.pop();
        self.rebuild_visible();
    }

    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.rebuild_visible();
    }

    pub fn selected_original(&self) -> Option<usize> {
        self.visible.get(self.cur).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st() -> PickerState {
        new(vec![
            "feat".into(),
            "fix".into(),
            "docs".into(),
            "style".into(),
            "refactor".into(),
            "perf".into(),
            "test".into(),
            "build".into(),
        ])
    }

    #[test]
    fn move_down_wraps() {
        let mut s = st();
        for _ in 0..s.items.len() {
            s.move_down();
        }
        assert_eq!(s.cur, 0);
    }

    #[test]
    fn move_up_from_top_wraps_to_last() {
        let mut s = st();
        s.move_up();
        assert_eq!(s.cur, s.items.len() - 1);
    }

    #[test]
    fn filter_narrows_visible_and_clamps_cur() {
        let mut s = st();
        s.cur = 7;
        s.push_filter('f'); // matches feat, fix, refactor, perf (contains f)
        assert_eq!(
            s.visible
                .iter()
                .map(|&i| s.items[i].clone())
                .collect::<Vec<_>>(),
            vec!["feat", "fix", "refactor", "perf"]
        );
        assert!(s.cur < s.visible.len());
    }

    #[test]
    fn selected_maps_to_original_index() {
        let mut s = st();
        s.push_filter('d'); // "docs", "build" contain d
        s.cur = 1; // second visible = "build" at original index 7
        assert_eq!(s.selected_original(), Some(7));
    }

    #[test]
    fn no_match_yields_none() {
        let mut s = st();
        s.push_filter('z');
        assert!(s.visible.is_empty());
        assert_eq!(s.selected_original(), None);
    }

    #[test]
    fn strips_ansi_for_matching() {
        let s = new(vec!["\x1b[1mfeat\x1b[0m  badge".into()]);
        assert_eq!(s.plain[0], "feat  badge");
    }
}

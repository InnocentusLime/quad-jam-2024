/* 0:25 is reserved for lowercase latin */
const TRIE_UNDERSCORE: usize = 26;
const TRIE_CHILDCOUNT: usize = 27;

struct TrieNode {
    entry: Option<usize>,
    children: [Option<usize>; TRIE_CHILDCOUNT],
}

impl TrieNode {
    fn new() -> Self {
        Self {
            entry: None,
            children: [None; TRIE_CHILDCOUNT],
        }
    }

    fn char_to_child_id(ch: char) -> Option<usize> {
        if 'a' <= ch && ch <= 'z' {
            return Some(ch as usize - 'a' as usize);
        }

        if ch == '_' {
            return Some(TRIE_UNDERSCORE);
        }

        None
    }
}

pub(crate) struct StrTrie {
    nodes: Vec<TrieNode>,
}

impl StrTrie {
    pub(crate) fn new() -> Self {
        Self {
            nodes: vec![TrieNode::new()],
        }
    }

    pub(crate) fn resolve_str(&self, s: &str) -> Option<usize> {
        let mut curr = 0;
        for ch in s.chars() {
            debug_assert!(curr <= self.nodes.len());
            let child_id = TrieNode::char_to_child_id(ch)?;
            curr = self.nodes[curr].children[child_id]?;
        }

        self.nodes[curr].entry
    }

    pub(crate) fn add_entry(&mut self, s: &str, e: usize) -> bool {
        let mut curr = 0;
        for ch in s.chars() {
            debug_assert!(curr <= self.nodes.len());
            let Some(child_id) = TrieNode::char_to_child_id(ch) else {
                return false;
            };
            if let Some(next) = self.nodes[curr].children[child_id] {
                curr = next;
                continue;
            }

            let next = self.nodes.len();
            self.nodes[curr].children[child_id] = Some(next);
            self.nodes.push(TrieNode::new());
            curr = next;
        }

        if self.nodes[curr].entry.is_some() {
            return false;
        }

        self.nodes[curr].entry = Some(e);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::StrTrie;

    #[test]
    fn simple_inserts() {
        let mut trie = StrTrie::new();
        let table = [("sas", 0), ("sa", 1), ("sasa", 2), ("amo", 3), ("a_b", 4)];

        for (s, idx) in table {
            assert!(trie.add_entry(s, idx));
        }

        for (s, idx) in table {
            assert_eq!(trie.resolve_str(s), Some(idx));
        }
    }
}

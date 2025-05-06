/* 0:25 is reserved for lowercase latin */
const TRIE_UNDERSCORE: usize = 26;
const TRIE_CHILDCOUNT: usize = 27;

struct TrieNode {
    entry: Option<usize>,
    children: [Option<usize>; TRIE_CHILDCOUNT],
}

impl TrieNode {
    fn char_to_child_id(&self, ch: char) -> Option<usize> {
        if 'a' <= ch && ch <= 'z' {
            return self.children[ch as usize - 'a' as usize]
        }

        if ch == '_' {
            return self.children[TRIE_UNDERSCORE]
        }

        None
    }
}

pub(crate) struct StrTrie<T> {
    nodes: Vec<TrieNode>,
    entries: Vec<T>,
}

impl<T> StrTrie<T> {
    fn resolve_str(&self, s: &str) -> Option<usize> {
        let mut curr = 0;
        for ch in s.chars() {
            debug_assert!(curr <= self.nodes.len());
            let next = self.nodes[curr].char_to_child_id(ch)?;
            curr = next;
        }

        self.nodes[curr].entry
    }

    pub(crate) fn resolve_entry(&self, s: &str) -> Option<&T> {
        Some(&self.entries[self.resolve_str(s)?])
    }
}
use hashbrown::HashSet;
use regex::Regex;

#[derive(Eq, PartialEq)]
enum Pattern {
    Any,
    Exact(String),
    Multiple(HashSet<String>),
}

enum Rewrite {
    Reference(usize),
    Text(String),
}

struct Edge {
    pattern: Pattern,
    target: usize,
}

enum Action {
    Transition(Edge),
    Rewrite(Vec<Rewrite>),
}

#[derive(Default)]
struct Node {
    actions: Vec<Action>,
}

/// Constructor of a prefix trie that stores rewrite patterns as nodes and
/// rewrite rules as associated values.
pub struct FeatureRewriterBuilder {
    nodes: Vec<Node>,
    ref_pattern: Regex,
}

impl FeatureRewriterBuilder {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            nodes: vec![Node::default()],
            ref_pattern: Regex::new(r"^\$([0-9]+)$").unwrap(),
        }
    }

    #[allow(unused)]
    /// Adds the rewrite rule associated with the pattern.
    /// If the pattern is shorter than the rewrite rule,
    /// the remainings are automatically padded with "*".
    pub fn add_rule<S>(&mut self, pattern: &[S], rewrite: &[S])
    where
        S: AsRef<str>,
    {
        let mut cursor = 0;
        'a: for p in pattern {
            let p = p.as_ref();
            let parsed = if p == "*" {
                Pattern::Any
            } else if p.starts_with('(') && p.ends_with(')') {
                let mut s = HashSet::new();
                for t in p[1..p.len() - 1].split('|') {
                    s.insert(t.to_string());
                }
                Pattern::Multiple(s)
            } else {
                Pattern::Exact(p.to_string())
            };
            for action in &self.nodes[cursor].actions {
                if let Action::Transition(edge) = action {
                    if parsed == edge.pattern {
                        cursor = edge.target;
                        continue 'a;
                    }
                }
            }
            let target = self.nodes.len();
            self.nodes[cursor].actions.push(Action::Transition(Edge {
                pattern: parsed,
                target,
            }));
            self.nodes.push(Node::default());
            cursor = target;
        }
        let mut parsed_rewrite = vec![];
        for p in rewrite {
            let p = p.as_ref();
            parsed_rewrite.push(self.ref_pattern.captures(p).map_or_else(
                || Rewrite::Text(p.to_string()),
                |cap| {
                    let idx = cap.get(1).unwrap().as_str().parse::<usize>().unwrap() - 1;
                    Rewrite::Reference(idx)
                },
            ));
        }
        self.nodes[cursor]
            .actions
            .push(Action::Rewrite(parsed_rewrite));
    }
}

/// Rewriter that maintains rewrite patterns and rules in a prefix trie.
pub struct FeatureRewriter {
    nodes: Vec<Node>,
}

impl From<FeatureRewriterBuilder> for FeatureRewriter {
    fn from(builder: FeatureRewriterBuilder) -> Self {
        Self {
            nodes: builder.nodes,
        }
    }
}

impl FeatureRewriter {
    /// Returns the rewritten features if matched.
    /// If multiple patterns are matched, the earlier registered one is applied.
    #[allow(unused)]
    pub fn rewrite<S>(&self, features: &[S]) -> Option<Vec<String>>
    where
        S: AsRef<str>,
    {
        let mut stack = vec![(0, 0)];
        'a: while let Some((node_idx, edge_idx)) = stack.pop() {
            for (i, action) in self.nodes[node_idx]
                .actions
                .iter()
                .enumerate()
                .skip(edge_idx)
            {
                match action {
                    Action::Transition(edge) => {
                        if let Some(f) = features.get(stack.len()) {
                            let f = f.as_ref();
                            let is_match = match &edge.pattern {
                                Pattern::Any => true,
                                Pattern::Multiple(s) => s.contains(f),
                                Pattern::Exact(s) => f == s,
                            };
                            if is_match {
                                stack.push((node_idx, i));
                                stack.push((edge.target, 0));
                                continue 'a;
                            }
                        }
                    }
                    Action::Rewrite(rule) => {
                        let mut result = vec![];
                        for r in rule {
                            result.push(match r {
                                Rewrite::Reference(idx) => {
                                    features.get(*idx).map_or("*", |s| s.as_ref()).to_string()
                                }
                                Rewrite::Text(s) => s.to_string(),
                            });
                        }
                        return Some(result);
                    }
                }
            }
            if let Some(item) = stack.last_mut() {
                item.1 += 1;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build() {
        let mut builder = FeatureRewriterBuilder::new();
        builder.add_rule(
            &["*", "(助詞|助動詞)", "*", "(よ|ヨ)"],
            &["$1", "$2", "$3", "よ"],
        );
        builder.add_rule(
            &["*", "(助詞|助動詞)", "*", "(無い|ない)"],
            &["$1", "$2", "$3", "ない"],
        );
        builder.add_rule(&["火星", "*", "*", "*"], &["$4", "$3", "$2", "$1"]);
        let rewriter = FeatureRewriter::from(builder);

        assert_eq!(10, rewriter.nodes.len());
    }

    #[test]
    fn test_rewrite_match() {
        let mut builder = FeatureRewriterBuilder::new();
        builder.add_rule(
            &["*", "(助詞|助動詞)", "*", "(よ|ヨ)"],
            &["$1", "$2", "$3", "よ"],
        );
        builder.add_rule(
            &["*", "(助詞|助動詞)", "*", "(無い|ない)"],
            &["$1", "$2", "$3", "ない"],
        );
        builder.add_rule(&["火星", "*", "*", "*"], &["$4", "$3", "$2", "$1"]);
        let rewriter = FeatureRewriter::from(builder);

        assert_eq!(
            Some(vec![
                "よ".to_string(),
                "助詞".to_string(),
                "かな".to_string(),
                "よ".to_string()
            ]),
            rewriter.rewrite(&["よ", "助詞", "かな", "ヨ"]),
        );
        assert_eq!(
            Some(vec![
                "yo".to_string(),
                "助詞".to_string(),
                "ローマ字".to_string(),
                "よ".to_string()
            ]),
            rewriter.rewrite(&["yo", "助詞", "ローマ字", "ヨ"]),
        );
        assert_eq!(
            Some(vec![
                "ない".to_string(),
                "助動詞".to_string(),
                "かな".to_string(),
                "ない".to_string()
            ]),
            rewriter.rewrite(&["ない", "助動詞", "かな", "無い"]),
        );
        assert_eq!(
            Some(vec![
                "かせい".to_string(),
                "漢字".to_string(),
                "名詞".to_string(),
                "火星".to_string()
            ]),
            rewriter.rewrite(&["火星", "名詞", "漢字", "かせい"]),
        );
    }

    #[test]
    fn test_rewrite_match_short() {
        let mut builder = FeatureRewriterBuilder::new();
        builder.add_rule(&["*", "*", "*"], &["$1", "$2", "$4", "$3"]);
        let rewriter = FeatureRewriter::from(builder);

        assert_eq!(
            Some(vec![
                "よ".to_string(),
                "助詞".to_string(),
                "かな".to_string(),
                "よ".to_string()
            ]),
            rewriter.rewrite(&["よ", "助詞", "よ", "かな"]),
        );
    }

    #[test]
    fn test_rewrite_fail() {
        let mut builder = FeatureRewriterBuilder::new();
        builder.add_rule(
            &["*", "(助詞|助動詞)", "*", "(よ|ヨ)"],
            &["$1", "$2", "$3", "よ"],
        );
        builder.add_rule(
            &["*", "(助詞|助動詞)", "*", "(無い|ない)"],
            &["$1", "$2", "$3", "ない"],
        );
        builder.add_rule(&["火星", "*", "*", "*"], &["$4", "$3", "$2", "$1"]);
        let rewriter = FeatureRewriter::from(builder);

        assert_eq!(None, rewriter.rewrite(&["よ", "助詞", "かな", "yo"]));
        assert_eq!(None, rewriter.rewrite(&["火星", "名詞", "漢字"]));
    }

    #[test]
    fn test_rewrite_match_mostfirst() {
        let mut builder1 = FeatureRewriterBuilder::new();
        builder1.add_rule(
            &["*", "(助詞|助動詞)", "*", "(よ|ヨ)"],
            &["$1", "$2", "$3", "よ"],
        );
        builder1.add_rule(
            &["*", "(助詞|助動詞)", "*", "(無い|ない)"],
            &["$1", "$2", "$3", "ない"],
        );
        builder1.add_rule(&["火星", "*", "*", "*"], &["$4", "$3", "$2", "$1"]);
        let rewriter1 = FeatureRewriter::from(builder1);

        assert_eq!(
            Some(vec![
                "火星".to_string(),
                "助詞".to_string(),
                "かな".to_string(),
                "よ".to_string()
            ]),
            rewriter1.rewrite(&["火星", "助詞", "かな", "よ"]),
        );

        let mut builder2 = FeatureRewriterBuilder::new();
        builder2.add_rule(&["火星", "*", "*", "*"], &["$4", "$3", "$2", "$1"]);
        builder2.add_rule(
            &["*", "(助詞|助動詞)", "*", "(よ|ヨ)"],
            &["$1", "$2", "$3", "よ"],
        );
        builder2.add_rule(
            &["*", "(助詞|助動詞)", "*", "(無い|ない)"],
            &["$1", "$2", "$3", "ない"],
        );
        let rewriter2 = FeatureRewriter::from(builder2);

        assert_eq!(
            Some(vec![
                "よ".to_string(),
                "かな".to_string(),
                "助詞".to_string(),
                "火星".to_string()
            ]),
            rewriter2.rewrite(&["火星", "助詞", "かな", "よ"]),
        );
    }

    #[test]
    fn test_rewrite_match_mostfirst_long_short() {
        let mut builder = FeatureRewriterBuilder::new();
        builder.add_rule(&["*", "*", "*", "*"], &["$1", "$2", "$3", "$4"]);
        builder.add_rule(&["*", "*"], &["$1", "$2", "*", "*"]);
        let rewriter = FeatureRewriter::from(builder);

        assert_eq!(
            Some(vec![
                "火星".to_string(),
                "助詞".to_string(),
                "かな".to_string(),
                "よ".to_string(),
            ]),
            rewriter.rewrite(&["火星", "助詞", "かな", "よ"]),
        );
        assert_eq!(
            Some(vec![
                "火星".to_string(),
                "助詞".to_string(),
                "*".to_string(),
                "*".to_string(),
            ]),
            rewriter.rewrite(&["火星", "助詞", "かな"]),
        );
    }

    #[test]
    fn test_invalid_index() {
        let mut builder = FeatureRewriterBuilder::new();
        builder.add_rule(
            &["*", "(助詞|助動詞)", "*", "(よ|ヨ)"],
            &["$1", "$2", "$5", "よ"],
        );
        let rewriter = FeatureRewriter::from(builder);

        assert_eq!(
            Some(vec![
                "火星".to_string(),
                "助詞".to_string(),
                "*".to_string(),
                "よ".to_string()
            ]),
            rewriter.rewrite(&["火星", "助詞", "かな", "よ"]),
        );
    }
}

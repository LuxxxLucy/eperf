// todo: turn on the dead code warning
#![allow(dead_code)]

pub mod error;
pub mod prelude;

type RuleIndex = usize;
type RewriteIndex = usize;
type EClassIndex = usize;

struct Rule {
    repr: String,
}

struct Match {
    left: usize,
    right: usize,
}

struct Rewrite {
    rule: RuleIndex,
    subs: Match,
}

struct Merge {
    rewrite_id: RewriteIndex,
    id1: EClassIndex,
    id2: EClassIndex,
}

struct Arena {
    rules: Vec<Rule>,
    rewrites: Vec<Rewrite>,
    merge: Vec<Merge>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(0, 0);
    }
}

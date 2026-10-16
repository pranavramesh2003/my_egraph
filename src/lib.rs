pub mod util;

use crate::util::*;
use std::collections::HashMap;
use crate::union_find::*;
use std::iter::zip;
use std::collections::HashSet; 
#[derive(Default, PartialEq, Eq, Hash)]
pub struct Node {
    f: String, 
    l: Vec<Id>,
}

impl Node {
    pub fn Leaf(name: String) -> Node {
        Node { f : name, l : Vec::new() }
    }
}

//E-matching definitions
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Pattern{
    Var(String),
    App(String, Vec<Pattern>),
}

//new type for substitutions
type Substitution = HashMap<String, Id>;

//The rewrite struct
pub struct Rewrite {
    lhs: Pattern, 
    rhs: Pattern,
}

//The match struct
pub struct Match {
    rhs: Pattern,
    root: Id, 
    subst: Substitution,
}


pub struct Egraph {
    union_find: UnionFind,
    memo: HashMap<Node, Id>,
}

impl Default for Egraph {
    fn default() -> Egraph {
        Egraph{
            union_find: UnionFind::default(),
            memo: HashMap::new(),
        }
    }
}

impl Egraph {
    pub fn add(&mut self, node: Node) -> Id{
        let node = self.canonicalize_node(&node);
        match self.memo.get(&node) {
           Some(id) => *id,
           None => { 
           let new_id = self.union_find.makeset();
           self.memo.insert(node, new_id);
           new_id
           }
       }
    }

    pub fn union(&mut self, a: Id, b: Id) -> bool {
        self.union_find.union(a,b)
    }
    
    fn canonicalize_node(&self, n: &Node) -> Node {
        let mut child_vec = Vec::new();
        for child in &n.l {
            child_vec.push(self.union_find.find(*child));
        }
        Node {f : n.f.clone(), l: child_vec}
    }
    pub fn rebuild(&mut self) {
        loop {
            let mut keep_going = false;
            let old_memo = std::mem::take(&mut self.memo);
            self.memo = HashMap::new();
            for (old_node, old_id) in old_memo
            {
                let new_node = self.canonicalize_node(&old_node);
                let new_id = match self.memo.get(&new_node) {
                    Some(id) => *id,
                    None => {
                        let new_id = self.union_find.find(old_id);
                        self.memo.insert(new_node, new_id);
                        new_id
                    }
                };
                if self.union(old_id, new_id) {
                    keep_going = true; 
                }
            }
            if !keep_going {
                break;
            }
        }
    }

    pub fn instantiate(&mut self, pat: Pattern, subst: &Substitution) -> Id {
        match pat {
            Pattern::Var(name) => subst[&name],
            Pattern::App(f,l) => {
                let mut subpat_vec = vec![];
                for subpat in l
                {
                    subpat_vec.push(self.instantiate(subpat, subst));
                }
                let node = Node {f : f, l: subpat_vec};
                self.add(node)
            }
        }
    }
    
    fn nodes_in_class(&self, root: Id) -> Vec<&Node> {
        let mut out = vec![];
        for (node,id) in &self.memo {
            if *id == root {
                out.push(node);
            }
        }
        out
    }
    fn ematch_rec(&self, pat: &Pattern, root: Id, subst: Substitution) -> Vec<Substitution> {
        match pat {
            Pattern::Var(name) => {
                match subst.get(name) {
                    Some(id) => {
                        if *id == root { vec![subst] }
                        else { vec![] }
                    }
                    None => {
                        let mut map_toreturn = HashMap::clone(&subst);
                        map_toreturn.insert(name.clone(), root);
                        vec![map_toreturn]
                    }
                }
            },
            Pattern::App(f,subpats) => {
                let mut outputs = vec![];
                for node in self.nodes_in_class(root) {
                    if node.f != *f { continue; }
                    if node.l.len() != subpats.len() { continue; }
                    let mut todo = vec![subst.clone()];
                    for (sub_id, sub_pat) in zip(node.l.clone(), subpats) {
                        let mut results = vec![];
                        for new_subst in todo {
                            results.extend(self.ematch_rec(sub_pat, sub_id, new_subst));
                        }
                        todo = results;
                    }
                    outputs.extend(todo);
                }
                outputs
            }
        }
    }

    pub fn ematch(&self, pat: &Pattern, root: Id) -> Vec<Substitution> {
        self.ematch_rec(pat, root, HashMap::new())
    }

    fn all_ids(&self) -> HashSet<Id> {
        self.memo.values().copied().collect()
    }

    pub fn rewrite(&mut self, rules: &Vec<Rewrite>) {
        let mut matches = vec![];
        for rule in rules {
            for id in self.all_ids() {
                let substs = self.ematch(&rule.lhs, id);
                for subst in substs {
                    matches.push(Match{rhs: rule.rhs.clone(), root: id, subst: subst});
                }
            }
        }
        for match_obj in matches {
            let new_id = self.instantiate(match_obj.rhs, &match_obj.subst);
            self.union(match_obj.root, new_id);
        }
    }

    pub fn saturate(&mut self, rules: &Vec<Rewrite>) {
        self.rebuild();
        let mut fingerprint = (self.union_find.parent.len(), self.memo.len());
        loop {
            self.rewrite(rules);
            self.rebuild();
            let new_fingerprint = (self.union_find.parent.len(), self.memo.len());
            if fingerprint == new_fingerprint { break; }
            else { fingerprint = new_fingerprint; }
        }
    }

}




#[cfg(test)]
mod test {

        use super::*;
        use crate::Pattern::*; 
        #[test]
        fn test1_egraph() {
            let mut test_eg = Egraph::default();
            let a = Node::Leaf(String::from("a"));
            let a_copy  = Node::Leaf(String::from("a"));
            let b = Node::Leaf(String::from("b"));
            let a_id = test_eg.add(a);
            let a_copy_id = test_eg.add(a_copy);
            let b_id = test_eg.add(b);
            assert_eq!(a_id, a_copy_id);
            assert_ne!(a_id, b_id);  // tests for simple add operation
            let f_a = Node { f: String::from("f"), l: vec![a_id] };
            let f_b =  Node { f: String::from("f"), l: vec![b_id] };
            let f_a_id = test_eg.add(f_a);
            let f_b_id = test_eg.add(f_b);
            test_eg.union(a_id, b_id);
            test_eg.rebuild();
            assert_eq!(test_eg.union_find.find(a_id), test_eg.union_find.find(b_id));
            assert_eq!(test_eg.union_find.find(f_a_id), test_eg.union_find.find(f_b_id));                                                                    
        }
        
        #[test]
        fn test_rewrite() {
            let mut test_eg = Egraph::default();
            let root = test_eg.instantiate(
            App(String::from("+"), vec![
                App(String::from("+"), vec![
                    App(String::from("+"), vec![
                        App(String::from("a"), vec![]),
                        App(String::from("b"), vec![]),
                    ]),
                    App(String::from("+"), vec![
                        App(String::from("c"), vec![]),
                        App(String::from("d"), vec![]),
                    ]),
                ]),
                App(String::from("+"), vec![
                    App(String::from("+"), vec![
                        App(String::from("e"), vec![]),
                        App(String::from("f"), vec![]),
                    ]),
                    App(String::from("g"), vec![]),
                ]),
            ]),
            &HashMap::new());
            let rules = vec![
            Rewrite {
                lhs : App(String::from("+"), vec![Var(String::from("a")), Var(String::from("b"))]),
                rhs : App(String::from("+"), vec![Var(String::from("b")), Var(String::from("a"))])
                },
            Rewrite{
                lhs : App(String::from("+"), vec![
                    App(String::from("+"), vec![Var(String::from("a")), Var(String::from("b"))]),
                    Var(String::from("c"))
                ]),
                rhs : App(String::from("+"), vec![
                    Var(String::from("a")),
                    App(String::from("+"), vec![Var(String::from("b")), Var(String::from("c"))])
                ])}
            ];
            test_eg.saturate(&rules);
            assert_eq!((test_eg.all_ids().len()), 127);
            assert_eq!((test_eg.memo).len(), 1939);
        }
}

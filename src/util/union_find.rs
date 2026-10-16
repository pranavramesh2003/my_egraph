#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, PartialOrd, Ord)]
pub struct Id(u32);

impl Id {
    pub fn new(n: usize) -> Self {
        Id(n as u32)
    }

    pub fn usize(self) -> usize {
        self.0 as usize
    }
}

pub struct UnionFind {
    pub parent: Vec<Id>,
}


impl Default for UnionFind {
    fn default() -> UnionFind {
        UnionFind{
            parent: Vec::new(),
        }
    }
}

impl UnionFind {
    pub fn len(&self) -> usize {
        self.parent.len()
    }
    pub fn makeset(&mut self) -> Id {
        let new_id = self.len();
        self.parent.push(Id::new(new_id));
        Id::new(new_id)
    }

    pub fn find(&self, id: Id) -> Id {
        if self.parent[id.usize()] == id
        {
            id
        }
        else
        {
            self.find(self.parent[id.usize()])
        }
    }

    pub fn union(&mut self, a: Id, b: Id) -> bool {
        let a = self.find(a);
        let b = self.find(b);
        if a == b { false }
        else { self.parent[b.usize()] = a; true} //return true, choose b's parent to be a. Note
                                                 //that this automatically performs path
                                                 //compression by setting the parent equal to the
                                                 //canonical node
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test1_unionfind() {
        let mut test_uf = UnionFind::default();
        let a = test_uf.makeset();
        let b = test_uf.makeset();
        let c = test_uf.makeset();
        test_uf.union(a, b);
        test_uf.union(b, c);
        assert_eq!(test_uf.find(a), test_uf.find(c));
    }
}

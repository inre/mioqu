pub trait Index {
    fn from_usize(i: usize) -> Self;
    fn as_usize(&self) -> usize;
}

#[derive(Debug)]
pub struct Fragments<E, I: Index> {
    list: Vec<Option<E>>,
    free: Vec<I>,
}

impl<E, I> Fragments<E, I> where I: Index {
    pub fn new() -> Self {
        Fragments {
            list: Vec::new(),
            free: Vec::new()
        }
    }

    pub fn elem(&mut self, index: I) -> &Option<E> {
        &self.list[index.as_usize()]
    }

    pub fn elem_mut(&mut self, index: I) -> &mut Option<E> {
        &mut self.list[index.as_usize()]
    }

    pub fn add(&mut self, elem: E) -> I {
        match self.free.pop() {
            Some(index) => {
                self.list[index.as_usize()] = Some(elem);
                index
            },
            None => {
                self.list.push(Some(elem));
                I::from_usize(self.list.len()-1)
            }
        }
    }

    pub fn delete(&mut self, index: I) {
        let idx = index.as_usize();
        match self.list[idx] {
            Some(_) => {
                self.list[idx] = None;
                self.free.push(index);
            },
            None => ()
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Fragments, Index};

    struct Complex {
        value: usize
    }

    #[derive(Copy, Clone)]
    struct Token(usize);

    impl Index for Token {
        fn from_usize(val: usize) -> Token {
            Token(val)
        }

        fn as_usize(&self) -> usize {
            let Token(val) = *self;
            val
        }
    }

    #[test]
    fn read_and_modify_elements() {
        let mut frag = Fragments::new();
        let token: Token = frag.add(Box::new(Complex { value: 1 }));
        {
            let mut compl = frag.elem_mut(token.clone()).as_mut().unwrap();
            compl.value = 3;
        }
        {
            let compl = frag.elem(token.clone()).as_ref().unwrap();
            assert_eq!(compl.value, 3);
        }
    }

    #[test]
    fn fill_voids() {
        let mut frag = Fragments::new();

        let t1: Token = frag.add(Complex { value: 1 });
        let t2: Token = frag.add(Complex { value: 2 });
        frag.delete(t1);
        let t3: Token = frag.add(Complex { value: 2 });
        assert_eq!(t1.0, t3.0);
        assert_eq!(t3.0, 0);
        assert_eq!(t2.0, 1);
    }
}

use std::collections::HashSet;
use std::rc::{Rc, Weak};
use std::hash::{Hash, Hasher};
pub trait Hashable = Hash + PartialEq;

pub struct WeakElem<T : Hashable> {
    elem : Weak<T>
}

pub struct WeakRef<T : Hashable> {
    elem : Rc<T>,
    owner : Rc<WeakSet<T>>
}

impl<T : Hashable> Drop for WeakRef<T> {
    fn drop(&mut self) {
        unsafe {
            // no worries because it will happen synchronously
            Rc::get_mut_unchecked(&mut self.owner).data.remove(&WeakElem { elem : Rc::downgrade(&self.elem) });
        }
    }
}

impl<T : Hashable> Hash for WeakElem<T> {
    fn hash<H : Hasher>(&self, state : &mut H) {
        match self.elem.upgrade() {
            Some(rc) => rc.hash(state),
            None => Option::<Rc<T>>::None.hash(state)
        }
    }
}

impl<T: Hashable> PartialEq for WeakElem<T> {
    fn eq(&self, other : &Self) -> bool {
        let lhs = self.elem.upgrade();
        let rhs = other.elem.upgrade();
        (match lhs {
            Some(left) => {
                match rhs {
                    Some(right) => left == right,
                    None => false
                }
            },
            None => {
                match rhs {
                    Some(_) => false,
                    None => true
                }
            }
        })
    }
}

impl<T : Hashable> Eq for WeakElem<T> {}

pub struct WeakSet<T : Hashable> {
    data : HashSet<WeakElem<T>>
}

impl<T : Hashable> WeakSet<T> {
    pub fn empty() -> Rc<Self> {
        Rc::new(Self { data : HashSet::new() })
    }

    pub fn insert(mut self : &mut Rc<Self>, elem : T) -> WeakRef<T> {
        let elem = Rc::new(elem);
        unsafe {
            Rc::get_mut_unchecked(&mut self).data.insert(WeakElem { elem : Rc::downgrade(&elem) });
        }
        WeakRef { elem : elem, owner : self.clone() }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Rc<T>> + 'a {
        self.data.iter().map(|x| x.elem.upgrade().unwrap()) // Unwrapping here is fine because it's contextually guaranteed to work. So there is no need to filter !is_none out
    }
}

/**** Tests ****/
#[cfg(test)]
mod tests {
    #[test]
    fn ins_n_rem() {
        use super::*;

        let mut collection = WeakSet::empty();
        let _keepref = collection.insert(1); // keeps ref until the end of scope
        assert_eq!(collection.len(), 1);

        {
            let _keepref = {
                let _keepref = collection.insert(2);
                assert_eq!(collection.len(), 2);

                let keepref = collection.insert(3);
                assert_eq!(collection.len(), 3);
                keepref
            };
            assert_eq!(collection.len(), 2);
        }
        assert_eq!(collection.len(), 1);

        let _keepref = collection.insert(4);
        assert_eq!(collection.len(), 2);

        for item in collection.iter() {
            println!("{}", item)
        }
    }
}

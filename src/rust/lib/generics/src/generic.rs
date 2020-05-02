
use super::hlist;



pub trait HasRepr {
    type GenericRepr : hlist::HList;
}

pub type Repr<T> = <T as HasRepr>::GenericRepr;

pub trait IntoGeneric : HasRepr + Into<Repr<Self>> {
    fn into_generic(self) -> Repr<Self> {
        self.into()
    }
}

impl<T> IntoGeneric for T where T : HasRepr + Into<Repr<T>> {}




// === HasFieldsCount ===

/// Information of field count of any structure implementing `Generics`.
#[allow(missing_docs)]
pub trait HasFieldsCount {
    const FIELDS_COUNT : usize;
    fn fields_count() -> usize {
        Self::FIELDS_COUNT
    }
}

impl<T> HasFieldsCount for T
where T:HasRepr {
    const FIELDS_COUNT : usize = <Repr<T> as hlist::HasLength>::LEN;
}

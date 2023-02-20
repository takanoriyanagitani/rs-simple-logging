use crate::Item;

pub trait Serialize: Sync + Send {
    fn serialize(&self, item: &Item, buf: &mut String);
}

struct FnSer<S> {
    internal: S,
}

impl<S> Serialize for FnSer<S>
where
    S: Fn(&Item, &mut String) + Sync + Send,
{
    fn serialize(&self, item: &Item, buf: &mut String) {
        (self.internal)(item, buf)
    }
}

pub fn serializer_new_from_fn<S>(internal: S) -> impl Serialize
where
    S: Fn(&Item, &mut String) + Sync + Send,
{
    FnSer { internal }
}

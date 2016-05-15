use super::{Callback, Consumer, Enumerate, Hide, Split, SplitIterator, ExactSizeSplitIterator};

pub struct EnumerateConsumer<T>(T);

impl<T: ExactSizeSplitIterator> SplitIterator for Enumerate<T> {
    type Item = (usize, T::Item);
    type Base = Hide<Enumerate<T::Base>>;
    type Consumer = EnumerateConsumer<T::Consumer>;

    fn destructure(self) -> (Self::Base, Self::Consumer) {
        let (b, c) = self.parent.destructure();

        (Hide(Enumerate { parent: b, off: self.off }), EnumerateConsumer(c))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.parent.size_hint()
    }
}

impl<T: ExactSizeSplitIterator> ExactSizeSplitIterator for Enumerate<T> {
    fn size(&self) -> usize {
        self.parent.size()
    }
}

struct EnumerateCallback<C> {
    cb: C,
    off: usize,
}

impl<Item, C: Callback<(usize, Item)>> Callback<Item> for EnumerateCallback<C> {
    type Out = C::Out;

    fn call<I: Iterator<Item=Item>>(self, iter: I) -> C::Out {
        let off = self.off;
        self.cb.call(iter.enumerate().map(|(i, x)| (i + off, x)))
    }
}

impl<In: IntoIterator, T: Consumer<In>> Consumer<Hide<Enumerate<In>>> for EnumerateConsumer<T> {
    type Item = (usize, T::Item);

    fn consume<C: Callback<Self::Item>>(&self, i: Hide<Enumerate<In>>, cb: C) -> C::Out {
        let cb = EnumerateCallback {
            cb: cb,
            off: i.0.off,
        };

        self.0.consume(i.0.parent, cb)
    }
}

impl<T: IntoIterator> IntoIterator for Hide<Enumerate<T>> {
    type Item = T::Item;
    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.parent.into_iter()
    }
}

impl<T: Split> Split for Hide<Enumerate<T>> {
    fn should_split(&self) -> Option<usize> {
        self.0.parent.should_split()
    }

    fn split(self, idx: usize) -> (Self, Self) {
        let (a, b) = self.0.parent.split(idx);
        let base_off = self.0.off;

        (
            Hide(Enumerate { parent: a, off: base_off }),
            Hide(Enumerate { parent: b, off: base_off + idx }),
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.parent.size_hint()
    }
}
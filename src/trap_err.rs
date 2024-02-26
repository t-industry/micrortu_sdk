/*!
Helper trait for trapping errors in an iterator.

# Example

```rust
use micrortu_sdk::trap_err::TrapErrExt;

let mut trap = Ok(());
let mut iter = vec![Ok(1), Err(2), Ok(3)].into_iter().trap_err(&mut trap);
let mut result: Vec<i32> = iter.collect();
assert_eq!(result, vec![1]);
assert_eq!(trap, Err(2));
```
*/
pub trait TrapErrExt: Sized + Iterator {
    type Trap;
    fn trap_err(self, trap: &mut Result<(), Self::Trap>) -> TrapErrIter<Self, Self::Trap>;
}

impl<It, T, E> TrapErrExt for It
where
    It: Iterator<Item = Result<T, E>>,
{
    type Trap = E;

    fn trap_err(self, trap: &mut Result<(), Self::Trap>) -> TrapErrIter<Self, Self::Trap> {
        TrapErrIter {
            iter: Some(self),
            trap,
        }
    }
}

pub struct TrapErrIter<'a, It, Trap: 'a> {
    iter: Option<It>,
    trap: &'a mut Result<(), Trap>,
}

impl<'a, It, T, E> Iterator for TrapErrIter<'a, It, E>
where
    It: Iterator<Item = Result<T, E>>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let trapped = {
            let iter = self.iter.as_mut()?;

            match iter.next() {
                Some(Ok(e)) => return Some(e),
                Some(Err(err)) => Err(err),
                None => Ok(()),
            }
        };

        self.iter = None;
        *self.trap = trapped;
        None
    }
}

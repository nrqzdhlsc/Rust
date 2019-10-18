//! A module for working with borrowed data.

#![stable(feature = "rust1", since = "1.0.0")]

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::ops::{Add, AddAssign, Deref};

#[stable(feature = "rust1", since = "1.0.0")]
pub use core::borrow::{Borrow, BorrowMut};

use crate::fmt;
use crate::string::String;

use Cow::*;

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, B: ?Sized> Borrow<B> for Cow<'a, B>
    where B: ToOwned,
          <B as ToOwned>::Owned: 'a
{
    fn borrow(&self) -> &B {
        &**self
    }
}

/// A generalization of `Clone` to borrowed data.
///
/// Some types make it possible to go from borrowed to owned, usually by
/// implementing the `Clone` trait. But `Clone` works only for going from `&T`
/// to `T`. The `ToOwned` trait generalizes `Clone` to construct owned data
/// from any borrow of a given type.
#[stable(feature = "rust1", since = "1.0.0")]
pub trait ToOwned {
    /// The resulting type after obtaining ownership.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Owned: Borrow<Self>;

    /// Creates owned data from borrowed data, usually by cloning.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let s: &str = "a";
    /// let ss: String = s.to_owned();
    ///
    /// let v: &[i32] = &[1, 2];
    /// let vv: Vec<i32> = v.to_owned();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[must_use = "cloning is often expensive and is not expected to have side effects"]
    fn to_owned(&self) -> Self::Owned;

    /// Uses borrowed data to replace owned data, usually by cloning.
    ///
    /// This is borrow-generalized version of `Clone::clone_from`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # #![feature(toowned_clone_into)]
    /// let mut s: String = String::new();
    /// "hello".clone_into(&mut s);
    ///
    /// let mut v: Vec<i32> = Vec::new();
    /// [1, 2][..].clone_into(&mut v);
    /// ```
    #[unstable(feature = "toowned_clone_into",
               reason = "recently added",
               issue = "41263")]
    fn clone_into(&self, target: &mut Self::Owned) {
        *target = self.to_owned();
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> ToOwned for T
    where T: Clone
{
    type Owned = T;
    fn to_owned(&self) -> T {
        self.clone()
    }

    fn clone_into(&self, target: &mut T) {
        target.clone_from(self);
    }
}

/// A clone-on-write smart pointer.
///
/// The type `Cow` is a smart pointer providing clone-on-write functionality: it
/// can enclose and provide immutable access to borrowed data, and clone the
/// data lazily when mutation or ownership is required. The type is designed to
/// work with general borrowed data via the `Borrow` trait.
///
/// `Cow` implements `Deref`, which means that you can call
/// non-mutating methods directly on the data it encloses. If mutation
/// is desired, `to_mut` will obtain a mutable reference to an owned
/// value, cloning if necessary.
///
/// # Examples
///
/// ```
/// use std::borrow::Cow;
///
/// fn abs_all(input: &mut Cow<[i32]>) {
///     for i in 0..input.len() {
///         let v = input[i];
///         if v < 0 {
///             // Clones into a vector if not already owned.
///             input.to_mut()[i] = -v;
///         }
///     }
/// }
///
/// // No clone occurs because `input` doesn't need to be mutated.
/// let slice = [0, 1, 2];
/// let mut input = Cow::from(&slice[..]);
/// abs_all(&mut input);
///
/// // Clone occurs because `input` needs to be mutated.
/// let slice = [-1, 0, 1];
/// let mut input = Cow::from(&slice[..]);
/// abs_all(&mut input);
///
/// // No clone occurs because `input` is already owned.
/// let mut input = Cow::from(vec![-1, 0, 1]);
/// abs_all(&mut input);
/// ```
///
/// Another example showing how to keep `Cow` in a struct:
///
/// ```
/// use std::borrow::Cow;
///
/// struct Items<'a, X: 'a> where [X]: ToOwned<Owned = Vec<X>> {
///     values: Cow<'a, [X]>,
/// }
///
/// impl<'a, X: Clone + 'a> Items<'a, X> where [X]: ToOwned<Owned = Vec<X>> {
///     fn new(v: Cow<'a, [X]>) -> Self {
///         Items { values: v }
///     }
/// }
///
/// // Creates a container from borrowed values of a slice
/// let readonly = [1, 2];
/// let borrowed = Items::new((&readonly[..]).into());
/// match borrowed {
///     Items { values: Cow::Borrowed(b) } => println!("borrowed {:?}", b),
///     _ => panic!("expect borrowed value"),
/// }
///
/// let mut clone_on_write = borrowed;
/// // Mutates the data from slice into owned vec and pushes a new value on top
/// clone_on_write.values.to_mut().push(3);
/// println!("clone_on_write = {:?}", clone_on_write.values);
///
/// // The data was mutated. Let check it out.
/// match clone_on_write {
///     Items { values: Cow::Owned(_) } => println!("clone_on_write contains owned data"),
///     _ => panic!("expect owned data"),
/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub enum Cow<'a, B: ?Sized + 'a>
    where B: ToOwned
{
    /// Borrowed data.
    #[stable(feature = "rust1", since = "1.0.0")]
    Borrowed(#[stable(feature = "rust1", since = "1.0.0")]
             &'a B),

    /// Owned data.
    #[stable(feature = "rust1", since = "1.0.0")]
    Owned(#[stable(feature = "rust1", since = "1.0.0")]
          <B as ToOwned>::Owned),
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<B: ?Sized + ToOwned> Clone for Cow<'_, B> {
    fn clone(&self) -> Self {
        match *self {
            Borrowed(b) => Borrowed(b),
            Owned(ref o) => {
                let b: &B = o.borrow();
                Owned(b.to_owned())
            }
        }
    }

    fn clone_from(&mut self, source: &Self) {
        if let Owned(ref mut dest) = *self {
            if let Owned(ref o) = *source {
                o.borrow().clone_into(dest);
                return;
            }
        }

        *self = source.clone();
    }
}

impl<B: ?Sized + ToOwned> Cow<'_, B> {
    /// Acquires a mutable reference to the owned form of the data.
    ///
    /// Clones the data if it is not already owned.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::borrow::Cow;
    ///
    /// let mut cow = Cow::Borrowed("foo");
    /// cow.to_mut().make_ascii_uppercase();
    ///
    /// assert_eq!(
    ///   cow,
    ///   Cow::Owned(String::from("FOO")) as Cow<str>
    /// );
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn to_mut(&mut self) -> &mut <B as ToOwned>::Owned {
        match *self {
            Borrowed(borrowed) => {
                *self = Owned(borrowed.to_owned());
                match *self {
                    Borrowed(..) => unreachable!(),
                    Owned(ref mut owned) => owned,
                }
            }
            Owned(ref mut owned) => owned,
        }
    }

    /// Extracts the owned data.
    ///
    /// Clones the data if it is not already owned.
    ///
    /// # Examples
    ///
    /// Calling `into_owned` on a `Cow::Borrowed` clones the underlying data
    /// and becomes a `Cow::Owned`:
    ///
    /// ```
    /// use std::borrow::Cow;
    ///
    /// let s = "Hello world!";
    /// let cow = Cow::Borrowed(s);
    ///
    /// assert_eq!(
    ///   cow.into_owned(),
    ///   String::from(s)
    /// );
    /// ```
    ///
    /// Calling `into_owned` on a `Cow::Owned` is a no-op:
    ///
    /// ```
    /// use std::borrow::Cow;
    ///
    /// let s = "Hello world!";
    /// let cow: Cow<str> = Cow::Owned(String::from(s));
    ///
    /// assert_eq!(
    ///   cow.into_owned(),
    ///   String::from(s)
    /// );
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn into_owned(self) -> <B as ToOwned>::Owned {
        match self {
            Borrowed(borrowed) => borrowed.to_owned(),
            Owned(owned) => owned,
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<B: ?Sized + ToOwned> Deref for Cow<'_, B> {
    type Target = B;

    fn deref(&self) -> &B {
        match *self {
            Borrowed(borrowed) => borrowed,
            Owned(ref owned) => owned.borrow(),
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<B: ?Sized> Eq for Cow<'_, B> where B: Eq + ToOwned {}

#[stable(feature = "rust1", since = "1.0.0")]
impl<B: ?Sized> Ord for Cow<'_, B>
    where B: Ord + ToOwned
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, 'b, B: ?Sized, C: ?Sized> PartialEq<Cow<'b, C>> for Cow<'a, B>
    where B: PartialEq<C> + ToOwned,
          C: ToOwned
{
    #[inline]
    fn eq(&self, other: &Cow<'b, C>) -> bool {
        PartialEq::eq(&**self, &**other)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, B: ?Sized> PartialOrd for Cow<'a, B>
    where B: PartialOrd + ToOwned
{
    #[inline]
    fn partial_cmp(&self, other: &Cow<'a, B>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<B: ?Sized> fmt::Debug for Cow<'_, B>
where
    B: fmt::Debug + ToOwned<Owned: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Borrowed(ref b) => fmt::Debug::fmt(b, f),
            Owned(ref o) => fmt::Debug::fmt(o, f),
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<B: ?Sized> fmt::Display for Cow<'_, B>
where
    B: fmt::Display + ToOwned<Owned: fmt::Display>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Borrowed(ref b) => fmt::Display::fmt(b, f),
            Owned(ref o) => fmt::Display::fmt(o, f),
        }
    }
}

#[stable(feature = "default", since = "1.11.0")]
impl<B: ?Sized> Default for Cow<'_, B>
where
    B: ToOwned<Owned: Default>,
{
    /// Creates an owned Cow<'a, B> with the default value for the contained owned value.
    fn default() -> Self {
        Owned(<B as ToOwned>::Owned::default())
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<B: ?Sized> Hash for Cow<'_, B>
    where B: Hash + ToOwned
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized + ToOwned> AsRef<T> for Cow<'_, T> {
    fn as_ref(&self) -> &T {
        self
    }
}

#[stable(feature = "cow_add", since = "1.14.0")]
impl<'a> Add<&'a str> for Cow<'a, str> {
    type Output = Cow<'a, str>;

    #[inline]
    fn add(mut self, rhs: &'a str) -> Self::Output {
        self += rhs;
        self
    }
}

#[stable(feature = "cow_add", since = "1.14.0")]
impl<'a> Add<Cow<'a, str>> for Cow<'a, str> {
    type Output = Cow<'a, str>;

    #[inline]
    fn add(mut self, rhs: Cow<'a, str>) -> Self::Output {
        self += rhs;
        self
    }
}

#[stable(feature = "cow_add", since = "1.14.0")]
impl<'a> AddAssign<&'a str> for Cow<'a, str> {
    fn add_assign(&mut self, rhs: &'a str) {
        if self.is_empty() {
            *self = Cow::Borrowed(rhs)
        } else if rhs.is_empty() {
            return;
        } else {
            if let Cow::Borrowed(lhs) = *self {
                let mut s = String::with_capacity(lhs.len() + rhs.len());
                s.push_str(lhs);
                *self = Cow::Owned(s);
            }
            self.to_mut().push_str(rhs);
        }
    }
}

#[stable(feature = "cow_add", since = "1.14.0")]
impl<'a> AddAssign<Cow<'a, str>> for Cow<'a, str> {
    fn add_assign(&mut self, rhs: Cow<'a, str>) {
        if self.is_empty() {
            *self = rhs
        } else if rhs.is_empty() {
            return;
        } else {
            if let Cow::Borrowed(lhs) = *self {
                let mut s = String::with_capacity(lhs.len() + rhs.len());
                s.push_str(lhs);
                *self = Cow::Owned(s);
            }
            self.to_mut().push_str(&rhs);
        }
    }
}
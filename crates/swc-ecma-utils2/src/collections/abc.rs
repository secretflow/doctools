pub trait MappingBase {
  type Key;
  type Value;

  fn _len(&self) -> usize;
  fn _iter<'k>(&'k self) -> Box<dyn Iterator<Item = Self::Key> + 'k>;
  fn _get(&self, key: &Self::Key) -> Option<&Self::Value>;
}

pub trait Mapping: MappingBase {
  fn len(&self) -> usize {
    self._len()
  }

  fn is_empty(&self) -> bool {
    self._len() == 0
  }

  fn get_item<K: Into<Self::Key>>(&self, key: K) -> Option<&Self::Value> {
    self._get(&key.into())
  }

  fn contains<K: Into<Self::Key>>(&self, key: K) -> bool {
    self._get(&key.into()).is_some()
  }

  fn keys(&self) -> Box<dyn Iterator<Item = Self::Key> + '_> {
    self._iter()
  }

  fn values(&self) -> Box<dyn Iterator<Item = &Self::Value> + '_> {
    Box::new(self._iter().map(|k| self._get(&k).unwrap()))
  }

  fn items(&self) -> Box<dyn Iterator<Item = (Self::Key, &Self::Value)> + '_> {
    Box::new(self._iter().map(|k| {
      let v = self._get(&k).unwrap();
      (k, v)
    }))
  }

  fn get_item_at_path<K, P>(&self, path: P) -> Option<&Self::Value>
  where
    Self::Value: Mapping<Value = Self::Value>,
    K: Into<Self::Key> + Into<<Self::Value as MappingBase>::Key>,
    P: IntoIterator<Item = K>,
  {
    let mut iter = path.into_iter();
    let key = iter.next()?;
    let mut value = self._get(&key.into())?;
    for key in iter {
      value = value._get(&key.into())?;
    }
    Some(value)
  }
}

pub trait MutableMappingBase: MappingBase {
  fn _get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Value>;
  fn _set(&mut self, key: &Self::Key, value: Self::Value);
  fn _del(&mut self, key: &Self::Key) -> Option<Self::Value>;
  fn _pop(&mut self) -> Option<(Self::Key, Self::Value)>;
}

pub trait MutableMapping: MutableMappingBase + Mapping {
  fn get_item_mut<K: Into<Self::Key>>(&mut self, key: K) -> Option<&mut Self::Value> {
    self._get_mut(&key.into())
  }

  fn set_item<K: Into<Self::Key>>(&mut self, key: K, value: Self::Value) -> &mut Self {
    self._set(&key.into(), value);
    self
  }

  fn del_item<K: Into<Self::Key>>(&mut self, key: K) -> Option<Self::Value> {
    self._del(&key.into())
  }

  fn mut_item<K, F>(&mut self, key: K, f: F) -> &mut Self
  where
    K: Into<Self::Key>,
    F: FnOnce(&mut Self::Value),
  {
    if let Some(value) = self._get_mut(&key.into()) {
      f(value);
    }
    self
  }

  fn pop_item(&mut self) -> Option<(Self::Key, Self::Value)> {
    self._pop()
  }

  fn drain(&mut self) -> Box<dyn Iterator<Item = (Self::Key, Self::Value)> + '_> {
    Box::new(std::iter::from_fn(|| self._pop()))
  }

  fn update_from<K, I>(&mut self, other: I)
  where
    K: Into<Self::Key>,
    I: IntoIterator<Item = (K, Self::Value)>,
  {
    for (key, value) in other {
      self._set(&key.into(), value);
    }
  }

  fn set_default<K>(
    &mut self,
    key: K,
    value: Self::Value,
  ) -> Result<&mut Self::Value, CollectionError>
  where
    K: Into<Self::Key>,
  {
    let key = key.into();
    if self._get(&key).is_some() {
      Ok(self._get_mut(&key).unwrap())
    } else {
      self._set(&key, value);
      self._get_mut(&key).ok_or(CollectionError::CannotSet)
    }
  }

  fn get_item_mut_at_path<K, P>(&mut self, path: P) -> Option<&mut Self::Value>
  where
    Self::Value: MutableMapping<Value = Self::Value>,
    K: Into<Self::Key> + Into<<Self::Value as MappingBase>::Key>,
    P: IntoIterator<Item = K>,
  {
    let mut iter = path.into_iter();
    let key = iter.next()?;
    let mut value = self._get_mut(&key.into())?;
    for key in iter {
      value = value._get_mut(&key.into())?;
    }
    Some(value)
  }

  fn set_item_at_path<K, P>(
    &mut self,
    path: P,
    value: Self::Value,
  ) -> Result<&mut Self::Value, CollectionError>
  where
    Self::Value: MutableMapping<Value = Self::Value> + DefaultContainer,
    K: Into<Self::Key> + Into<<Self::Value as MappingBase>::Key>,
    P: IntoIterator<Item = K>,
  {
    let err = CollectionError::NoDefaultContainer(std::any::type_name::<Self::Value>());

    let mut path = path.into_iter().peekable();
    let key = path.next().ok_or(CollectionError::EmptyPath)?;

    if path.peek().is_none() {
      self.set_default(key, value)
    } else {
      let mut inner = self.set_default(key, Self::Value::create().ok_or(err)?)?;
      let mut key = path.next().unwrap();
      loop {
        if path.peek().is_none() {
          return inner.set_default(key, value);
        } else {
          inner = inner.set_default(key, Self::Value::create().ok_or(err)?)?;
          key = path.next().unwrap();
        }
      }
    }
  }

  fn from_iter<I, K>(iterable: I) -> Self
  where
    Self: DefaultContainer,
    I: IntoIterator<Item = (K, Self::Value)>,
    K: Into<Self::Key>,
  {
    let mut result = Self::guarantee();
    result.update_from(iterable);
    result
  }
}

pub trait SequenceBase {
  type Value;

  fn _len(&self) -> usize;
  fn _get(&self, idx: usize) -> Option<&Self::Value>;
}

pub trait Sequence: SequenceBase {
  fn len(&self) -> usize {
    self._len()
  }

  fn is_empty(&self) -> bool {
    self._len() == 0
  }

  fn get_item(&self, idx: usize) -> Option<&Self::Value> {
    self._get(idx)
  }

  fn iter<'k>(&'k self) -> Box<dyn DoubleEndedIterator<Item = &Self::Value> + 'k> {
    Box::new((0..self._len()).map(|i| self._get(i).unwrap()))
  }

  fn reversed(&self) -> Box<dyn Iterator<Item = &Self::Value> + '_> {
    Box::new(self.iter().rev())
  }

  fn contains(&self, value: &Self::Value) -> bool
  where
    Self::Value: PartialEq,
  {
    self.index_of(value).is_some()
  }

  fn index_of(&self, value: &Self::Value) -> Option<usize>
  where
    Self::Value: PartialEq,
  {
    self.iter().position(|v| v == value)
  }

  fn count(&self, value: &Self::Value) -> usize
  where
    Self::Value: PartialEq,
  {
    self.iter().filter(|v| *v == value).count()
  }
}

pub trait MutableSequenceBase: SequenceBase {
  fn _get_mut(&mut self, idx: usize) -> Option<&mut Self::Value>;
  fn _set(&mut self, idx: usize, value: Self::Value) -> Result<(), ()>;
  fn _del(&mut self, idx: usize) -> Option<Self::Value>;
  fn _ins(&mut self, idx: usize, value: Self::Value);
}

pub trait MutableSequence: MutableSequenceBase + Sequence {
  fn get_item_mut(&mut self, idx: usize) -> Option<&mut Self::Value> {
    self._get_mut(idx)
  }

  fn set_item(&mut self, idx: usize, value: Self::Value) -> Result<&mut Self, CollectionError> {
    self
      ._set(idx, value)
      .map_err(|_| CollectionError::IndexError)?;
    Ok(self)
  }

  fn del_item(&mut self, idx: usize) -> Option<Self::Value> {
    self._del(idx)
  }

  fn insert(&mut self, idx: usize, value: Self::Value) -> &mut Self {
    self._ins(idx, value);
    self
  }

  fn append(&mut self, value: Self::Value) -> &mut Self {
    self.insert(self._len(), value);
    self
  }

  fn drain(&mut self) -> Box<dyn Iterator<Item = Self::Value> + '_> {
    Box::new(std::iter::from_fn(|| self.pop(Some(0))))
  }

  fn extend<I>(&mut self, other: I)
  where
    I: IntoIterator<Item = Self::Value>,
  {
    for value in other {
      self.append(value);
    }
  }

  fn pop(&mut self, idx: Option<usize>) -> Option<Self::Value> {
    let idx = match idx {
      Some(idx) => idx,
      None => {
        let len = self._len();
        if len == 0 {
          return None;
        } else {
          len - 1
        }
      }
    };
    self._del(idx)
  }

  fn remove(&mut self, value: &Self::Value) -> Option<Self::Value>
  where
    Self::Value: PartialEq,
  {
    self.index_of(value).map(|idx| self._del(idx).unwrap())
  }

  fn reverse(&mut self) {
    let len = self._len();
    for i in 0..(len / 2) {
      let x: *mut Self::Value = self._get_mut(i).unwrap();
      let y: *mut Self::Value = self._get_mut(len - i - 1).unwrap();
      unsafe { std::ptr::swap(x, y) }
    }
  }

  fn from_iterable<I>(iterable: I) -> Self
  where
    Self: DefaultContainer,
    I: IntoIterator<Item = Self::Value>,
  {
    let mut result = Self::guarantee();
    result.extend(iterable);
    result
  }
}

impl<T> MappingBase for &T
where
  T: MappingBase,
{
  type Key = T::Key;
  type Value = T::Value;
  fn _len(&self) -> usize {
    T::_len(*self)
  }
  fn _iter<'k>(&'k self) -> Box<dyn Iterator<Item = Self::Key> + 'k> {
    T::_iter(*self)
  }
  fn _get(&self, key: &Self::Key) -> Option<&Self::Value> {
    T::_get(*self, key)
  }
}

impl<T> MappingBase for &mut T
where
  T: MappingBase,
{
  type Key = T::Key;
  type Value = T::Value;
  fn _len(&self) -> usize {
    T::_len(*self)
  }
  fn _iter<'k>(&'k self) -> Box<dyn Iterator<Item = Self::Key> + 'k> {
    T::_iter(*self)
  }
  fn _get(&self, key: &Self::Key) -> Option<&Self::Value> {
    T::_get(*self, key)
  }
}

impl<T> MappingBase for Option<T>
where
  T: MappingBase,
{
  type Key = T::Key;
  type Value = T::Value;

  fn _len(&self) -> usize {
    match self {
      Some(inner) => inner._len(),
      None => 0,
    }
  }

  fn _iter<'k>(&'k self) -> Box<dyn Iterator<Item = Self::Key> + 'k> {
    match self {
      Some(inner) => inner._iter(),
      None => Box::new(std::iter::empty()),
    }
  }

  fn _get(&self, key: &Self::Key) -> Option<&Self::Value> {
    match self {
      Some(inner) => inner._get(key),
      None => None,
    }
  }
}

impl<T> MutableMappingBase for &mut T
where
  T: MutableMappingBase,
{
  fn _get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Value> {
    T::_get_mut(*self, key)
  }
  fn _set(&mut self, key: &Self::Key, value: Self::Value) {
    T::_set(*self, key, value)
  }
  fn _del(&mut self, key: &Self::Key) -> Option<Self::Value> {
    T::_del(*self, key)
  }
  fn _pop(&mut self) -> Option<(Self::Key, Self::Value)> {
    T::_pop(*self)
  }
}

impl<T> MutableMappingBase for Option<T>
where
  T: MutableMappingBase + DefaultContainer,
{
  fn _get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Value> {
    match self {
      Some(inner) => inner._get_mut(key),
      None => None,
    }
  }

  fn _set(&mut self, key: &Self::Key, value: Self::Value) {
    if let Some(inner) = self {
      inner._set(key, value);
    } else {
      *self = T::create();
      if let Some(inner) = self {
        inner._set(key, value);
      }
    }
  }

  fn _del(&mut self, key: &Self::Key) -> Option<Self::Value> {
    match self {
      Some(inner) => inner._del(key),
      None => None,
    }
  }

  fn _pop(&mut self) -> Option<(Self::Key, Self::Value)> {
    match self {
      Some(inner) => inner._pop(),
      None => None,
    }
  }
}

impl<T> SequenceBase for &T
where
  T: SequenceBase,
{
  type Value = T::Value;
  fn _len(&self) -> usize {
    T::_len(*self)
  }
  fn _get(&self, idx: usize) -> Option<&Self::Value> {
    T::_get(*self, idx)
  }
}

impl<T> SequenceBase for &mut T
where
  T: SequenceBase,
{
  type Value = T::Value;
  fn _len(&self) -> usize {
    T::_len(*self)
  }
  fn _get(&self, idx: usize) -> Option<&Self::Value> {
    T::_get(*self, idx)
  }
}

impl<T> SequenceBase for Option<T>
where
  T: SequenceBase,
{
  type Value = T::Value;
  fn _len(&self) -> usize {
    match self {
      Some(inner) => inner._len(),
      None => 0,
    }
  }
  fn _get(&self, idx: usize) -> Option<&Self::Value> {
    match self {
      Some(inner) => inner._get(idx),
      None => None,
    }
  }
}

impl<T> MutableSequenceBase for &mut T
where
  T: MutableSequenceBase,
{
  fn _get_mut(&mut self, idx: usize) -> Option<&mut Self::Value> {
    T::_get_mut(*self, idx)
  }
  fn _set(&mut self, idx: usize, value: Self::Value) -> Result<(), ()> {
    T::_set(*self, idx, value)
  }
  fn _del(&mut self, idx: usize) -> Option<Self::Value> {
    T::_del(*self, idx)
  }
  fn _ins(&mut self, idx: usize, value: Self::Value) {
    T::_ins(*self, idx, value)
  }
}

impl<T> MutableSequenceBase for Option<T>
where
  T: MutableSequenceBase + DefaultContainer,
{
  fn _get_mut(&mut self, idx: usize) -> Option<&mut Self::Value> {
    match self {
      Some(inner) => inner._get_mut(idx),
      None => None,
    }
  }

  fn _set(&mut self, idx: usize, value: Self::Value) -> Result<(), ()> {
    if let Some(inner) = self {
      inner._set(idx, value)
    } else {
      *self = T::create();
      if let Some(inner) = self {
        inner._set(idx, value)?;
        Ok(())
      } else {
        Err(())
      }
    }
  }

  fn _del(&mut self, idx: usize) -> Option<Self::Value> {
    match self {
      Some(inner) => inner._del(idx),
      None => None,
    }
  }

  fn _ins(&mut self, idx: usize, value: Self::Value) {
    if let Some(inner) = self {
      inner._ins(idx, value);
    } else {
      *self = T::create();
      if let Some(inner) = self {
        inner._ins(idx, value);
      }
    }
  }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum CollectionError {
  #[error("index out of range")]
  IndexError,
  #[error("cannot set value")]
  CannotSet,
  #[error("failed to create default container for {0}")]
  NoDefaultContainer(&'static str),
  #[error("empty path")]
  EmptyPath,
}

pub trait DefaultContainer: Sized {
  fn create() -> Option<Self>;
  fn guarantee() -> Self;
}

impl<T> DefaultContainer for &mut T {
  fn create() -> Option<Self> {
    None
  }
  fn guarantee() -> Self {
    panic!("cannot guarantee mutable reference")
  }
}

impl<T> Mapping for T where T: MappingBase {}
impl<T> MutableMapping for T where T: MutableMappingBase {}

impl<T> Sequence for T where T: SequenceBase {}
impl<T> MutableSequence for T where T: MutableSequenceBase {}

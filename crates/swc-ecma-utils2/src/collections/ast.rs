use swc_core::{
  common::util::take::Take,
  ecma::ast::{
    ArrayLit, CallExpr, Callee, ComputedPropName, Expr, ExprOrSpread, Ident, KeyValueProp, Lit,
    ObjectLit, Prop, PropName, PropOrSpread, Str,
  },
};

use super::{
  abc::{
    DefaultContainer, MappingBase, MutableMappingBase, MutableSequence, MutableSequenceBase,
    SequenceBase,
  },
  Sequence,
};

impl MappingBase for ObjectLit {
  type Key = Lit;
  type Value = Expr;

  fn _len(&self) -> usize {
    self._iter().count()
  }

  fn _iter<'k>(&'k self) -> Box<dyn Iterator<Item = Self::Key> + 'k> {
    Box::new(self.props.iter().filter_map(prop_name_to_lit))
  }

  fn _get(&self, key: &Self::Key) -> Option<&Self::Value> {
    self.props.iter().find_map(|prop| {
      if test_object_key(prop, key) {
        Some(&*prop.as_prop()?.as_key_value()?.value)
      } else {
        None
      }
    })
  }
}

impl MutableMappingBase for ObjectLit {
  fn _get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Value> {
    self.props.iter_mut().find_map(|prop| {
      if test_object_key(prop, key) {
        Some(&mut *prop.as_mut_prop()?.as_mut_key_value()?.value)
      } else {
        None
      }
    })
  }

  fn _set(&mut self, key: &Self::Key, value: Self::Value) {
    if let Some(found) = self._get_mut(key) {
      *found = value;
    } else {
      self.props.push(PropOrSpread::Prop(
        Prop::KeyValue(KeyValueProp {
          key: lit_to_prop_name(key),
          value: Box::new(value),
        })
        .into(),
      ));
    }
  }

  fn _del(&mut self, key: &Self::Key) -> Option<Self::Value> {
    let found = self.props.iter_mut().enumerate().find_map(|(idx, prop)| {
      if test_object_key(prop, key) {
        Some((
          idx,
          (&mut *prop.as_mut_prop()?.as_mut_key_value()?.value).take(),
        ))
      } else {
        None
      }
    });

    if let Some((idx, found)) = found {
      self.props.swap_remove(idx);
      Some(found)
    } else {
      None
    }
  }

  fn _pop(&mut self) -> Option<(Self::Key, Self::Value)> {
    let key = self.props.iter().rev().find_map(prop_name_to_lit)?;
    let value = self._del(&key)?;
    Some((key, value))
  }
}

fn prop_name_to_lit(prop: &PropOrSpread) -> Option<Lit> {
  match prop {
    PropOrSpread::Prop(prop) => match &**prop {
      Prop::KeyValue(KeyValueProp { key, .. }) => match &*key {
        PropName::Str(string) => Some(string.clone().into()),
        PropName::Num(number) => Some(number.clone().into()),
        PropName::Ident(ident) => Some(Str::from(&*ident.sym).into()),
        PropName::Computed(ComputedPropName { expr, .. }) => match &**expr {
          Expr::Lit(lit) => Some(lit.clone()),
          Expr::Ident(ident) => Some(Str::from(&*ident.sym).into()),
          _ => None,
        },
        _ => None,
      },
      _ => None,
    },
    _ => None,
  }
}

fn lit_to_prop_name(lit: &Lit) -> PropName {
  match lit {
    Lit::Str(string) => PropName::Str(string.clone()),
    Lit::Num(number) => PropName::Num(number.clone()),
    _ => unreachable!(),
  }
}

fn get_object_key_for_cmp(key: &Lit) -> Option<String> {
  match key {
    Lit::Str(string) => Some(string.value.to_string()),
    Lit::Num(number) => Some(number.value.to_string()),
    _ => None,
  }
}

fn test_object_key(prop: &PropOrSpread, test: &Lit) -> bool {
  let Some(key) = prop_name_to_lit(prop) else {
    return false;
  };
  let Some(key) = get_object_key_for_cmp(&key) else {
    return false;
  };
  let Some(test) = get_object_key_for_cmp(test) else {
    return false;
  };
  key == test
}

impl DefaultContainer for ObjectLit {
  fn create() -> Option<Self> {
    Some(Self::guarantee())
  }

  fn guarantee() -> Self {
    Self {
      span: Default::default(),
      props: vec![],
    }
  }
}

impl SequenceBase for ArrayLit {
  type Value = Expr;

  fn _len(&self) -> usize {
    self.elems.len()
  }

  fn _get(&self, idx: usize) -> Option<&Self::Value> {
    Some(self.elems.get(idx)?.as_ref()?.expr.as_ref())
  }
}

impl MutableSequenceBase for ArrayLit {
  fn _get_mut(&mut self, idx: usize) -> Option<&mut Self::Value> {
    Some(self.elems.get_mut(idx)?.as_mut()?.expr.as_mut())
  }

  fn _set(&mut self, idx: usize, value: Self::Value) -> Result<(), ()> {
    if idx >= self.elems.len() {
      Err(())
    } else {
      self.elems[idx] = Some(ExprOrSpread {
        spread: None,
        expr: Box::new(value),
      });
      Ok(())
    }
  }

  fn _del(&mut self, idx: usize) -> Option<Self::Value> {
    if idx >= self.elems.len() {
      None
    } else {
      let found = self.elems.remove(idx)?;
      Some(*found.expr)
    }
  }

  fn _ins(&mut self, idx: usize, value: Self::Value) {
    let idx = idx.min(self.elems.len());
    self.elems.insert(
      idx,
      Some(ExprOrSpread {
        spread: None,
        expr: Box::new(value),
      }),
    );
  }
}

impl DefaultContainer for ArrayLit {
  fn create() -> Option<Self> {
    Some(Self::guarantee())
  }

  fn guarantee() -> Self {
    Self {
      span: Default::default(),
      elems: vec![],
    }
  }
}

impl MappingBase for CallExpr {
  type Key = usize;
  type Value = Expr;

  fn _len(&self) -> usize {
    self.args.len() + 1
  }

  fn _iter<'k>(&'k self) -> Box<dyn Iterator<Item = Self::Key> + 'k> {
    Box::new(0..self._len())
  }

  fn _get(&self, key: &Self::Key) -> Option<&Self::Value> {
    if *key == 0usize {
      match &self.callee {
        Callee::Expr(ref expr) => Some(expr.as_ref()),
        _ => None,
      }
    } else {
      Some(self.args.get(key - 1)?.expr.as_ref())
    }
  }
}

impl MutableMappingBase for CallExpr {
  fn _get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Value> {
    if *key == 0usize {
      match &mut self.callee {
        Callee::Expr(ref mut expr) => Some(expr.as_mut()),
        _ => None,
      }
    } else {
      Some(self.args.get_mut(key - 1)?.expr.as_mut())
    }
  }

  fn _set(&mut self, key: &Self::Key, value: Self::Value) {
    if *key == 0 {
      self.callee = Callee::Expr(Box::new(value));
    } else {
      let idx = key - 1;
      if idx >= self.args.len() {
        self.args.resize_with(idx + 1, || ExprOrSpread {
          spread: None,
          expr: undefined().into(),
        });
      }
      self.args[idx] = ExprOrSpread {
        spread: None,
        expr: Box::new(value),
      };
    }
  }

  fn _del(&mut self, key: &Self::Key) -> Option<Self::Value> {
    if *key == 0 {
      let found = self.callee.take();
      let expr = *found.expr()?;
      Some(expr)
    } else if *key - 1 >= self.args.len() {
      None
    } else {
      let found = self.args.remove(*key - 1);
      let expr = *found.expr;
      Some(expr)
    }
  }

  fn _pop(&mut self) -> Option<(Self::Key, Self::Value)> {
    let idx = self._len().saturating_sub(1);
    Some((idx, self._del(&idx)?))
  }
}

fn undefined() -> Expr {
  Ident::new("undefined".into(), Default::default()).into()
}

impl DefaultContainer for CallExpr {
  fn create() -> Option<Self> {
    Some(Self::guarantee())
  }

  fn guarantee() -> Self {
    Self {
      span: Default::default(),
      callee: Callee::Expr(Box::new(Expr::dummy())),
      args: vec![],
      type_args: None,
    }
  }
}

impl MappingBase for Expr {
  type Key = Lit;
  type Value = Expr;

  fn _len(&self) -> usize {
    match self {
      Expr::Object(ref obj) => obj._len(),
      Expr::Array(ref arr) => arr.len(),
      Expr::Call(ref call) => call._len(),
      _ => 0,
    }
  }

  fn _iter<'k>(&'k self) -> Box<dyn Iterator<Item = Self::Key> + 'k> {
    match self {
      Expr::Object(ref obj) => obj._iter(),
      Expr::Array(ref arr) => Box::new((0..arr.len()).map(|idx| idx.to_string().into())),
      Expr::Call(ref call) => Box::new((0..call._len()).map(|idx| idx.to_string().into())),
      _ => Box::new(std::iter::empty()),
    }
  }

  fn _get(&self, key: &Self::Key) -> Option<&Self::Value> {
    match self {
      Expr::Object(ref obj) => obj._get(key),
      Expr::Array(ref arr) => lit_as_usize(key).and_then(|idx| arr.get_item(idx)),
      Expr::Call(ref call) => lit_as_usize(key).and_then(|idx| call._get(&idx)),
      _ => None,
    }
  }
}

impl MutableMappingBase for Expr {
  fn _get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Value> {
    match self {
      Expr::Object(ref mut obj) => obj._get_mut(key),
      Expr::Array(ref mut arr) => lit_as_usize(key).and_then(|idx| arr._get_mut(idx)),
      Expr::Call(ref mut call) => lit_as_usize(key).and_then(|idx| call._get_mut(&idx)),
      _ => None,
    }
  }

  fn _set(&mut self, key: &Self::Key, value: Self::Value) {
    match self {
      Expr::Object(ref mut obj) => obj._set(key, value),
      Expr::Array(ref mut arr) => {
        let idx = lit_as_usize(key).unwrap_or(arr.len());
        arr._set(idx, value).unwrap();
      }
      Expr::Call(ref mut call) => {
        let idx = lit_as_usize(key).unwrap_or(call._len());
        call._set(&idx, value);
      }
      _ => {}
    }
  }

  fn _del(&mut self, key: &Self::Key) -> Option<Self::Value> {
    match self {
      Expr::Object(ref mut obj) => obj._del(key),
      Expr::Array(ref mut arr) => {
        let idx = lit_as_usize(key).unwrap_or(arr.len());
        Some(arr._del(idx)?)
      }
      Expr::Call(ref mut call) => {
        let idx = lit_as_usize(key).unwrap_or(call._len());
        Some(call._del(&idx)?)
      }
      _ => None,
    }
  }

  fn _pop(&mut self) -> Option<(Self::Key, Self::Value)> {
    match self {
      Expr::Object(ref mut obj) => obj._pop(),
      Expr::Array(ref mut arr) => {
        let idx = arr.len().saturating_sub(1);
        Some((idx.to_string().into(), arr.pop(None)?))
      }
      Expr::Call(ref mut call) => call
        ._pop()
        .and_then(|(idx, expr)| Some((idx.to_string().into(), expr))),
      _ => None,
    }
  }
}

fn lit_as_usize(lit: &Lit) -> Option<usize> {
  match lit {
    Lit::Num(num) => {
      if num.value.fract() != 0.0 {
        None
      } else if num.value.is_sign_negative() {
        None
      } else if num.value > usize::MAX as f64 {
        None
      } else {
        Some(num.value as usize)
      }
    }
    Lit::Str(str) => str.value.parse().ok(),
    _ => None,
  }
}

impl DefaultContainer for Expr {
  fn create() -> Option<Self> {
    Some(Self::guarantee())
  }

  fn guarantee() -> Self {
    Expr::Object(ObjectLit::guarantee())
  }
}

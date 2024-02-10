use swc_core::{
  common::util::take::Take,
  ecma::{
    ast::{ArrayLit, Expr, Ident, Lit, Number, ObjectLit},
    utils::drop_span,
  },
};

use swc_ecma_testing2::{
  assert_eq_codegen, parse_expr_unchecked, pretty_assertions::assert_eq, print_one_unchecked,
};
use swc_ecma_utils2::{
  collections::{Mapping as _, MutableMapping as _, MutableSequence, Sequence as _},
  json_expr,
};

fn create_object() -> Expr {
  json_expr!({
    "null": null,
    "bool": true,
    "string": "string",
    "number": 42,
    "array": [null, true, "string", 42],
    "object": {
      "lorem": "ipsum",
      "dolor": "sit amet",
    },
  })
}

#[test]
fn test_mapping_len() {
  let obj = create_object();
  assert_eq!(obj.len(), 6);
}

#[test]
fn test_mapping_get_item() {
  let obj = create_object();
  assert_eq_codegen!(obj.get_item("null").unwrap(), &json_expr!(null));
}

#[test]
fn test_mapping_get_item_chain() {
  let obj = create_object();
  assert_eq_codegen!(
    obj.get_item("object").get_item("lorem").unwrap(),
    &json_expr!("ipsum")
  );
}

#[test]
fn test_mapping_get_item_by_path() {
  let obj = create_object();
  assert_eq_codegen!(
    obj.get_item_at_path(["object", "lorem"]).unwrap(),
    &json_expr!("ipsum")
  );
}

#[test]
fn test_mapping_get_item_chain_none() {
  let obj = create_object();
  assert_eq!(obj.get_item("abc").get_item("def"), None);
}

#[test]
fn test_mapping_iter_items() {
  let obj = create_object();

  let mut iter = obj.items();

  assert_eq!(iter.next(), Some(("null".into(), &json_expr!(null))));

  assert_eq!(
    iter.last(),
    Some((
      "object".into(),
      &json_expr!({
        "lorem": "ipsum",
        "dolor": "sit amet",
      })
    ))
  );
}

#[test]
fn test_mapping_set_item() {
  let mut obj = create_object();

  obj.set_item("abc", json_expr!(123));

  assert_eq_codegen!(obj.get_item("abc").unwrap(), &json_expr!(123));
}

#[test]
fn test_mapping_del_item() {
  let mut obj = create_object();
  assert_eq_codegen!(&obj.del_item("string").unwrap(), &json_expr!("string"));
  assert_eq!(obj.get_item("string"), None);
}

#[test]
fn test_mapping_pop_item() {
  let mut obj = create_object();
  assert_eq!(
    Some((
      "object".into(),
      json_expr!({
        "lorem": "ipsum",
        "dolor": "sit amet",
      })
    )),
    obj.pop_item(),
  );
}

#[test]
fn test_mapping_mut_item() {
  let mut obj = create_object();

  obj.mut_item("number", |num| match num {
    Expr::Lit(Lit::Num(Number { ref mut value, .. })) => {
      *value += 1.0;
    }
    _ => unreachable!(),
  });

  assert_eq_codegen!(obj.get_item("number").unwrap(), &json_expr!(43));
}

#[test]
fn test_mapping_drain() {
  let mut obj = create_object();
  {
    let mut iter = obj.drain();
    assert_eq_codegen!(&iter.next().unwrap().0.into(), &"object".into());
  }
  assert_eq!(obj.len(), 5);
}

#[test]
fn test_mapping_update() {
  let mut this = create_object();

  let other = json_expr!({
    "null": "null",
    "bool": 1.5,
    "string": false,
    "number": [],
    "array": {},
    "object": null,
  });

  this.update_from(&mut other.clone().drain());

  assert_eq_codegen!(&this, &other);
}

#[test]
fn test_mapping_set_default() {
  let mut obj = create_object();

  obj.set_default("abc", json_expr!(123)).unwrap();

  assert_eq_codegen!(obj.get_item("abc").unwrap(), &json_expr!(123));

  obj.set_default("null", json_expr!(123)).unwrap();

  assert_eq_codegen!(obj.get_item("null").unwrap(), &json_expr!(null));
}

#[test]
#[should_panic(expected = "cannot set value")]
fn test_mapping_set_default_err() {
  fn test() -> anyhow::Result<()> {
    let mut obj = create_object();

    obj
      .set_default("null", ObjectLit::dummy().into())?
      .set_default("data", "value".into())?;

    Ok(())
  }

  test().unwrap();
}

#[test]
fn test_mapping_set_path() {
  let mut obj = create_object();

  obj
    .set_item_at_path(["consectetur", "adipiscing"], json_expr!("elit"))
    .unwrap();

  assert_eq_codegen!(
    obj.get_item("consectetur").get_item("adipiscing").unwrap(),
    &json_expr!("elit")
  );
}

#[test]
#[should_panic(expected = "CannotSet")]
fn test_mapping_set_path_cannot_set() {
  let mut obj = create_object();
  obj
    .set_item_at_path(["number", "value"], json_expr!("elit"))
    .unwrap();
}

fn create_array() -> ArrayLit {
  json_expr!([
    null,
    true,
    "string",
    42,
    [[1, 2, 4], [8, 16, 32]],
    {
      "lorem": "ipsum",
      "dolor": "sit amet",
    },
  ])
  .array()
  .unwrap()
}

#[test]
fn test_sequence_len() {
  let arr = create_array();
  assert_eq!(arr.len(), 6);
}

#[test]
fn test_sequence_get_item() {
  let arr = create_array();
  assert_eq_codegen!(arr.get_item(0).unwrap(), &json_expr!(null));
}

#[test]
fn test_sequence_get_item_chain() {
  let arr = create_array();
  assert_eq_codegen!(
    arr.get_item(4).get_item(1).get_item(2).unwrap(),
    &json_expr!(32)
  );
}

#[test]
fn test_sequence_get_item_chain_none() {
  let arr = create_array();
  assert_eq!(arr.get_item(4).get_item(2).get_item(3), None);
}

#[test]
fn test_sequence_reversed() {
  let arr = create_array();
  let mut rev = arr.reversed();
  assert_eq_codegen!(
    rev.next().unwrap(),
    &json_expr!({
      "lorem": "ipsum",
      "dolor": "sit amet",
    })
  );
}

#[test]
fn test_sequence_index_of() {
  let arr = create_array();
  assert_eq!(arr.index_of(&json_expr!(42)), Some(3));
}

#[test]
fn test_sequence_set_item() {
  let mut arr = create_array();

  arr.set_item(0, json_expr!(123)).unwrap();

  assert_eq_codegen!(arr.get_item(0).unwrap(), &json_expr!(123));
}

#[test]
#[should_panic(expected = "IndexError")]
fn test_sequence_set_item_index_error() {
  let mut arr = create_array();

  arr.set_item(6, json_expr!(123)).unwrap();

  assert_eq_codegen!(arr.get_item(6).unwrap(), &json_expr!(123));
}

#[test]
fn test_sequence_del_item() {
  let mut arr = create_array();
  assert_eq_codegen!(&arr.del_item(2).unwrap(), &json_expr!("string"));
  assert_eq_codegen!(arr.get_item(2).unwrap(), &json_expr!(42));
  assert_eq!(arr.len(), 5);
}

#[test]
fn test_sequence_del_item_out_of_bounds() {
  let mut arr = create_array();
  assert_eq!(arr.del_item(12), None);
  assert_eq!(arr.len(), 6);
}

#[test]
fn test_sequence_insert() {
  let mut arr = create_array();
  arr.insert(2, json_expr!(123));
  assert_eq_codegen!(arr.get_item(2).unwrap(), &json_expr!(123));
  assert_eq!(arr.len(), 7);
}

#[test]
fn test_sequence_pop_empty() {
  let mut arr = json_expr!([]).array();
  assert_eq!(arr.pop(Some(0)), None);
}

#[test]
fn test_sequence_reverse_in_place() {
  let mut arr = create_array();
  let mut arr = arr.del_item(4).unwrap().array().unwrap();

  arr.reverse();

  arr
    .get_item_mut(1)
    .unwrap()
    .as_mut_array()
    .unwrap()
    .reverse();

  assert_eq_codegen!(&Expr::from(arr), &json_expr!([[8, 16, 32], [4, 2, 1]]));
}

fn create_expr(src: &str) -> Expr {
  drop_span(parse_expr_unchecked(src))
}

#[test]
fn test_call_callee() {
  let call = create_expr("add(1, 2, 3, 4, 5)").call();
  assert_eq_codegen!(call.get_item(0usize).unwrap(), &Ident::from("add").into());
}

#[test]
fn test_call_set_arg() {
  let mut call = create_expr("add(1, 2, 3, 4, 5)").call().unwrap();

  call.set_item(1usize, (-1f64).into());

  call.set_item(7usize, (7f64).into());

  assert_eq!(
    print_one_unchecked(&call),
    "add(-1, 2, 3, 4, 5, undefined, 7)"
  )
}

#[test]
fn test_call_set_callee() {
  let mut call = create_expr("add(1, 2, 3, 4, 5)").call().unwrap();

  call.set_item(0usize, json_expr!([]));

  assert_eq!(print_one_unchecked(&call), "[](1, 2, 3, 4, 5)")
}

#[test]
fn test_call_del_args() {
  let mut call = create_expr("add(1, 2, 3, 4, 5)").call().unwrap();

  call.del_item(3usize);

  assert_eq!(print_one_unchecked(&call), "add(1, 2, 4, 5)")
}

#[test]
fn test_call_del_callee() {
  let mut call = create_expr("add(1, 2, 3, 4, 5)").call().unwrap();

  call.del_item(0usize);

  assert_eq!(print_one_unchecked(&call), "super(1, 2, 3, 4, 5)")
}

#[test]
fn test_call_set_default() {
  let mut call = create_expr("add(1, 2, 3, 4, 5)").call().unwrap();

  call.set_default(3usize, call.clone().into()).unwrap();
  call.set_default(6usize, call.clone().into()).unwrap();

  assert_eq!(
    print_one_unchecked(&call),
    "add(1, 2, 3, 4, 5, add(1, 2, 3, 4, 5))"
  )
}

#[test]
fn test_call_drain() {
  let mut call = create_expr("add(1, 2, 3, 4, 5)").call().unwrap();

  for _ in call.drain() {}

  assert_eq!(print_one_unchecked(&call), "super()")
}

#[test]
fn test_array_with_holes_iter() {
  let mut arr = create_expr("[,,,3]").array().unwrap();
  let items = arr.drain().collect::<Vec<_>>();
  assert_eq!(items.len(), 0);
}

fn create_arbitrary() -> Expr {
  create_expr(
    r#"
    {
      constructor() {
        throw new Error("Cannot construct object");
      },

      lorem: "ipsum",

      get number() {
        return 42;
      },

      set number(value) {
        console.log(value);
      },

      NaN: "nan",

      42: 43,

      [Infinity]: "infinity",

      [Symbol.iterator]: function*() {
        yield 1;
        yield 2;
        yield 3;
      },

      addEventListener,

      ...window,
    }
    "#,
  )
}

#[test]
fn test_arbitrary_object_len() {
  let obj = create_arbitrary().object().unwrap();
  assert_eq!(obj.len(), 4);
}

#[test]
fn test_arbitrary_object_get_item() {
  let obj = create_arbitrary().object().unwrap();
  assert_eq!(
    print_one_unchecked(&obj.get_item("Infinity").unwrap()),
    r#""infinity""#
  );
}

#[test]
fn test_arbitrary_object_set_item() {
  let mut obj = create_arbitrary().object().unwrap();

  obj.set_item("Infinity", create_expr("NaN"));

  assert_eq!(
    print_one_unchecked(&obj.get_item("Infinity").unwrap()),
    "NaN"
  );

  assert!(print_one_unchecked(&obj).contains("[Infinity]: NaN"));
}

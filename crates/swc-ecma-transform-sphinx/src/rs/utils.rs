#[macro_export]
macro_rules! move_basic_attributes {
  ($from:expr, $into:expr) => {{
    $into.ids.extend($from.ids.drain(..));
    $into.classes.extend($from.classes.drain(..));
    $into.names.extend($from.names.drain(..));
    $into.dupnames.extend($from.dupnames.drain(..));
  }};
}

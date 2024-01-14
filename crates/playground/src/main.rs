use serde_json::from_str;
use swc_core::ecma::ast::FnDecl;
use swc_utils::testing::print_one;

fn main() {
    let ast = from_str::<FnDecl>(include_str!("./ensure.json")).unwrap();
    println!("{}", print_one(&ast, Default::default()));
}

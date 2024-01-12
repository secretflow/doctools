use swc_core::ecma::ast::Program;
use swc_core::ecma::visit::VisitMutWith;
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

use esm_serve::{externalize_modules, ExternalPackages};

#[plugin_transform]
pub fn esm_serve(mut program: Program, data: TransformPluginProgramMetadata) -> Program {
    let config = serde_json::from_str::<ExternalPackages>(
        &data
            .get_transform_plugin_config()
            .expect("[esm-serve] failed to get plugin config"),
    )
    .expect("[esm-serve] invalid config");
    program.visit_mut_with(&mut externalize_modules(&config));
    program
}

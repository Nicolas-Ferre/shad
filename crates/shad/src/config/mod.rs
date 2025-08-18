pub(crate) mod transpilation;
pub(crate) mod validation;

use serde::Deserialize;
use serde_valid::Validate;
use std::collections::HashMap;
use std::fs;
use std::fs::DirEntry;
use std::ops::RangeInclusive;
use std::rc::Rc;

pub(crate) fn load_config() -> Config {
    let mut config: Config = serde_yml::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/config.yaml"
    )))
    .expect("internal error: config should be valid");
    let additional_kinds = fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/resources"))
        .expect("internal error: config folder should be valid")
        .filter_map(Result::ok)
        .filter(is_kind_config_file)
        .flat_map(load_kind_config);
    config.kinds.extend(additional_kinds);
    config
        .validate()
        .expect("internal error: config should be valid");
    config
}

fn is_kind_config_file(entry: &DirEntry) -> bool {
    entry.path().extension().is_some_and(|ext| ext == "yaml")
        && entry
            .file_name()
            .as_os_str()
            .to_str()
            .is_some_and(|filename| filename.starts_with("kinds_"))
}

fn load_kind_config(entry: DirEntry) -> HashMap<String, Rc<KindConfig>> {
    serde_yml::from_str::<KindsConfig>(
        fs::read_to_string(entry.path())
            .expect("internal error: kind config file should exist")
            .as_str(),
    )
    .expect("internal error: kind config should be valid")
    .kinds
}

fn default_repeat() -> u32 {
    1
}

fn validate_kinds(
    kinds: &HashMap<String, Rc<KindConfig>>,
) -> Result<(), serde_valid::validation::Error> {
    for kind in kinds.values() {
        kind.validate()
            .map_err(|err| serde_valid::validation::Error::Custom(err.to_string()))?;
    }
    Ok(())
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub(crate) struct Config {
    #[validate(min_length = 1)]
    pub(crate) root_kind: String,
    #[validate(min_length = 1)]
    pub(crate) root_expected_first_tokens: Vec<String>,
    #[validate(min_length = 1)]
    pub(crate) comment_prefix: String,
    #[validate(min_length = 1)]
    pub(crate) import_index_key: String,
    pub(crate) type_transpilation: HashMap<String, String>,
    #[validate(custom(validate_kinds))]
    pub(crate) kinds: HashMap<String, Rc<KindConfig>>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub(crate) struct KindsConfig {
    #[validate(custom(validate_kinds))]
    pub(crate) kinds: HashMap<String, Rc<KindConfig>>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub(crate) struct KindConfig {
    pub(crate) string: Option<String>,
    #[serde(default)]
    #[validate]
    pub(crate) pattern_parts: Vec<PatternPartConfig>,
    #[serde(default)]
    #[validate(min_length = 1)]
    pub(crate) choice: Vec<String>,
    #[serde(default)]
    #[validate(min_length = 1)]
    pub(crate) sequence: Vec<String>,
    pub(crate) sequence_error_after: Option<String>,
    #[serde(default = "default_repeat")]
    pub(crate) min_repeat: u32,
    #[serde(default = "default_repeat")]
    pub(crate) max_repeat: u32,
    pub(crate) display_name: Option<String>,
    #[validate]
    pub(crate) import_path: Option<ImportPathConfig>,
    #[validate]
    pub(crate) buffer: Option<BufferConfig>,
    #[validate]
    pub(crate) index_key: Option<IndexKeyConfig>,
    #[validate]
    pub(crate) index_key_source: Option<IndexKeySourceConfig>,
    #[serde(default)]
    #[validate]
    pub(crate) validation: Vec<ValidationConfig>,
    #[serde(default)]
    #[validate]
    pub(crate) type_resolution: TypeResolutionConfig,
    #[serde(default)]
    pub(crate) transpilation: String,
    #[validate]
    pub(crate) init_shader: Option<ShaderConfig>,
    #[validate]
    pub(crate) run_shader: Option<ShaderConfig>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub(crate) struct PatternPartConfig {
    pub(crate) char_ranges: Vec<RangeInclusive<char>>,
    pub(crate) min_length: usize,
    pub(crate) max_length: usize,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImportPathConfig {
    #[validate(min_length = 1)]
    pub(crate) parent: String,
    #[validate(min_length = 1)]
    pub(crate) segment: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub(crate) struct BufferConfig {
    #[validate(min_length = 1)]
    pub(crate) ident: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub(crate) struct IndexKeyConfig {
    #[validate(min_length = 1)]
    pub(crate) child: Option<String>,
    #[validate(min_length = 1)]
    pub(crate) string: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub(crate) struct IndexKeySourceConfig {
    #[validate(min_length = 1)]
    pub(crate) parents: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub(crate) struct ValidationConfig {
    #[validate(min_length = 1)]
    pub(crate) name: String,
    pub(crate) params: HashMap<String, String>,
}

#[derive(Default, Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub(crate) struct TypeResolutionConfig {
    pub(crate) name: Option<String>,
    pub(crate) source_child: Option<String>,
}

#[derive(Default, Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub(crate) struct ShaderConfig {
    pub(crate) transpilation: String,
}

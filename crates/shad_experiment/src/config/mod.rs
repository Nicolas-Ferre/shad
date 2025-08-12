pub(crate) mod transpilation;
pub(crate) mod validation;

use serde::Deserialize;
use serde_valid::Validate;
use std::collections::HashMap;
use std::error::Error;
use std::ops::RangeInclusive;
use std::rc::Rc;

pub(crate) fn load_config() -> Result<Config, Box<dyn Error>> {
    let config: Config = serde_yml::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/config.yaml"
    )))?;
    config.validate()?;
    Ok(config)
}

fn default_repeat() -> u32 {
    1
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
    #[validate(custom(validate_kinds))]
    pub(crate) kinds: HashMap<String, Rc<KindConfig>>,
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
#[serde(deny_unknown_fields)]
pub struct KindConfig {
    pub string: Option<String>,
    #[serde(default)]
    #[validate]
    pub pattern_parts: Vec<PatternPartConfig>,
    #[serde(default)]
    #[validate(min_length = 1)]
    pub choice: Vec<String>,
    #[serde(default)]
    #[validate(min_length = 1)]
    pub sequence: Vec<String>,
    pub sequence_error_after: Option<String>,
    #[serde(default = "default_repeat")]
    pub min_repeat: u32,
    #[serde(default = "default_repeat")]
    pub max_repeat: u32,
    pub display_name: Option<String>,
    #[validate]
    pub import_path: Option<ImportPathConfig>,
    #[validate]
    pub buffer: Option<BufferConfig>,
    #[validate]
    pub index_key: Option<IndexKeyConfig>,
    #[validate]
    pub index_key_source: Option<IndexKeySourceConfig>,
    #[serde(default)]
    #[validate]
    pub validation: Vec<ValidationConfig>,
    #[serde(default)]
    #[validate]
    pub type_resolution: TypeResolutionConfig,
    #[serde(default)]
    pub transpilation: String,
    #[validate]
    pub init_shader: Option<ShaderConfig>,
    #[validate]
    pub run_shader: Option<ShaderConfig>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct PatternPartConfig {
    pub char_ranges: Vec<RangeInclusive<char>>,
    pub min_length: usize,
    pub max_length: usize,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ImportPathConfig {
    #[validate(min_length = 1)]
    pub parent: String,
    #[validate(min_length = 1)]
    pub segment: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct BufferConfig {
    #[validate(min_length = 1)]
    pub ident: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct IndexKeyConfig {
    #[validate(min_length = 1)]
    pub child: Option<String>,
    #[validate(min_length = 1)]
    pub string: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct IndexKeySourceConfig {
    #[validate(min_length = 1)]
    pub parents: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ValidationConfig {
    #[validate(min_length = 1)]
    pub name: String,
    pub params: HashMap<String, String>,
}

#[derive(Default, Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct TypeResolutionConfig {
    pub name: Option<String>,
    pub source_child: Option<String>,
}

#[derive(Default, Debug, Clone, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ShaderConfig {
    pub transpilation: String,
}

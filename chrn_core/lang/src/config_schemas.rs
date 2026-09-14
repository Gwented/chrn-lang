use std::fmt::Display;

use chrn_utils::{id_types::InternedId, intern};

use crate::types::boundaries::TypeBoundaryFlags;

/// Represent a configurataion description that must be followed,
#[derive(Debug)]
pub struct ConfigSchema {
    pub kind: ConfigSchemaKind,
    pub opt_schema: &'static [OptionSchema],
}

impl ConfigSchema {
    pub const fn new(kind: ConfigSchemaKind, opt_schema: &'static [OptionSchema]) -> ConfigSchema {
        ConfigSchema { kind, opt_schema }
    }
    //TODO: Change to O(1) with interned id -> idx mapppppppppppppppppppppppppppppping
    //Huh
    // Manual hashmap.

    /// Attempts to find option from the given identifier and returns it's `OptionSchema`
    pub const fn get_opt(&self, target_name_id: InternedId) -> Option<&OptionSchema> {
        let mut i = 0;
        while i < self.opt_schema.len() {
            let opt = &self.opt_schema[i];
            if opt.name_id.id == target_name_id.id {
                return Some(opt);
            }
            i += 1;
        }
        None
    }

    /// Attempts to find option from the given identifier
    ///
    /// Returns `true` if present, `false` if not
    pub const fn has_opt(&self, target_name_id: InternedId) -> bool {
        self.get_opt(target_name_id).is_some()
    }
}

/// Represents a configs options, that are preloaded by the compiler as schemas to follow
#[derive(Debug)]
pub struct OptionSchema {
    pub name_id: InternedId,
    pub boundaries: Option<OptionSchemaConstraint>,
}

impl OptionSchema {
    pub const fn new(
        name_id: InternedId,
        boundaries: Option<OptionSchemaConstraint>,
    ) -> OptionSchema {
        OptionSchema {
            name_id,
            boundaries,
        }
    }
}

#[derive(Debug)]
pub enum ConfigSchemaKind {
    Struct,
    Enum,
    Member,
}

impl Display for ConfigSchemaKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            ConfigSchemaKind::Struct => "struct",
            ConfigSchemaKind::Enum => "enum",
            ConfigSchemaKind::Member => "member",
        };
        write!(f, "{out}")
    }
}

/// All known preset schemas and the options associated with them
pub static PRESET_CONFIG_SCHEMAS: [ConfigSchema; 3] = [
    // ONLY 3 VALID SCHEMAS RIGHT NOW
    ConfigSchema::new(ConfigSchemaKind::Struct, &[OPT_CASES, OPT_IDENTS]),
    ConfigSchema::new(ConfigSchemaKind::Enum, &[OPT_CASES, OPT_IDENTS]),
    ConfigSchema::new(
        ConfigSchemaKind::Member,
        &[OPT_CASES, OPT_IDENTS, OPT_DEFAULT_VAL],
    ),
];

/// Types of option restrictions
#[derive(Debug, Clone)]
pub enum OptionSchemaConstraint {
    Boundaries(TypeBoundaryFlags),
    SameTypeAsConfig,
    // None,
}

const OPT_DEFAULT_VAL: OptionSchema = OptionSchema::new(
    InternedId::new(intern::INTERNED_DEFAULT_VAL),
    Some(OptionSchemaConstraint::SameTypeAsConfig),
);
const OPT_IDENTS: OptionSchema = OptionSchema::new(
    InternedId::new(intern::INTERNED_IDENTS),
    Some(OptionSchemaConstraint::Boundaries(TypeBoundaryFlags::STR)),
);
const OPT_CASES: OptionSchema = OptionSchema::new(
    InternedId::new(intern::INTERNED_CASES),
    Some(OptionSchemaConstraint::Boundaries(TypeBoundaryFlags::STR)),
);

pub const fn get_cfg_schema(kind: ConfigSchemaKind) -> &'static ConfigSchema {
    match kind {
        ConfigSchemaKind::Struct => &PRESET_CONFIG_SCHEMAS[0],
        ConfigSchemaKind::Enum => &PRESET_CONFIG_SCHEMAS[1],
        ConfigSchemaKind::Member => &PRESET_CONFIG_SCHEMAS[2],
    }
}

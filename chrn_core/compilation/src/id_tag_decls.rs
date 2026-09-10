// Not entirely sure what to do with this module
use chrn_utils::{id_types::id_tags::ArenaIndexTag, tag_decl};
tag_decl!(
    TypeDefTag,
    StructTag,
    FuncTag,
    EnumTag,
    AliasTag,
    VarTag,
    ConfigRootTag,
    DirectiveTag,
    FieldTag,
    VariantTag,
    ExternTypeTag,
    ConfigMemberTag,
    OptionAssignmentRootTag,
    OptionAssignmentMemberTag,
    MultiTypeAssignmentTag,
);

use chrn_utils::{
    id_types::InternedIdUTF8, source_utils::source_span::SourceSpan,
    utils::containers::SpannedContainer,
};
use lang::types::boundaries::TypeBoundaryFlags;

/// General container for `OptionAssignment(Root/Member)`
pub(super) struct OptionContext {
    pub cfg_ty_name_id: Option<InternedIdUTF8>,
    pub cfg_name_span: SourceSpan,
    pub user_boundaries_opt: Option<TypeBoundaryFlags>,
    pub sp_opt_name_id: SpannedContainer<InternedIdUTF8>,
}

impl OptionContext {
    pub(super) fn new(
        cfg_ty_name_id: Option<InternedIdUTF8>,
        cfg_name_span: SourceSpan,
        user_boundaries_opt: Option<TypeBoundaryFlags>,
        sp_opt_name_id: SpannedContainer<InternedIdUTF8>,
    ) -> Self {
        Self {
            cfg_ty_name_id,
            cfg_name_span,
            user_boundaries_opt,
            sp_opt_name_id,
        }
    }
}

use bitflags::bitflags;

//TEST: May or may not have stages depend on parts of other stages so these are bitflags not enums
bitflags! {
    /// State that matches to a resolver to allow for external users to track and compare states
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ResolverState: u16 {
        const NAMESPACE = 1 << 1;
        const MEMBER = 1 << 2;
        const TYPE = 1 << 3;
        const CONSTRAINT = 1 << 4;
        const COMPLETE = 1 << 5;
    }
}

//WARN: These are unsafe...
impl ResolverState {
    /// Gets what would be the next state
    pub fn next_state(self) -> ResolverState {
        match self {
            Self::NAMESPACE => Self::MEMBER,
            Self::MEMBER => Self::TYPE,
            Self::TYPE => Self::CONSTRAINT,
            _ => Self::COMPLETE,
        }
    }

    /// Gets what would be the previous state if possible
    pub fn prev_state(self) -> Option<ResolverState> {
        match self {
            Self::COMPLETE => Some(Self::TYPE),
            Self::TYPE => Some(Self::MEMBER),
            Self::MEMBER => Some(Self::NAMESPACE),
            Self::CONSTRAINT => Some(Self::TYPE),
            _ => None,
        }
    }

    /// Mutates current state to the next possible state
    pub fn advance(&mut self) {
        let out = match *self {
            Self::NAMESPACE => Self::MEMBER,
            Self::MEMBER => Self::TYPE,
            Self::TYPE => Self::CONSTRAINT,
            _ => Self::COMPLETE,
        };

        *self = out;
    }
}

//TEST:
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerStage {
    ModuleGraph,
    Lexer,
    Parser,
    NameResolver,
    MemberResolver,
    TypeResolver,
    ConstraintResolver,
    Complete,
}

impl CompilerStage {
    pub fn prev_stage(self) -> Option<CompilerStage> {
        let prev = match self {
            CompilerStage::ModuleGraph => return None,
            CompilerStage::Lexer => CompilerStage::ModuleGraph,
            CompilerStage::Parser => CompilerStage::Lexer,
            CompilerStage::NameResolver => CompilerStage::Parser,
            CompilerStage::MemberResolver => CompilerStage::NameResolver,
            CompilerStage::TypeResolver => CompilerStage::MemberResolver,
            CompilerStage::ConstraintResolver => CompilerStage::TypeResolver,
            CompilerStage::Complete => CompilerStage::ConstraintResolver,
        };
        Some(prev)
    }
    pub fn next_stage(self) -> CompilerStage {
        match self {
            CompilerStage::ModuleGraph => CompilerStage::Lexer,
            CompilerStage::Lexer => CompilerStage::Parser,
            CompilerStage::Parser => CompilerStage::NameResolver,
            CompilerStage::NameResolver => CompilerStage::NameResolver,
            CompilerStage::MemberResolver => CompilerStage::MemberResolver,
            CompilerStage::TypeResolver => CompilerStage::TypeResolver,
            CompilerStage::ConstraintResolver => CompilerStage::ConstraintResolver,
            CompilerStage::Complete => CompilerStage::Complete,
        }
    }
}

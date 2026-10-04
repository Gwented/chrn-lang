//TEST: May or may not have stages depend on parts of other stages so these are bitflags not enums
//
// // NOTE: Replacing this for now with `CompilerStage`
// bitflags! {
//     /// State that matches to a resolver to allow for external users to track and compare states
//     #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//     pub struct ResolverState: u16 {
//         const NAMESPACE = 1 << 1;
//         const MEMBER = 1 << 2;
//         const TYPE = 1 << 3;
//         const CONSTRAINT = 1 << 4;
//         const COMPLETE = 1 << 5;
//     }
// }
//
// //WARN: These are unsafe...
// impl ResolverState {
//     /// Gets what would be the next state
//     pub fn next_state(self) -> ResolverState {
//         match self {
//             Self::NAMESPACE => Self::MEMBER,
//             Self::MEMBER => Self::TYPE,
//             Self::TYPE => Self::CONSTRAINT,
//             _ => Self::COMPLETE,
//         }
//     }
//
//     /// Gets what would be the previous state if possible
//     pub fn prev_state(self) -> Option<ResolverState> {
//         match self {
//             Self::COMPLETE => Some(Self::TYPE),
//             Self::TYPE => Some(Self::MEMBER),
//             Self::MEMBER => Some(Self::NAMESPACE),
//             Self::CONSTRAINT => Some(Self::TYPE),
//             _ => None,
//         }
//     }
//
//     /// Mutates current state to the next possible state
//     pub fn advance(&mut self) {
//         let out = match *self {
//             Self::NAMESPACE => Self::MEMBER,
//             Self::MEMBER => Self::TYPE,
//             Self::TYPE => Self::CONSTRAINT,
//             _ => Self::COMPLETE,
//         };
//
//         *self = out;
//     }
// }

// bitflags! {
//     /// State that matches to a resolver to allow for external users to track and compare states
//     #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//     pub struct CompilerStage: u16 {
//         const MODULE_GRAPH = 1 << 1;
//         const LEXER = 1 << 2;
//         const PARSER = 1 << 3;
//         const NAMESPACE = 1 << 4;
//         const MEMBER = 1 << 5;
//         const TYPE = 1 << 6;
//         const CONSTRAINT = 1 << 7;
//         const COMPLETE = 1 << 8;
//     }
// }
//
// //WARN: These are unsafe...
// impl CompilerStage {
//     /// Gets what would be the next state
//     pub fn next_state(self) -> Self {
//         match self {
//             Self::MODULE_GRAPH => Self::LEXER,
//             Self::LEXER => Self::PARSER,
//             Self::PARSER => Self::NAMESPACE,
//             Self::NAMESPACE => Self::MEMBER,
//             Self::MEMBER => Self::TYPE,
//             Self::TYPE => Self::CONSTRAINT,
//             _ => Self::COMPLETE,
//         }
//     }
//
//     /// Gets what would be the previous state if possible
//     pub fn prev_state(self) -> Option<Self> {
//         match self {
//             Self::COMPLETE => Some(Self::CONSTRAINT),
//             Self::CONSTRAINT => Some(Self::TYPE),
//             Self::TYPE => Some(Self::MEMBER),
//             Self::MEMBER => Some(Self::NAMESPACE),
//             Self::NAMESPACE => Some(Self::PARSER),
//             Self::PARSER => Some(Self::LEXER),
//             Self::LEXER => Some(Self::MODULE_GRAPH),
//             Self::MODULE_GRAPH => None,
//             _ => None,
//         }
//     }
//
//     /// Mutates current state to the next possible state
//     pub fn advance(&mut self) {
//         *self = self.next_state();
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerStage {
    ModuleGraph,
    Lexer,
    Parser,
    Namespace,
    Member,
    Type,
    Constraint,
    Complete,
}

//WARN: These are unsafe...
impl CompilerStage {
    /// Gets what would be the next state
    pub fn next_state(self) -> Self {
        match self {
            Self::ModuleGraph => Self::Lexer,
            Self::Lexer => Self::Parser,
            Self::Parser => Self::Namespace,
            Self::Namespace => Self::Member,
            Self::Member => Self::Type,
            Self::Type => Self::Constraint,
            Self::Constraint => Self::Complete,
            Self::Complete => Self::Complete,
        }
    }

    /// Gets what would be the previous state if possible
    pub fn prev_state(self) -> Option<Self> {
        match self {
            Self::Complete => Some(Self::Constraint),
            Self::Constraint => Some(Self::Type),
            Self::Type => Some(Self::Member),
            Self::Member => Some(Self::Namespace),
            Self::Namespace => Some(Self::Parser),
            Self::Parser => Some(Self::Lexer),
            Self::Lexer => Some(Self::ModuleGraph),
            Self::ModuleGraph => None,
        }
    }

    /// Mutates current state to the next possible state
    pub fn advance(&mut self) {
        *self = self.next_state();
    }
}

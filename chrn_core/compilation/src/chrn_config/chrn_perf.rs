pub mod chrn_perf_concepts;

use std::time::{Duration, Instant};

use chrn_utils::{id_types::PathId, intern::Intern, utils::trackers::perf_tracker::PerfTracker};

use crate::chrn_config::chrn_perf::chrn_perf_concepts::{
    ChrnPerfReport, ChrnPerfTimeReport, ModuleIdentity, ModulePerfData, ModulePerfReport,
};

//TODO: How will this compensate for each module without being massively inconvenient
/// Compiler stages considered for performance review
pub const STAGES_COUNT: usize = ChrnPerfStage::CONSTRAINT_RESOLVER_IDX + 1;

/// Holds and orchestrates tracking info
#[derive(Debug, Default)]
pub struct ChrnPerf {
    /// Whether or not every all should be a no-op
    can_use: bool,
    /// Currently active identifier
    active_tracker: Option<(ModuleIdentity, PerfTracker)>,
    /// Modules registered under an identifier that have performance metadata
    tracked_mods: Vec<ModulePerfData>,
}

impl ChrnPerf {
    pub const fn new(can_use: bool) -> Self {
        Self {
            can_use,
            active_tracker: None,
            tracked_mods: Vec::new(),
        }
    }

    // What if this took in a stage, and if the stage on stop doesn't match the start then it fails
    // an assertion?
    /// Returns perf tracker to use for the current run
    pub fn start(&mut self, path_id: PathId) {
        if self.can_use() {
            let ident = ModuleIdentity::new(path_id);
            self.active_tracker = Some((ident, PerfTracker::new(Instant::now())));
        }
    }

    // Do we want to assert this exists to prevent dev errors?
    /// Stores stage's time using the active perf
    pub fn stop(&mut self, stage: ChrnPerfStage) {
        // Equivalent to can_use
        if let Some((active_ident, active_tracker)) = self.active_tracker {
            let existing_opt = self
                .tracked_mods
                .iter_mut()
                .find(|d| active_ident.path_id() == d.ident.path_id());

            let perf_data = if let Some(existing) = existing_opt {
                existing
            } else {
                let new_perf = ModulePerfData::new(active_ident);
                let idx = self.tracked_mods.len();
                self.tracked_mods.push(new_perf);

                &mut self.tracked_mods[idx]
            };

            let perf_out = active_tracker.stop();
            let time_report = ChrnPerfTimeReport::with_perf_output(
                stage,
                perf_out,
                perf_out.elapsed,
                perf_out.elapsed,
            );
            perf_data.tracked_stages[stage.to_idx()] = Some(time_report);
            self.active_tracker = None;
        }
    }

    // Why are we trying so hard to keep it const!
    pub fn form_report<'a>(&'a self, interner: &'a Intern) -> ChrnPerfReport<'a> {
        // global means
        let mut stage_means: [Option<ChrnPerfTimeReport>; STAGES_COUNT] = [None; STAGES_COUNT];
        // Wrapper for visualizing data
        let mut mod_reports = Vec::with_capacity(self.tracked_mods.len());

        for mod_perf_data in &self.tracked_mods {
            for (i, tracked) in mod_perf_data.tracked_stages.iter().enumerate() {
                if let Some(out) = tracked {
                    let stage = ChrnPerfStage::from_idx(i).expect("Idx should be aligned");
                    let mean_report = ChrnPerfTimeReport::new(
                        stage,
                        out.elapsed,
                        out.times,
                        out.elapsed,
                        out.elapsed,
                    );

                    if let Some(exists) = &mut stage_means[i] {
                        exists.merge(mean_report);
                    } else {
                        // Initial placement
                        stage_means[i] = Some(mean_report);
                    }
                }
            }
            let path = interner.search_path(mod_perf_data.ident.path_id());
            let mod_report = ModulePerfReport::new(path, mod_perf_data);
            mod_reports.push(mod_report);
        }
        ChrnPerfReport::new(stage_means, mod_reports)
    }

    // Might change again so stays wrapped
    /// Whether or not the tracker can be used
    pub const fn can_use(&self) -> bool {
        self.can_use
    }
}

///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ChrnPerfStage {
    ModuleGraph,
    Lexer,
    Parser,
    NamespaceResolver,
    MemberResolver,
    TypeResolver,
    ConstraintResolver,
}

impl ChrnPerfStage {
    pub const MODULE_GRAPH: usize = 0;
    pub const LEXER_IDX: usize = 1;
    pub const PARSER_IDX: usize = 2;
    pub const NAMESPACE_RESOLVER_IDX: usize = 3;
    pub const MEMBER_RESOLVER_IDX: usize = 4;
    pub const TYPE_RESOLVER_IDX: usize = 5;
    pub const CONSTRAINT_RESOLVER_IDX: usize = 6;

    pub const fn to_idx(self) -> usize {
        match self {
            ChrnPerfStage::ModuleGraph => Self::MODULE_GRAPH,
            ChrnPerfStage::Lexer => Self::LEXER_IDX,
            ChrnPerfStage::Parser => Self::PARSER_IDX,
            ChrnPerfStage::NamespaceResolver => Self::NAMESPACE_RESOLVER_IDX,
            ChrnPerfStage::MemberResolver => Self::MEMBER_RESOLVER_IDX,
            ChrnPerfStage::TypeResolver => Self::TYPE_RESOLVER_IDX,
            ChrnPerfStage::ConstraintResolver => Self::CONSTRAINT_RESOLVER_IDX,
        }
    }

    pub const fn from_idx(idx: usize) -> Option<ChrnPerfStage> {
        match idx {
            Self::MODULE_GRAPH => Some(ChrnPerfStage::ModuleGraph),
            Self::LEXER_IDX => Some(ChrnPerfStage::Lexer),
            Self::PARSER_IDX => Some(ChrnPerfStage::Parser),
            Self::NAMESPACE_RESOLVER_IDX => Some(ChrnPerfStage::NamespaceResolver),
            Self::MEMBER_RESOLVER_IDX => Some(ChrnPerfStage::MemberResolver),
            Self::TYPE_RESOLVER_IDX => Some(ChrnPerfStage::TypeResolver),
            Self::CONSTRAINT_RESOLVER_IDX => Some(ChrnPerfStage::ConstraintResolver),
            _ => None,
        }
    }
}

use std::{path::Path, time::Duration};

use chrn_utils::{id_types::PathId, utils::trackers::perf_tracker::PerfOutput};

use crate::chrn_config::chrn_perf::ChrnPerfStage;

#[derive(Debug, Clone, Copy)]
pub struct ModuleIdentity {
    // This is the only stable identifier
    path_id: PathId,
}

impl ModuleIdentity {
    pub fn new(path_id: PathId) -> Self {
        Self { path_id }
    }

    pub fn path_id(&self) -> PathId {
        self.path_id
    }
}

#[derive(Debug)]
pub struct ModulePerfData {
    /// Stable module identification
    pub ident: ModuleIdentity,
    /// Indexed through `ChrnPerfStage` constants
    pub tracked_stages: [Option<ChrnPerfTimeReport>; super::STAGES_COUNT],
}

impl ModulePerfData {
    pub fn new(ident: ModuleIdentity) -> Self {
        Self {
            ident,
            tracked_stages: [None; super::STAGES_COUNT],
        }
    }
}

#[derive(Debug)]
pub struct ModulePerfReport<'a> {
    pub path: &'a Path,
    pub data: &'a ModulePerfData,
}

impl<'a> ModulePerfReport<'a> {
    pub fn new(path: &'a Path, data: &'a ModulePerfData) -> Self {
        Self { path, data }
    }
}

/// Report derived from `ChrnPerf` which holds utility to access the data collected
#[derive(Debug)]
pub struct ChrnPerfReport<'a> {
    stage_means: [Option<ChrnPerfTimeReport>; 7],
    mod_reports: Vec<ModulePerfReport<'a>>,
    /// Options that are always applied
    opts: ChrnPerfReportOptions,
}

impl<'a> ChrnPerfReport<'a> {
    /// Sets `opts` to `all()`
    pub const fn new(
        stage_means: [Option<ChrnPerfTimeReport>; 7],
        mod_reports: Vec<ModulePerfReport<'a>>,
    ) -> Self {
        Self {
            stage_means,
            mod_reports,
            opts: ChrnPerfReportOptions::all(),
        }
    }

    pub const fn with_opts(
        stage_means: [Option<ChrnPerfTimeReport>; 7],
        mod_reports: Vec<ModulePerfReport<'a>>,
        opts: ChrnPerfReportOptions,
    ) -> Self {
        Self {
            stage_means,
            mod_reports,
            opts,
        }
    }

    pub fn print_all(&self, given_opts: ChrnPerfReportOptions) {
        Self::print_separators();
        self.print_all_mod_reports(given_opts);
        self.print_means(given_opts);
    }

    pub fn print_all_mod_reports(&self, given_opts: ChrnPerfReportOptions) {
        for i in 0..self.mod_reports.len() {
            let report = &self.mod_reports[i];
            println!("path: {}\n", report.path.display());

            for tracked_opt in &report.data.tracked_stages {
                let Some(tracked) = tracked_opt else { continue };
                let stage_opts = chrn_perf_stage_to_opt(tracked.stage);

                let closure =
                    || format!("stage = {:?} | time = {:?}", tracked.stage, tracked.elapsed);

                self.print_report(closure, given_opts, stage_opts);
            }

            if i + 1 != self.stage_means.len() {
                Self::print_separators();
            }
        }
    }

    pub fn print_means(&self, given_opts: ChrnPerfReportOptions) {
        for i in 0..self.stage_means.len() {
            let Some(mean_report) = self.stage_means[i] else {
                continue;
            };

            let closure = || {
                format!(
                    "stage: {:?}\nmean time = {:?}\ntimes ran = {:?}",
                    mean_report.stage, mean_report.elapsed, mean_report.times
                )
            };

            let stage_opts = chrn_perf_stage_to_opt(mean_report.stage);

            if self.print_report(closure, stage_opts, given_opts) {
                if i + 1 != self.stage_means.len() {
                    Self::print_separators();
                }
            };
        }
    }

    /// Returns `true` if printed, `false` otherwise
    fn print_report<F, T>(
        &self,
        f: F,
        opt1: ChrnPerfReportOptions,
        opt2: ChrnPerfReportOptions,
    ) -> bool
    where
        F: FnOnce() -> T,
        T: std::fmt::Display,
    {
        if (self.opts.flags & opt1.flags & opt2.flags) != 0 {
            println!("{}", f());
            true
        } else {
            false
        }
    }

    /// Prints 60 hyphens
    fn print_separators() {
        println!("------------------------------------------------------------");
    }

    pub fn stage_means(&self) -> [Option<ChrnPerfTimeReport>; 7] {
        self.stage_means
    }

    pub fn mod_reports(&self) -> &[ModulePerfReport<'a>] {
        &self.mod_reports
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChrnPerfReportOptions {
    flags: u16,
}

impl ChrnPerfReportOptions {
    pub const fn new(flags: u16) -> Self {
        Self { flags }
    }

    pub const fn all() -> Self {
        Self::new(Self::ALL)
    }

    pub const fn new_module_graph() -> Self {
        ChrnPerfReportOptions::new(Self::MODULE_GRAPH)
    }
    pub const fn new_lexer() -> Self {
        ChrnPerfReportOptions::new(Self::LEXER)
    }
    pub const fn new_parser() -> Self {
        ChrnPerfReportOptions::new(Self::PARSER)
    }
    pub const fn new_namespace_resolver() -> Self {
        ChrnPerfReportOptions::new(Self::NAMESPACE_RESOLVER)
    }
    pub const fn new_member_resolver() -> Self {
        ChrnPerfReportOptions::new(Self::MEMBER_RESOLVER)
    }
    pub const fn new_type_resolver() -> Self {
        ChrnPerfReportOptions::new(Self::TYPE_RESOLVER)
    }
    pub const fn new_constraint_resolver() -> Self {
        ChrnPerfReportOptions::new(Self::CONSTRAINT_RESOLVER)
    }

    // Looking. Odd.
    pub const MODULE_GRAPH: u16 = 1 << 0;
    pub const LEXER: u16 = 1 << 1;
    pub const PARSER: u16 = 1 << 2;
    pub const NAMESPACE_RESOLVER: u16 = 1 << 3;
    pub const MEMBER_RESOLVER: u16 = 1 << 4;
    pub const TYPE_RESOLVER: u16 = 1 << 5;
    pub const CONSTRAINT_RESOLVER: u16 = 1 << 6;
    /// Does perf check for all stages
    pub const ALL: u16 = Self::LEXER
        | Self::PARSER
        | Self::NAMESPACE_RESOLVER
        | Self::MEMBER_RESOLVER
        | Self::TYPE_RESOLVER
        | Self::CONSTRAINT_RESOLVER;
}

const fn chrn_perf_stage_to_opt(stage: ChrnPerfStage) -> ChrnPerfReportOptions {
    match stage {
        ChrnPerfStage::ModuleGraph => ChrnPerfReportOptions::new_module_graph(),
        ChrnPerfStage::Lexer => ChrnPerfReportOptions::new_lexer(),
        ChrnPerfStage::Parser => ChrnPerfReportOptions::new_parser(),
        ChrnPerfStage::NamespaceResolver => ChrnPerfReportOptions::new_namespace_resolver(),
        ChrnPerfStage::MemberResolver => ChrnPerfReportOptions::new_member_resolver(),
        ChrnPerfStage::TypeResolver => ChrnPerfReportOptions::new_type_resolver(),
        ChrnPerfStage::ConstraintResolver => ChrnPerfReportOptions::new_constraint_resolver(),
    }
}

impl std::default::Default for ChrnPerfReportOptions {
    fn default() -> Self {
        Self::all()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChrnPerfTimeReport {
    pub stage: ChrnPerfStage,
    pub elapsed: Duration,
    pub times: u16,
}

impl ChrnPerfTimeReport {
    pub const fn new(stage: ChrnPerfStage, elapsed: Duration, times: u16) -> Self {
        Self {
            stage,
            elapsed,
            times,
        }
    }

    pub const fn with_perf_output(stage: ChrnPerfStage, perf: PerfOutput) -> Self {
        Self {
            stage,
            elapsed: perf.elapsed,
            times: perf.times,
        }
    }
    pub fn merge(&mut self, other: ChrnPerfTimeReport) {
        debug_assert_eq!(self.stage, other.stage);
        self.elapsed += other.elapsed;
        self.times += other.times;
    }
}

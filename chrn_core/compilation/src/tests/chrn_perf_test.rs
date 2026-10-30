use std::path::Path;

use chrn_utils::{intern::Intern, source_utils::source_manager::SourceManager};

use crate::chrn_config::chrn_perf::{
    ChrnPerf, ChrnPerfStage, chrn_perf_concepts::ChrnPerfReportOptions,
};

#[test]
fn reports_preserve_source_identity_labels_and_stage_data() {
    let mut interner = Intern::init();
    let name = interner.intern_str("編集中");
    let mut sources = SourceManager::new();
    let path = sources
        .register_path(Path::new("/fixture/main.chrn"))
        .inner();
    let named = sources.register_in_memory(Some(name)).inner();
    let same_name = sources.register_in_memory(Some(name)).inner();
    let unnamed = sources.register_in_memory(None).inner();
    let ids = [path, named, same_name, unnamed];
    let mut perf = ChrnPerf::new(true);
    for id in ids {
        perf.start(id);
        perf.stop(ChrnPerfStage::Lexer);
    }
    // A second stage must join the existing source rather than create another report.
    perf.start(named);
    perf.stop(ChrnPerfStage::Parser);

    let report = perf.form_report(&sources, &interner);
    let modules = report.mod_reports();
    assert_eq!(modules.len(), ids.len());
    for ((module, id), label) in
        modules
            .iter()
            .zip(ids)
            .zip(["/fixture/main.chrn", "編集中", "編集中", "source"])
    {
        assert_eq!(module.data.ident.src_id(), id);
        assert_eq!(module.src_name, label, "source {id:?}");
        let lexer = module.data.tracked_stages[ChrnPerfStage::LEXER_IDX].unwrap();
        assert_eq!(lexer.stage, ChrnPerfStage::Lexer);
        assert_eq!(lexer.times, 1);
        assert_eq!(lexer.min, lexer.elapsed);
        assert_eq!(lexer.max, lexer.elapsed);
        for stage in [
            ChrnPerfStage::ModuleGraph,
            ChrnPerfStage::NamespaceResolver,
            ChrnPerfStage::MemberResolver,
            ChrnPerfStage::TypeResolver,
            ChrnPerfStage::ConstraintResolver,
        ] {
            assert!(
                module.data.tracked_stages[stage.to_idx()].is_none(),
                "{id:?}: {stage:?}"
            );
        }
        let parser = module.data.tracked_stages[ChrnPerfStage::PARSER_IDX];
        if id == named {
            let parser = parser.unwrap();
            assert_eq!(parser.stage, ChrnPerfStage::Parser);
            assert_eq!(parser.times, 1);
            assert_eq!(module.data.comp_time(), lexer.elapsed + parser.elapsed);
        } else {
            assert!(parser.is_none(), "parser timing leaked to {id:?}");
            assert_eq!(module.data.comp_time(), lexer.elapsed);
        }
    }

    let means = report.stage_means();
    let lexer_mean = means[ChrnPerfStage::LEXER_IDX].unwrap();
    let lexer_times: Vec<_> = modules
        .iter()
        .map(|module| {
            module.data.tracked_stages[ChrnPerfStage::LEXER_IDX]
                .unwrap()
                .elapsed
        })
        .collect();
    assert_eq!(lexer_mean.stage, ChrnPerfStage::Lexer);
    assert_eq!(lexer_mean.times, 4);
    assert_eq!(lexer_mean.elapsed, lexer_times.iter().copied().sum());
    assert_eq!(lexer_mean.min, *lexer_times.iter().min().unwrap());
    assert_eq!(lexer_mean.max, *lexer_times.iter().max().unwrap());
    assert_eq!(lexer_mean.mean(), lexer_mean.elapsed / 4);
    let parser_mean = means[ChrnPerfStage::PARSER_IDX].unwrap();
    assert_eq!(parser_mean.times, 1);
    assert_eq!(
        parser_mean.elapsed,
        modules[1].data.tracked_stages[ChrnPerfStage::PARSER_IDX]
            .unwrap()
            .elapsed
    );
    for stage in [
        ChrnPerfStage::ModuleGraph,
        ChrnPerfStage::NamespaceResolver,
        ChrnPerfStage::MemberResolver,
        ChrnPerfStage::TypeResolver,
        ChrnPerfStage::ConstraintResolver,
    ] {
        assert!(
            means[stage.to_idx()].is_none(),
            "unexpected mean for {stage:?}"
        );
    }
    // Exercise the public printing entry point, including populated per-module reports.
    report.print_all(ChrnPerfReportOptions::all());
}

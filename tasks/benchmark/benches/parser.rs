use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn bench_parser(criterion:&mut Criterion) {
	let mut group = criterion.benchmark_group("parser");
	for file in TestFiles::complicated().files() {
		let id = BenchmarkId::from_parameter(&file.file_name);
		let source_text = file.source_text.as_str();
		let source_type = SourceType::from_path(&file.file_name).unwrap();
		group.bench_function(id, |b| {
			// Do not include initializing allocator in benchmark.
			// User code would likely reuse the same allocator over and over to parse
			// multiple files, so we do the same here.
			let mut allocator = Allocator::default();
			b.iter(|| {
				Parser::new(&allocator, source_text, source_type)
					.with_options(ParseOptions {
						parse_regular_expression:true,
						..ParseOptions::default()
					})
					.parse();
				allocator.reset();
			});
		});
	}
	group.finish();
}

criterion_group!(parser, bench_parser);
criterion_main!(parser);

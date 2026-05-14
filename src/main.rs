mod model;

use model::NormalizedGraph;

fn main() {
    let model = NormalizedGraph::default();
    println!(
        "txviz normalized model initialized: {}",
        model.format_version
    );
}

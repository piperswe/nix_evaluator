use color_eyre::eyre::Result;
use nix_evaluator::evaluator::eval;
use rnix::parse;
use rustyline::Editor;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut rl = Editor::<()>::new();
    loop {
        let source = rl.readline("> ")?;
        rl.add_history_entry(source.as_str());
        let ast = parse(&source).as_result()?;
        let result = eval(ast.node())?.materialize_deep()?;
        println!("{}", result);
    }
}

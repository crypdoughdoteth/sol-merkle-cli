mod data_structures;
use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use data_structures::{cli_gen_proof, cli_gen_root, Leaf};
use std::path::PathBuf;
#[derive(Debug, Parser)]
#[clap(name = "sol-merkle", about = ABOUT, after_help = EXAMPLES, version)]
struct App {
    #[clap(short, long)]
    path: PathBuf,
    #[clap(subcommand)]
    pub cmd: Command,
}
#[derive(Debug, Args)]
struct Index {
    #[clap(short, long)]
    idx: Option<usize>,
}

#[derive(Debug, Subcommand)]
enum Command {
    GetRoot,
    GenerateProof(Index),
    // add multi-proof generation + concurrency
    VerifyProof, // multi-proof verification
}

const ABOUT: &str = "A command line tool for generating Merkle Trees, Proofs, and Verifier based on Ethereum's keccak256 & ABI encoding.";
const EXAMPLES: &str = r#"
EXAMPLES:

    #

"#;

fn main() -> Result<()> {
    let app = App::parse();
    let path: PathBuf = app.path;
    let _ = match app.cmd {
        Command::GetRoot => cli_gen_root(&path),
        Command::GenerateProof(args) => cli_gen_proof(&path, args.idx),
        Command::VerifyProof => todo!(),
    };
    Ok(())
}

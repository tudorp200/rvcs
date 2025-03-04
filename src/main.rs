use clap::{arg, command, Command};
mod commands;
mod objects;
mod others;

fn cli() -> Command {
    command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("init")
                .about("Create an empty SVN directory or reinitialize an existing one"),
        )
        .subcommand(
            Command::new("add")
                .about("Add files contents to the index")
                .arg(arg!([NAME] "The name of the file to add").required(true)),
        )
        .subcommand(
            Command::new("commit")
                .about("Record changes to the repository")
                .arg(arg!([NAME] "The commit message").required(true)),
        )
        .subcommand(
            Command::new("dec-object")
                .about("Decompressing an object information and the print in the standard output")
                .arg(arg!([NAME]"The hash of the object").required(true)),
        )
        .subcommand(
            Command::new("ls-files")
                .about("Pretty print all the files that are stagged in index file"),
        )
        .subcommand(Command::new("status").about("Show the working tree status"))
        .subcommand(
            Command::new("write-tree").about("Records the content of the index in a tree object"),
        )
        .subcommand(
            Command::new("commit-tree")
                .about("Create a commit object that reference a tree")
                .arg(arg!([TREE_HASH]"Tree hash").required(true))
                .arg(arg!([COMMIT_MSG]"The commit message").required(true)),
        )
        .subcommand(
            Command::new("branch")
                .about("Create a new branch")
                .arg(arg!([NAME]"Branch name").required(true)),
        )
        .subcommand(
            Command::new("checkout")
                .about("Moving to a a branch")
                .arg(arg!([NAME]"Branch Name").required(true)),
        )
        .subcommand(
            Command::new("diff-files")
                .about("Diff between files")
                .arg(arg!([FILE1]"File1").required(true))
                .arg(arg!([FILE2]"File2").required(true)),
        )
        .subcommand(
            Command::new("diff-commit")
                .about("Diff between two commits")
                .arg(arg!([HASH1]"Firs commit hash").required(true))
                .arg(arg!([HASH2]"Second commit hash").required(true)),
        )
        .subcommand(Command::new("diff").about("Diff between current commit and previous one."))
        .subcommand(
            Command::new("diff-branch")
                .about("Diff between two branches")
                .arg(arg!([BRANCH1]"First branch name"))
                .arg(arg!([BRANCH2]"Second branch name")),
        )
        .subcommand(
            Command::new("ff-merge")
                .about("No fast forward merge between two branches")
                .arg(arg!([BRANCH1]"First branch name").required(true))
                .arg(arg!([BRANCH2]"Second branch name").required(true))
                .arg(arg!(-a --"auto-resolve" "Automatically resolve conflicts, the file will be overwrite with the file in the branch you want to merge to.").required(false))
        )
        .subcommand(
            Command::new("merge3")
                .about("Three way merge between two branches")
                .arg(arg!([BRANCH1]"First branch name").required(true))
                .arg(arg!([BRANCH2]"Second branch name").required(true))
                .arg(arg!(-a --"auto-resolve" "Automatically resolve conflicts, the file will be overwrite with the file in the branch you want to merge to.").required(false))
                )
        .subcommand(
            Command::new("merge")
                .about("Merge between the current branch and another, the merge algorithm will be choosen by the program.")
                .arg(arg!([BRANCH]"Branch name").required(true))
                .arg(arg!(-a --"auto-resolve" "Automatically resolve conflicts, the file will be overwrite with the file in the branch you want to merge to.").required(false))
                )
}

fn main() {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("diff-files", sub_matches)) => {
            let file1 = sub_matches.get_one::<String>("FILE1");
            let file2 = sub_matches.get_one::<String>("FILE2");
            if file1.is_some() && file2.is_some() {
                match commands::diff::diff_between_files(file1.unwrap(), file2.unwrap()) {
                    Ok(ans) => {
                        println!("{}", ans);
                    }
                    Err(e) => {
                        println!("Error {}", e);
                    }
                }
            }
        }
        Some(("ff-merge", sub_matches)) => {
            let branch1 = sub_matches.get_one::<String>("BRANCH1");
            let branch2 = sub_matches.get_one::<String>("BRANCH2");
            let flag = sub_matches.get_flag("auto-resolve");
            if let Err(e) =
                commands::merge::fast_forward_merge(branch1.unwrap(), branch2.unwrap(), flag)
            {
                eprintln!("Err: {}", e);
            }
        }
        Some(("merge3", sub_matches)) => {
            let branch1 = sub_matches.get_one::<String>("BRANCH1");
            let branch2 = sub_matches.get_one::<String>("BRANCH2");
            let flag = sub_matches.get_flag("auto-resolve");
            if let Err(e) =
                commands::merge::three_way_merge(branch1.unwrap(), branch2.unwrap(), flag)
            {
                eprintln!("Err: {}", e);
            }
        }
        Some(("merge", sub_matches)) => {
            let branch1 = sub_matches.get_one::<String>("BRANCH");
            let flag = sub_matches.get_flag("auto-resolve");
            if let Err(e) = commands::merge::merge(branch1.unwrap(), flag) {
                eprintln!("Err: {}", e);
            }
        }
        Some(("diff-commit", sub_matches)) => {
            let hash1 = sub_matches.get_one::<String>("HASH1");
            let hash2 = sub_matches.get_one::<String>("HASH2");
            match commands::diff::diff_between_commits(hash1.unwrap(), hash2.unwrap()) {
                Ok(ans) => {
                    println!("{}", ans);
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }
        Some(("diff", _)) => {
            if let Err(err) = commands::diff::diff_between_current_last_commit() {
                println!("{}", err);
            }
        }
        Some(("diff-branch", sub_matches)) => {
            let hash1 = sub_matches.get_one::<String>("BRANCH1");
            let hash2 = sub_matches.get_one::<String>("BRANCH2");
            if let Err(err) = commands::diff::diff_between_branches(hash1.unwrap(), hash2.unwrap())
            {
                println!("{}", err);
            }
        }
        Some(("add", sub_matches)) => {
            let name = sub_matches.get_one::<String>("NAME");
            if let Err(err) = commands::add::add_protocol(name.unwrap()) {
                eprintln!("Error: {}", err);
            }
        }
        Some(("init", _)) => {
            if let Err(err) = commands::init::initialize_repo() {
                eprintln!("Erorr: {}", err);
            }
        }
        Some(("ls-files", _)) => {
            if let Err(err) = commands::ls_files::get_info() {
                eprintln!("Error: {}", err);
            }
        }
        Some(("dec-object", sub_matches)) => {
            let hash = sub_matches.get_one::<String>("NAME");
            if let Err(err) = commands::dec_object::get_info(hash.unwrap()) {
                eprintln!("Error: {}", err);
            }
        }
        Some(("write-tree", _)) => {
            if let Err(err) = commands::write_tree::write_tree_command() {
                eprintln!("Error: {}", err);
            }
        }

        Some(("commit", sub_matches)) => {
            let msg = sub_matches.get_one::<String>("NAME");
            if let Err(err) = commands::commit::commit_command(msg.unwrap()) {
                eprintln!("Error: {}", err);
            }
        }
        Some(("commit-tree", sub_matches)) => {
            let tree_hash = sub_matches.get_one::<String>("TREE_HASH");
            let commit_msg = sub_matches.get_one::<String>("COMMIT_MSG");
            if let Err(err) =
                commands::commit::commit_tree_command(tree_hash.unwrap(), commit_msg.unwrap())
            {
                eprintln!("Error: {}", err);
            }
        }
        Some(("status", _)) => {
            if let Err(err) = commands::status::status_command() {
                eprintln!("Error: {}", err);
            }
        }
        Some(("branch", sub_matches)) => {
            let branch_name = sub_matches.get_one::<String>("NAME");
            if let Err(err) = commands::branch::branch_command(branch_name.unwrap()) {
                eprintln!("Error: {}", err);
            }
        }
        Some(("checkout", sub_matches)) => {
            let branch_name = sub_matches.get_one::<String>("NAME");
            if let Err(err) = commands::checkout::checkout(branch_name.unwrap()) {
                eprintln!("Error: {}", err);
            }
        }
        _ => unreachable!("subcommand_required ensures this branch won't be reached"),
    }
}

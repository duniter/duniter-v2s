// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Substrate-Libre-Currency is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Substrate-Libre-Currency. If not, see <https://www.gnu.org/licenses/>.

fn main() {
    //cli::main();

    substrate_build_script_utils::generate_cargo_keys();
    substrate_build_script_utils::rerun_if_git_head_changed();
}
/*
mod cli {
    include!("src/cli.rs");

    use clap::{ArgEnum, IntoApp};
    use clap_complete::{generate_to, Shell};
    use std::{env, fs, path::Path};
    use substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

    pub fn main() {
        build_shell_completion();
        generate_cargo_keys();

        rerun_if_git_head_changed();
    }

    /// Build shell completion scripts for all known shells
    fn build_shell_completion() {
        for shell in Shell::value_variants() {
            build_completion(shell);
        }
    }

    /// Build the shell auto-completion for a given Shell
    fn build_completion(shell: &Shell) {
        let outdir = match env::var_os("OUT_DIR") {
            None => return,
            Some(dir) => dir,
        };
        let path = Path::new(&outdir)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("completion-scripts");

        fs::create_dir(&path).ok();

        let _ = generate_to(*shell, &mut Cli::into_app(), "substrate-node", &path);
    }
}*/

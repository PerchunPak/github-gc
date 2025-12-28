use std::collections::HashMap;

use tracing::*;

use super::get_forks::Fork;
use super::get_prs::{PR, PullRequestState};

#[allow(dead_code)]
pub struct RepoAndBranch {
    pub repo: String,
    pub branch: String,
}

pub fn collect_merged_branches(
    forks: &HashMap<String, Fork>,
    prs: &Vec<PR>,
) -> Vec<RepoAndBranch> {
    let mut to_delete: Vec<RepoAndBranch> = vec![];

    for pr in prs {
        if pr.state == PullRequestState::OPEN {
            continue;
        };

        let fork = match forks.get(&pr.repo) {
            Some(v) => v,
            None => {
                error!("PR for {} found, but didn't find a fork!", pr.repo);
                continue;
            }
        };

        if fork.default_branch_name == pr.branch_name {
            continue;
        };

        to_delete.push(RepoAndBranch {
            repo: pr.repo.clone(),
            branch: pr.branch_name.clone(),
        })
    }

    return to_delete;
}

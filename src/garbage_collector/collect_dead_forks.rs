use std::collections::{HashMap, HashSet};
use std::vec;

use crate::garbage_collector::get_forks::ForkBranchInfo;
use crate::garbage_collector::get_prs::PullRequestState;

use super::get_forks::Fork;
use super::get_prs::PR;

pub enum ForkBranchState {
    NoPR,
    HasOpenPR,
    // has associated merged PR and its commit equal to PR's commit
    Dead,
    // has associated merged PR and its commit *not* equal to PR's commit
    Different,
}

pub enum ForkSpecialState {
    OnlyDefaultBranch,
    NoPRs,
}

pub struct ForkBranchWithState {
    pub branch: ForkBranchInfo,
    pub pr: Option<PR>,
    pub state: ForkBranchState,
}

pub struct ForkDeadnessInfo {
    pub fork: Fork,
    pub branches: Vec<ForkBranchWithState>,
    pub special_state: Option<ForkSpecialState>,
}

pub fn collect_dead_forks(
    forks_vec: &Vec<Fork>,
    prs: &Vec<PR>,
) -> Vec<ForkDeadnessInfo> {
    let forks = vec_forks_to_hashmap(forks_vec);
    let mut result: Vec<ForkDeadnessInfo> = vec![];
    let branches_map = prs_to_branches(prs);

    for (repo, fork) in forks.iter() {
        let mut result_branches: Vec<ForkBranchWithState> = vec![];
        for fork_branch in fork.branches.iter() {
            let pr = match prs
                .iter()
                .filter(|pr| pr.branch_name == *fork_branch.name)
                .next()
            {
                Some(pr) => pr,
                None => {
                    result_branches.push(ForkBranchWithState {
                        branch: fork_branch.clone(),
                        pr: None,
                        state: ForkBranchState::NoPR,
                    });
                    continue;
                }
            };

            result_branches.push(ForkBranchWithState {
                branch: fork_branch.clone(),
                pr: Some(pr.clone()),
                state: if pr.state == PullRequestState::OPEN {
                    ForkBranchState::HasOpenPR
                } else if pr.commit == fork_branch.commit {
                    ForkBranchState::Dead
                } else {
                    ForkBranchState::Different
                },
            });
        }

        let special_state: Option<ForkSpecialState> =
            if branches_map.contains_key(&repo.clone()) {
                Some(ForkSpecialState::NoPRs)
            } else if fork.branches.len() == 1
                && fork.branches[0].name == fork.default_branch_name
            {
                Some(ForkSpecialState::OnlyDefaultBranch)
            } else {
                None
            };

        result.push(ForkDeadnessInfo {
            fork: fork.clone(),
            branches: result_branches,
            special_state: special_state,
        });
    }

    return result;
}

fn prs_to_branches(prs: &Vec<PR>) -> HashMap<String, HashSet<String>> {
    let mut result: HashMap<String, HashSet<String>> = HashMap::new();

    for pr in prs {
        result
            .entry(pr.repo.to_string())
            .or_insert(HashSet::new())
            .insert(pr.branch_name.to_string());
    }

    return result;
}

fn vec_forks_to_hashmap(forks: &Vec<Fork>) -> HashMap<String, Fork> {
    let mut map: HashMap<String, Fork> = HashMap::new();

    for fork in forks {
        map.insert(fork.name.to_string(), fork.clone());
    }

    return map;
}

use crate::{
    potlock_admin_interactions::ProjectPercentage,
    potlock_storage::{self, PotlockId, ProjectId, Status},
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const MAX_PERCENTAGE: u64 = 10_000; // 100%

#[multiversx_sc::module]
pub trait PotlockRequirements: potlock_storage::PotlockStorage {
    fn is_valid_potlock_id(&self, potlock_id: PotlockId) -> bool {
        potlock_id >= 1 && potlock_id <= self.potlocks().len()
    }

    fn require_potlock_exists(&self, potlock_id: PotlockId) {
        require!(
            self.is_valid_potlock_id(potlock_id) && !self.potlocks().item_is_empty(potlock_id),
            "Potlock doesn't exist!",
        )
    }

    fn require_potlock_is_active(&self, potlock_id: PotlockId) {
        let potlock = self.potlocks().get(potlock_id);
        require!(potlock.status == Status::Active, "Pot is not active!",)
    }

    fn require_potlock_is_inactive(&self, potlock_id: PotlockId) {
        let potlock = self.potlocks().get(potlock_id);
        require!(potlock.status != Status::Active, "Pot is active!",)
    }

    fn is_valid_project_id(&self, project_id: ProjectId) -> bool {
        project_id >= 1 && project_id <= self.projects().len()
    }

    fn require_project_exists(&self, project_id: ProjectId) {
        require!(
            self.is_valid_project_id(project_id) && !self.projects().item_is_empty(project_id),
            "Project doesn't exist!",
        )
    }

    fn require_project_is_active(&self, project_id: ProjectId) {
        let project = self.projects().get(project_id);
        require!(project.status == Status::Active, "Project is not active!",)
    }

    fn require_project_is_inactive(&self, project_id: ProjectId) {
        let project = self.projects().get(project_id);
        require!(project.status != Status::Active, "Project is active!",)
    }

    fn require_correct_percentages(
        &self,
        project_percentages: MultiValueEncoded<ProjectPercentage>,
    ) {
        let mut total_perc: u64 = 0;
        for pp in project_percentages {
            let (_, perc) = pp.into_tuple();
            total_perc += perc;
        }
        require!(
            total_perc <= MAX_PERCENTAGE,
            "Total percentages more than 100%"
        );
    }
}

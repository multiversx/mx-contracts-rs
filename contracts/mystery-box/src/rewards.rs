use multiversx_sc::imports::*;

use crate::config::{self, Reward, RewardType, MAX_PERCENTAGE};

#[multiversx_sc::module]
pub trait RewardsModule: config::ConfigModule {
    fn get_winning_reward(
        &self,
        rewards_list: &ManagedVec<Reward<Self::Api>>,
    ) -> Reward<Self::Api> {
        let mut winning_reward = Reward::default();
        if rewards_list.is_empty() {
            return winning_reward;
        }

        let mut rng = RandomnessSource::new();
        let winner_number = rng.next_u64_in_range(1, MAX_PERCENTAGE + 1);
        let mut cumulative_percentage = 0;

        for reward in rewards_list {
            cumulative_percentage += reward.percentage_chance;
            if winner_number <= cumulative_percentage {
                winning_reward = reward.clone();
                break;
            }
        }

        winning_reward
    }

    fn check_reward_validity(&self, reward: &Reward<Self::Api>) {
        match reward.reward_type {
            RewardType::ExperiencePoints => {
                require!(
                    reward.value > 0,
                    "The experience points amount must be greater than 0"
                );
            }
            RewardType::MysteryBox => {
                require!(
                    reward.reward_token_id == self.mystery_box_token_id().get(),
                    "The reward token id must be the same as the mystery box"
                );
            }
            RewardType::SFT => {
                require!(
                    reward.reward_token_id.is_esdt(),
                    "The reward token id must be an ESDT"
                );
            }
            RewardType::PercentValue => {
                require!(
                    reward.value > 0 && reward.value <= MAX_PERCENTAGE,
                    "The reward percentage must be positive and <= 100%"
                );
            }
            RewardType::FixedValue => {
                require!(reward.value > 0, "The reward amount must be greater than 0");
            }
            RewardType::CustomReward => {
                require!(
                    !reward.description.is_empty(),
                    "The custom reward needs to have a description"
                );
            }
            _ => {}
        }
    }

    fn check_global_cooldown(&self, current_epoch: u64, reward: &Reward<Self::Api>) -> bool {
        let global_cooldown_epoch = self.global_cooldown_epoch(&reward.reward_type).get();

        if reward.epochs_cooldown == 0 {
            false
        } else if global_cooldown_epoch <= current_epoch {
            self.global_cooldown_epoch(&reward.reward_type)
                .set(current_epoch + reward.epochs_cooldown);
            false
        } else {
            true
        }
    }
}

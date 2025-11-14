multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::config::{self, CooldownType, Reward, RewardCooldown, RewardType, MAX_PERCENTAGE};

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

    fn check_reward_validity(&self, reward: &Reward<Self::Api>, reward_cooldown: &RewardCooldown) {
        if reward_cooldown.cooldown_type == CooldownType::ResetOnCooldown {
            require!(
                reward_cooldown.wins_per_cooldown > 0 && reward_cooldown.cooldown_epochs > 0,
                "Invalid cooldown input for resettable cooldown type"
            );
        }
        if reward_cooldown.cooldown_epochs > 0 {
            require!(
                reward_cooldown.wins_per_cooldown > 0,
                "Wins per cooldown must be greater than 0 for rewards with cooldown"
            );
        }
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

    fn check_reward_cooldown(&self, current_epoch: u64, reward: &Reward<Self::Api>) -> bool {
        let reward_cooldown_mapper = self.reward_cooldown(reward.reward_id);
        if reward_cooldown_mapper.is_empty() {
            return false;
        };

        let mut reward_cooldown = reward_cooldown_mapper.get();
        if reward_cooldown.cooldown_type == CooldownType::None {
            return false;
        };

        let mut cooldown_check = true;

        if reward_cooldown.cooldown_type == CooldownType::ResetOnCooldown
            && current_epoch >= reward_cooldown.last_update_epoch + reward_cooldown.cooldown_epochs
        {
            reward_cooldown.last_update_epoch = current_epoch;
            reward_cooldown.remaining_epoch_wins = reward_cooldown.wins_per_cooldown;
        }

        if reward_cooldown.remaining_epoch_wins > 0 {
            reward_cooldown.remaining_epoch_wins -= 1;
            cooldown_check = false;
        }

        reward_cooldown_mapper.set(reward_cooldown);

        cooldown_check
    }
}

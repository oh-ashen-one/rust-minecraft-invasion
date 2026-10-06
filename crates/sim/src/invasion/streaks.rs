use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reward {
    Uav,
    CarePackage,
    CounterUav,
    Sentry,
    Predator,
    Precision,
    Harrier,
    Helicopter,
    Emergency,
    PaveLow,
    Stealth,
    ChopperGunner,
    Ac130,
    Emp,
    Nuke,
}
impl Reward {
    pub const ALL: [Self; 15] = [
        Self::Uav,
        Self::CarePackage,
        Self::CounterUav,
        Self::Sentry,
        Self::Predator,
        Self::Precision,
        Self::Harrier,
        Self::Helicopter,
        Self::Emergency,
        Self::PaveLow,
        Self::Stealth,
        Self::ChopperGunner,
        Self::Ac130,
        Self::Emp,
        Self::Nuke,
    ];
    pub fn cost(self) -> u32 {
        match self {
            Self::Uav => 3,
            Self::CarePackage | Self::CounterUav => 4,
            Self::Sentry | Self::Predator => 5,
            Self::Precision => 6,
            Self::Harrier | Self::Helicopter => 7,
            Self::Emergency => 8,
            Self::PaveLow | Self::Stealth => 9,
            Self::ChopperGunner | Self::Ac130 => 11,
            Self::Emp => 15,
            Self::Nuke => 25,
        }
    }
    pub fn script_name(self) -> &'static str {
        match self {
            Self::Uav => "uav",
            Self::CarePackage => "airdrop",
            Self::CounterUav => "counter_uav",
            Self::Sentry => "airdrop_sentry_minigun",
            Self::Predator => "predator_missile",
            Self::Precision => "precision_airstrike",
            Self::Harrier => "harrier_airstrike",
            Self::Helicopter => "helicopter",
            Self::Emergency => "airdrop_mega",
            Self::PaveLow => "helicopter_flares",
            Self::Stealth => "stealth_airstrike",
            Self::ChopperGunner => "helicopter_minigun",
            Self::Ac130 => "ac130",
            Self::Emp => "emp",
            Self::Nuke => "nuke",
        }
    }
}
#[derive(Default)]
pub struct Streaks {
    pub count: u32,
    pub pending: VecDeque<Reward>,
    awarded: u16,
}
impl Streaks {
    pub fn award_kill(&mut self) {
        self.count += 1;
        for (i, reward) in Reward::ALL.into_iter().enumerate() {
            if self.count >= reward.cost() && self.awarded & (1 << i) == 0 {
                self.awarded |= 1 << i;
                self.pending.push_back(reward);
            }
        }
    }
    pub fn reset_life(&mut self) {
        self.count = 0;
        self.awarded = 0;
    }
}

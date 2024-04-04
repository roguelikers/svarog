use std::collections::HashSet;

use bevy::ecs::component::Component;
use super::{value::Value, Amount, Index, React, Time};
use itertools::Itertools;
use std::fmt::Debug;

#[derive(Default, Component, Debug)]
pub struct Health {
    pub hit_dice: Vec<HitDie>,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum HitDieStatus {
    /// A `Void` Hit Die has no influence
    Void,
    /// An `Empty` Hit Die has zero hit points in it
    Empty,
    /// A `Grafted` Hit Die gives no effort
    Grafted,
    /// A `Temporary` Hit Die shatters if empty
    Temporary,
    /// A `Guarded` Hit Die can't be Broken, or Drained
    Guarded,
    /// When a `Fortified` Hit Die would become Empty, if its value is greater than zero, heal completely 
    /// and reduce the value by 1, and remove if at 0
    Fortified(Amount),  
    /// A `Stifled` Hit Die gives no effort for the next number of turns
    Stifled(Time),
    /// A `Cracked` Hit Die chips for 1 damage every set number of turns
    Cracked(Time),
    /// A `Mending` Hit Die heals 1 damage every set number of turns
    Mending(Time),
}

#[derive(Debug)]
pub enum Effort {
    Uncommited,
    Commited(Commitment)
}

#[derive(Debug)]
pub enum Commitment {
    Turn(Amount),
    Encounter,
    Rest,
}

#[derive(Debug)]
pub enum Influence {}

#[derive(Debug)]
pub struct HitDie {
    pub value: Value,
    pub effort: Effort,
    pub statuses: HashSet<HitDieStatus>,
    pub influences: Vec<Influence>,
}

impl HitDie {
    fn new(size: u32) -> Self {
        Self { 
            value: Value::new(size),
            effort: Effort::Uncommited, 
            statuses: HashSet::new(),
            influences: Vec::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum HealthAction {
    /// The `Create` action adds a Hit Die of some size to the health bar
    Create(Amount),
    /// The `AddStatus` action adds a status to the right-most Hit Die
    AddStatus(HitDieStatus),
    /// The `RemoveStatus` action removes a status to the right-most Hit Die
    RemoveStatus(HitDieStatus),
    /// The `Chip` action reduces the right-most Hit Die for a set amount of hitpoints,
    /// spilling over to the left if there's any remaining damage
    Chip(Amount),
    /// The `Heal` action recovers the left-most Hit Die for a set amount of hitpoints,
    /// spilling over to the right if there's any healing remaining
    Heal(Amount),
    /// The `Drain` action empties the right-most non-empty Hit Die completely
    Drain,
    /// The `Break` action afflicts the right-most non-Void Hit Die with the **Void** status
    Break,
    /// The `Mend` action removes the **Void** status from the left-most HitDie with that status 
    Mend,
    /// The `Shatter` action removes the right-most Hit Die completely
    Shatter,
    /// The `Fortify` action makes the right-most Hit Die **Fortified**
    Fortify(Amount),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum HitDieActionResponse {
    None,
    ChipResponse(Amount),
    HealResponse(Amount),
    DrainResponse,
    BreakResponse,
    MendResponse,
    ShatterResponse,
    FortifiedResponse(Amount),
    RecoveryResponse,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum HealthActionResponse {
    None,
    CreateResponse(Index),
    ChipResponse(Index, Amount),
    HealResponse(Index, Amount),
    DrainResponse(Index),
    FortifiedResponse(Index, Amount),
    RecoveryResponse(Index),
    BreakResponse(Index),
    MendResponse(Index),
    ShatterResponse(Index),
}

impl From<(&HitDieActionResponse, Index)> for HealthActionResponse {
    fn from(value: (&HitDieActionResponse, Index)) -> Self {
        match value.0 {
            HitDieActionResponse::None => HealthActionResponse::None,
            HitDieActionResponse::DrainResponse => HealthActionResponse::DrainResponse(value.1),
            HitDieActionResponse::BreakResponse => HealthActionResponse::BreakResponse(value.1),
            HitDieActionResponse::MendResponse => HealthActionResponse::MendResponse(value.1),
            HitDieActionResponse::ChipResponse(r) => HealthActionResponse::ChipResponse(value.1, *r),
            HitDieActionResponse::HealResponse(r) => HealthActionResponse::HealResponse(value.1, *r),
            HitDieActionResponse::FortifiedResponse(f) => HealthActionResponse::FortifiedResponse(value.1, *f),
            HitDieActionResponse::RecoveryResponse => HealthActionResponse::RecoveryResponse(value.1),
            HitDieActionResponse::ShatterResponse => HealthActionResponse::ShatterResponse(value.1),
        }
    }
}

impl React<HealthAction, HitDieActionResponse> for HitDie {
    fn execute(&mut self, action: HealthAction) -> HitDieActionResponse {
        let main_response = match action {
            HealthAction::Chip(n) if n > 0 => {
                HitDieActionResponse::ChipResponse(self.value.reduce(n as u32))
            },
            HealthAction::AddStatus(status) => {
                self.statuses.insert(status);
                HitDieActionResponse::None
            },
            HealthAction::RemoveStatus(status) => {
                self.statuses.remove(&status);
                HitDieActionResponse::None
            },
            HealthAction::Heal(n) if n > 0 => {
                self.statuses.remove(&HitDieStatus::Empty);
                HitDieActionResponse::HealResponse(self.value.add(n as u32))
            },
            HealthAction::Drain if self.statuses.contains(&HitDieStatus::Guarded) => {
                HitDieActionResponse::RecoveryResponse
            },
            HealthAction::Drain => {
                self.value.empty();
                HitDieActionResponse::DrainResponse
            },
            HealthAction::Break if self.statuses.contains(&HitDieStatus::Guarded) => {
                HitDieActionResponse::RecoveryResponse
            },
            HealthAction::Break => {
                self.statuses.insert(HitDieStatus::Void);
                HitDieActionResponse::BreakResponse
            },
            HealthAction::Mend => {
                self.statuses.remove(&HitDieStatus::Void);
                HitDieActionResponse::MendResponse
            },
            HealthAction::Fortify(v) => {
                let n = if let Some(&HitDieStatus::Fortified(n)) = 
                    self.statuses.iter().find(|s| matches!(s, HitDieStatus::Fortified(_))) {
                    Some(n)
                } else { 
                    None 
                };

                let value = if let Some(n) = n {
                    self.statuses.remove(&HitDieStatus::Fortified(n));
                    self.statuses.insert(HitDieStatus::Fortified(n + v));
                    n + v
                } else {
                    self.statuses.insert(HitDieStatus::Fortified(v));
                    v
                };

                HitDieActionResponse::FortifiedResponse(value)
            },
            _ => HitDieActionResponse::None,
        };

        let statuses = self.statuses.iter().cloned().collect_vec();

        if *self.value == 0 {
            let armor = if let Some(HitDieStatus::Fortified(armor)) = 
                statuses.iter().find(|p| matches!(p, HitDieStatus::Fortified(_))) {
                
                Some(*armor)
            } else {
                None
            };

            if let Some(armor) = armor { 
                self.statuses.remove(&HitDieStatus::Fortified(armor));
                self.execute(HealthAction::Heal(self.value.total()));
                if armor > 1 {
                    self.statuses.insert(HitDieStatus::Fortified(armor - 1));
                }
                return HitDieActionResponse::RecoveryResponse;
            }
        }

        if *self.value == 0 && self.statuses.contains(&HitDieStatus::Temporary) {
            return HitDieActionResponse::ShatterResponse;
        }

        if *self.value == 0 {
            self.statuses.insert(HitDieStatus::Empty);
        }

        main_response
    }
}

impl React<HealthAction, Vec<HealthActionResponse>> for Health {
    fn execute(&mut self, action: HealthAction) -> Vec<HealthActionResponse> {
        if self.hit_dice.is_empty() && !matches!(action, HealthAction::Create(_)) { return vec![ HealthActionResponse::None ]; }

        match action {
            HealthAction::Create(size) if size > 0 => {
                self.hit_dice.push(HitDie::new(size as u32));
                vec![ HealthActionResponse::CreateResponse(self.hit_dice.len() - 1) ]
            },
            HealthAction::AddStatus(status) => {
                self.hit_dice.last_mut().map(|hd| hd.execute(HealthAction::AddStatus(status)));
                vec![ HealthActionResponse::None ]
            },
            HealthAction::RemoveStatus(status) => {
                self.hit_dice.last_mut().map(|hd| hd.execute(HealthAction::RemoveStatus(status)));
                vec![ HealthActionResponse::None ]
            },
            HealthAction::Chip(mut damage) if damage > 0 => {
                let mut result = vec![];
                let mut last = self.hit_dice.len() - 1;
                
                loop {
                    let reaction = self.hit_dice[last].execute(HealthAction::Chip(damage));
                    result.push((&reaction, last).into());

                    if let HitDieActionResponse::ShatterResponse = reaction {
                        self.hit_dice.remove(last);
                        break;
                    } else if let HitDieActionResponse::ChipResponse(rest) = reaction {
                        if rest > 0 && last > 0 {
                            damage = rest;
                            last -= 1;
                        } else {
                            break;
                        }
                    } else if let HitDieActionResponse::RecoveryResponse = reaction {
                        break;
                    }
                }

                result
            },
            HealthAction::Heal(mut restoration) if restoration > 0 => {
                let mut result = vec![];
                let mut first = 0;
                loop {
                    let reaction = self.hit_dice[first].execute(HealthAction::Heal(restoration));
                    result.push((&reaction, first).into());
                    if let HitDieActionResponse::HealResponse(rest) = reaction {
                        if rest > 0 && first < self.hit_dice.len() - 1 {
                            restoration = rest;
                            first += 1;
                        } else {
                            break;
                        }
                    }
                }

                result
            },
            HealthAction::Drain => {
                let mut should_apply = true;
                let mut last = self.hit_dice.len() - 1;
                loop {
                    if self.hit_dice[last].statuses.contains(&HitDieStatus::Empty) {
                        if last > 0 {
                            last -= 1;
                        } else {
                            should_apply = false;
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if should_apply {
                    let response = self.hit_dice[last].execute(HealthAction::Drain);
                    vec![ (&response, last).into() ]
                } else {
                    vec![ HealthActionResponse::None ]
                }
            },
            HealthAction::Break => {
                let mut last = self.hit_dice.len() - 1;
                
                let mut should_apply = true;
                loop {
                    if self.hit_dice[last].statuses.contains(&HitDieStatus::Void) {
                        if last > 0 {
                            last -= 1;
                        } else {
                            should_apply = false;
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if should_apply {
                    let response = self.hit_dice[last].execute(HealthAction::Break);
                    vec![ (&response, last).into() ]
                } else {
                    vec![ HealthActionResponse::None ]
                }
            },
            HealthAction::Fortify(n) => {
                let last = self.hit_dice.len() - 1;
                vec![ (&self.hit_dice[last].execute(HealthAction::Fortify(n)), last).into() ]
            },
            HealthAction::Mend => {
                let mut first = 0;
                
                let mut should_apply = true;
                loop {
                    if !self.hit_dice[first].statuses.contains(&HitDieStatus::Void) {
                        if first < self.hit_dice.len() - 1 {
                            first += 1;
                        } else {
                            should_apply = false;
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if should_apply {
                    self.hit_dice[first].execute(HealthAction::Mend);
                    vec![ HealthActionResponse::MendResponse(first) ]
                } else {
                    vec![ HealthActionResponse::None ]
                }
            },
            HealthAction::Shatter => {
                if let Some(_) = self.hit_dice.pop() {
                    vec![ HealthActionResponse::ShatterResponse(self.hit_dice.len()) ]
                } else {
                    vec![ HealthActionResponse::None ]
                }
            },
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod health_testing {
    use crate::gameplay::{health::{HealthActionResponse, HitDieStatus}, React};

    use super::Health;

    #[test]
    fn test_creating_hit_dice() {

        // 0

        let mut health = Health::default();
        let result = health.execute(super::HealthAction::Create(6));
        assert_eq!(result, vec![ HealthActionResponse::CreateResponse(0) ]);

        // [......] 6

        let result = health.execute(super::HealthAction::Create(2));
        assert_eq!(result, vec![ HealthActionResponse::CreateResponse(1) ]);

        // [......][..] 6+2
    }

    #[test]
    fn test_chip_hit_dice() {

        // [......][......] 6+6

        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));
        let _ = health.execute(super::HealthAction::Create(6));
        let result = health.execute(super::HealthAction::Chip(4));
        
        // [......][......] 6+6    -CHIP 4->    [......][..xxxx] 6+2 / spill 0 damage

        assert_eq!(result, vec![ HealthActionResponse::ChipResponse(1, 0) ]);
    }

    #[test]
    fn test_chip_spill_hit_dice() {
        
        // [......][......] 6+6

        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));
        let _ = health.execute(super::HealthAction::Create(6));
        let result = health.execute(super::HealthAction::Chip(10));
        
        // [......][......] 6+6   -CHIP 10->   [......][xxxxxx] / spill 4 damage   -CHIP 4->   [..xxxx][xxxxxx] 2+0 / spill 0 damage

        assert_eq!(result, vec![ HealthActionResponse::ChipResponse(1, 4), HealthActionResponse::ChipResponse(0, 0) ]);
    }

    #[test]
    fn test_chip_spill_empty_hit_dice() {

        // [......] 6

        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));
        let result = health.execute(super::HealthAction::Chip(10));
        
        // [......] 6   -CHIP 10->    [xxxxxx] 0 / spill 4 damage   --no remaining dice, spill ends

        assert_eq!(result, vec![ HealthActionResponse::ChipResponse(0, 4) ]);
    }

    #[test]
    fn test_temporary_hit_dice_shatter() {
        
        // [......] 6
        //  |
        //  Temporary

        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));
        let _ = health.execute(super::HealthAction::AddStatus(HitDieStatus::Temporary));
        assert!(health.hit_dice.last().unwrap().statuses.contains(&HitDieStatus::Temporary));
        
        // [......] 6   -CHIP 6->   [xxxxxx]    -TEMP->    gone!
        //  |                        |
        //  Temporary                Temporary, Empty

        let result = health.execute(super::HealthAction::Chip(6));
        assert_eq!(result, vec![ HealthActionResponse::ShatterResponse(0) ]);
        assert_eq!(0, health.hit_dice.len());
    }

    #[test]
    fn test_fortify_hit_dice() {

        // [......] 6

        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));
        let result = health.execute(super::HealthAction::Fortify(1));
        assert_eq!(result, vec![ HealthActionResponse::FortifiedResponse(0, 1)]);

        // [......] 6
        //  |
        //  Fortified(1)
    }

    #[test]
    fn test_fortify_stacks_hit_dice() {

        // [......] 6

        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));

        // [......] 6   -FORT 1->    [......] 6
        //                            |
        //                            Fortified(1)

        let result = health.execute(super::HealthAction::Fortify(1));
        assert_eq!(result, vec![ HealthActionResponse::FortifiedResponse(0, 1)]);

        // [......] 6   -FORT 3->    [......] 6
        //  |                         |
        //  Fortified 1               Fortified(4)

        let result = health.execute(super::HealthAction::Fortify(3));
        assert_eq!(result, vec![ HealthActionResponse::FortifiedResponse(0, 4)]);
    }

    #[test]
    fn test_chip_fortified_hit_dice() {

        // [......][......] 6+6
        //          |
        //          Fortified(1)

        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));
        let _ = health.execute(super::HealthAction::Create(6));
        let _ = health.execute(super::HealthAction::Fortify(1));
        assert!(health.hit_dice.last().unwrap().statuses.iter().any(|s| matches!(s, HitDieStatus::Fortified(_))));

        // [......][......] 6+6   -CHIP 4->   [......][..xxxx] 6+2
        //          |                                  |
        //          Fortified(1)                       Fortified(1)

        let result = health.execute(super::HealthAction::Chip(4));
        assert_eq!(result, vec![ HealthActionResponse::ChipResponse(1, 0) ]);
        assert!(health.hit_dice.last().unwrap().statuses.iter().any(|s| matches!(s, HitDieStatus::Fortified(_))));

        // [......][..xxxx] 6+6   -CHIP 4->   [......][xxxxxx] / spill 2 damage   -FORT 1->    [......][......] 6+6 / no spill!
        //          |                                  |
        //          Fortified(1)                       Empty, Fortified(1)

        let result = health.execute(super::HealthAction::Chip(4));
        assert_eq!(result, vec![ HealthActionResponse::RecoveryResponse(1) ]);
        assert!(!health.hit_dice.last().unwrap().statuses.iter().any(|s| matches!(s, HitDieStatus::Fortified(_))));
    }

    #[test]
    fn test_chip_spill_fortified_hit_dice() {
        
        // [......][......] 6+6
        //          |
        //          Fortified(1)

        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));
        let _ = health.execute(super::HealthAction::Create(6));
        let _ = health.execute(super::HealthAction::Fortify(1));
        assert!(health.hit_dice.last().unwrap().statuses.iter().any(|s| matches!(s, HitDieStatus::Fortified(_))));

        // [......][......] 6+6   -CHIP 10->   [......][xxxxxx] / spill 4 damage   -FORT 1->   [......][......] 6+6
        //          |                                   |
        //          Fortified(1)                        Empty, Fortified(1)

        let result = health.execute(super::HealthAction::Chip(10));
        assert_eq!(result, vec![ HealthActionResponse::RecoveryResponse(1) ]);
        assert!(!health.hit_dice.last().unwrap().statuses.iter().any(|s| matches!(s, HitDieStatus::Fortified(_))));

        // [......][......] 6+6   -CHIP 10->   [......][xxxxxx] / spill 4 damage   ->    [..xxxx][xxxxxx] 2+0
        //                                              |                                         |
        //                                              Empty                                     Empty

        let result = health.execute(super::HealthAction::Chip(10));
        assert_eq!(result, vec![ HealthActionResponse::ChipResponse(1, 4), HealthActionResponse::ChipResponse(0, 0) ]);
    }

    #[test]
    fn test_chip_spill_deep_fortified_hit_dice() {
        let mut health = Health::default();
        
        //   [......][......] 6+6
        //    |
        //    Fortified(1)

        let _ = health.execute(super::HealthAction::Create(6));
        let _ = health.execute(super::HealthAction::Fortify(1));
        let _ = health.execute(super::HealthAction::Create(6));
        assert!(health.hit_dice.first().unwrap().statuses.iter().any(|s| matches!(s, HitDieStatus::Fortified(_))));
        assert!(!health.hit_dice.last().unwrap().statuses.iter().any(|s| matches!(s, HitDieStatus::Fortified(_))));

        // 13 damage is overwhelming, so let's go step by step as to how it will resolve:
        
        //   [......][......] 6+6   -CHIP 13->   [......][xxxxxx] / spill 7 damage   -CHIP 7->   [xxxxxx][xxxxxx] / spill 1 damage
        //    |                                   |       |                                       |       |
        //    Fortified(1)                        |       Empty                                   |       Empty
        //                                        |                                               |
        //                                        Fortified(1)                                    Fortified(1), Empty

        // now that all the damage is spilled as much as it could, Fortification starts its thing:
        
        //   [xxxxxx][xxxxxx]    -FORT 1->    [......][xxxxxx]
        //    |       |                                |
        //    |       Empty                            Empty
        //    |
        //    Fortified(1), Empty

        // you could imagine it shortened to:

        //       [......][......] 6+6    -->    [xxxxxx][xxxxxx]    -FORT 1->    [......][xxxxxx] 6
        //        |                              |       |                                |
        //        Fortified(1)                   |       Empty                            Empty
        //                                       |
        //                                       Empty, Fortified(1)

        
        let result = health.execute(super::HealthAction::Chip(13));
        assert_eq!(result, vec![ HealthActionResponse::ChipResponse(1, 7), HealthActionResponse::RecoveryResponse(0) ]);
        assert!(!health.hit_dice.first().unwrap().statuses.iter().any(|s| matches!(s, HitDieStatus::Fortified(_))));
        assert!(!health.hit_dice.last().unwrap().statuses.iter().any(|s| matches!(s, HitDieStatus::Fortified(_))));
        assert!(health.hit_dice.last().unwrap().statuses.iter().any(|s| matches!(s, HitDieStatus::Empty)));
    }

    #[test]
    fn test_fortified_over_multiple_attacks() {
        
        // [......] 6
        //  |
        //  Fortified(3)

        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));
        let _ = health.execute(super::HealthAction::AddStatus(HitDieStatus::Fortified(3)));
        assert!(health.hit_dice.last().unwrap().statuses.contains(&HitDieStatus::Fortified(3)));

        // [......] 6   -CHIP 6->   [xxxxxx]    -FORT 3->    [......] 6
        //  |                        |                        |
        //  Fortified(3)             Fortified(3), Empty      Fortified(2) / change: 3 -> 2

        let result = health.execute(super::HealthAction::Chip(6));
        assert_eq!(result, vec![ HealthActionResponse::RecoveryResponse(0) ]);
        assert!(health.hit_dice.last().unwrap().statuses.contains(&HitDieStatus::Fortified(2)));

        // [......] 6   -CHIP 6->   [xxxxxx]    -FORT 2->    [......] 6
        //  |                        |                        |
        //  Fortified(2)             Fortified(2), Empty      Fortified(1) / change: 2 -> 1

        let result = health.execute(super::HealthAction::Chip(6));
        assert_eq!(result, vec![ HealthActionResponse::RecoveryResponse(0) ]);
        assert!(health.hit_dice.last().unwrap().statuses.contains(&HitDieStatus::Fortified(1)));


        // [......] 6   -CHIP 6->   [xxxxxx]    -FORT 1->    [......] 6
        //  |                        |
        //  Fortified(1)             Fortified(1), Empty

        let result = health.execute(super::HealthAction::Chip(6));
        assert_eq!(result, vec![ HealthActionResponse::RecoveryResponse(0) ]);
        assert!(health.hit_dice.last().unwrap().statuses.is_empty());

        // [......] 6   -CHIP 6->   [xxxxxx]
        //                           |
        //                           Empty

        let result = health.execute(super::HealthAction::Chip(6));
        assert_eq!(result, vec![ HealthActionResponse::ChipResponse(0, 0) ]);
        assert!(health.hit_dice.last().unwrap().statuses.contains(&HitDieStatus::Empty));
    }

    #[test]
    fn test_fortified_activates_before_temporary() {
        
        // [......] 6
        //  |
        //  Temporary, Fortified(1)

        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));
        let _ = health.execute(super::HealthAction::AddStatus(HitDieStatus::Temporary));
        let _ = health.execute(super::HealthAction::AddStatus(HitDieStatus::Fortified(1)));
        assert!(health.hit_dice.last().unwrap().statuses.contains(&HitDieStatus::Temporary));
        assert!(health.hit_dice.last().unwrap().statuses.contains(&HitDieStatus::Fortified(1)));
        
        // [......] 6   -CHIP 6->   [xxxxxx]    -FORT 1->    [......]
        //  |                        |                        |
        //  Temporary, Fortified(1)  Temporary, Fortified(1)  Temporary

        let result = health.execute(super::HealthAction::Chip(6));
        assert_eq!(result, vec![ HealthActionResponse::RecoveryResponse(0) ]);
        assert_eq!(1, health.hit_dice.len());
        assert_eq!(6, *health.hit_dice.last().unwrap().value);
        assert!(!health.hit_dice.last().unwrap().statuses.contains(&HitDieStatus::Fortified(1)));

        // [......] 6   -CHIP 6->   [xxxxxx]    -TEMP->    gone!
        //  |                        |
        //  Temporary                Temporary, Empty

        let result = health.execute(super::HealthAction::Chip(6));
        assert_eq!(result, vec![ HealthActionResponse::ShatterResponse(0) ]);
        assert_eq!(0, health.hit_dice.len());
    }

    #[test]
    pub fn test_drain_hit_dice() {
                
        //   [......] 6
        
        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));
       
        //   [......] 6    -DRAIN->   [xxxxxx] 0
        //                             |
        //                             Empty

        let result = health.execute(super::HealthAction::Drain);
        assert_eq!(result, vec![ HealthActionResponse::DrainResponse(0) ]);
        assert_eq!(*health.hit_dice.last().unwrap().value, 0);
        assert!(health.hit_dice.last().unwrap().statuses.contains(&HitDieStatus::Empty));
    }

    #[test]
    pub fn test_guard_cancels_drain_hit_dice() {

        // [......] 6
        //  |
        //  Guarded

        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));
        let _ = health.execute(super::HealthAction::AddStatus(HitDieStatus::Guarded));

        // [......] 6   -DRAIN->   [......] 6 / no change!
        //  |                       |
        //  Guarded                 Guarded

        let result = health.execute(super::HealthAction::Drain);
        assert_eq!(result, vec![ HealthActionResponse::RecoveryResponse(0) ]);
    }

    #[test]
    pub fn test_drain_fortified_hit_dice() {
        
        //   [......] 6

        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));
        let _ = health.execute(super::HealthAction::Fortify(1));

        //   [......] 6    -DRAIN->    [xxxxxx]    -FORT 1->    [......] 6
        //    |                         |
        //    Fortified(1)              Empty, Fortified(1)

        let result = health.execute(super::HealthAction::Drain);
        assert_eq!(result, vec![ HealthActionResponse::RecoveryResponse(0) ]);
        assert_eq!(*health.hit_dice.last().unwrap().value, 6);
        assert!(!health.hit_dice.last().unwrap().statuses.iter().any(|s| matches!(s, HitDieStatus::Fortified(_))));
    }

    #[test]
    pub fn test_break_hit_dice() {
        
        //   [......][......] 6+6
        
        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));
        let _ = health.execute(super::HealthAction::Create(6));
        assert_eq!(health.hit_dice.len(), 2);

        //   [......][......] 6+6    -BREAK->   [......][......] 6+6
        //                                               |
        //                                               Void

        let result = health.execute(super::HealthAction::Break);
        assert_eq!(result, vec![ HealthActionResponse::BreakResponse(1) ]);

        //   [......][......] 6+6    -BREAK->   [......][......] 6+6
        //            |                          |       |
        //            Void                       Void    Void

        let result = health.execute(super::HealthAction::Break);
        assert_eq!(result, vec![ HealthActionResponse::BreakResponse(0) ]);
    }

    #[test]
    pub fn test_guard_cancels_break_hit_dice() {

        // [......] 6
        //  |
        //  Guarded

        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));
        let _ = health.execute(super::HealthAction::AddStatus(HitDieStatus::Guarded));

        // [......] 6   -BREAK->   [......] 6
        //  |                       |
        //  Guarded                 Guarded / no change - not Void!

        let result = health.execute(super::HealthAction::Break);
        assert_eq!(result, vec![ HealthActionResponse::RecoveryResponse(0) ]);
    }

    #[test]
    pub fn test_break_void_hit_dice_has_no_effect() {

        // [......] 6

        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));        
        assert_eq!(health.hit_dice.len(), 1);

        //   [......] 6    -BREAK->   [......] 6
        //                             |
        //                             Void

        let result = health.execute(super::HealthAction::Break);
        assert_eq!(result, vec![ HealthActionResponse::BreakResponse(0) ]);

        //   [......] 6    -BREAK->   [......] 6 / no change
        //    |                        |
        //    Void                     Void

        let result = health.execute(super::HealthAction::Break);
        assert_eq!(result, vec![ HealthActionResponse::None ]);
    }

    #[test]
    pub fn test_mend_hit_dice() {

        // [......] 6
        //  |
        //  Void

        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));        
        assert_eq!(health.hit_dice.len(), 1);
        let _ = health.execute(super::HealthAction::Break);
        assert!(health.hit_dice.last().unwrap().statuses.contains(&HitDieStatus::Void));

        // [......] 6   -MEND->    [......] 6
        //  |
        //  Void
        
        let result = health.execute(super::HealthAction::Mend);
        assert_eq!(result, vec![ HealthActionResponse::MendResponse(0) ]);
        assert!(!health.hit_dice.last().unwrap().statuses.contains(&HitDieStatus::Void));
    }

    #[test]
    pub fn test_shatter_hit_dice() {
        
        // [......] 6

        let mut health = Health::default();
        let _ = health.execute(super::HealthAction::Create(6));        
        assert_eq!(health.hit_dice.len(), 1);

        // [......] 6   -SHATTER->   gone!

        let result = health.execute(super::HealthAction::Shatter);
        assert_eq!(result, vec![ HealthActionResponse::ShatterResponse(0) ]);
        assert!(health.hit_dice.last().is_none());
    }

    #[test]
    pub fn test_shatter_no_hit_dice() {

        //  nothing

        let mut health = Health::default();
        assert!(health.hit_dice.last().is_none());

        //  nothing   -SHATTER->   nothing

        let result = health.execute(super::HealthAction::Shatter);
        assert_eq!(result, vec![ HealthActionResponse::None ]);
        assert!(health.hit_dice.last().is_none());
    }
}
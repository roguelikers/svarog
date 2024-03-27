use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum SvarogGameplaySet {
    Movement,
    Physics,
    Combat,
    Other,
}

pub struct SvarogConfigPlugin;

impl Plugin for SvarogConfigPlugin {
    fn build(&self, app: &mut App) {
        use SvarogGameplaySet::*;
        app.configure_sets(
            Update,
            (
                Physics.after(Movement),
                Combat.after(Physics),
                Other.after(Combat),
            ),
        );
    }
}

pub mod health;
pub mod value;

pub type Time = i32;
pub type Amount = i32;
pub type Index = usize;
pub type Swap = bool;

pub trait React<Action, ActionResponse> {
    fn execute(&mut self, action: Action) -> ActionResponse;
}


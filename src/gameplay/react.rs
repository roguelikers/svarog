
pub trait React<Action, ActionResponse> {
    fn execute(&mut self, action: Action) -> ActionResponse;
}
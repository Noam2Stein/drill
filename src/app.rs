use lib_app::{AppContext, AppHandler};

#[derive(Debug)]
pub struct Game {}

impl AppHandler for Game {
    const TITLE: &str = "Drill";

    fn new(_ctx: AppContext<'_>) -> Self {
        Self {}
    }
}

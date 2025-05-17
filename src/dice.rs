use crate::error::Result;
use caith::{RollResultType, Roller};

byond_fn!(fn roll_dice(dice) {
    match roll(dice) {
        Ok(result) => return Some(result),
        Err(error) => return Some(error.to_string())
    }
});

fn roll(input: &str) -> Result<String> {
    let dice = Roller::new(input)?;
    let result = dice.roll()?;

    match result.get_result() {
        RollResultType::Single(res_single) => Ok(res_single.get_total().to_string()),
        RollResultType::Repeated(res_repeated) => Ok(res_repeated.get_total().unwrap().to_string()),
    }
}

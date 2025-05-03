/*
 *
 * Syntax Guide: https://docs.rs/caith/latest/caith/#syntax
 * Examples: https://docs.rs/caith/latest/caith/#examples
 *
 * Args:
 * * input: the xdy dice to roll; see syntax guide & examples for proper formatting.
 *
 * Returns:
 * * the total sum of the roll as a string.
 */
#define rustg_roll_dice(input) RUSTG_CALL(RUST_G, "roll_dice")("[input]")

#ifdef RUSTG_OVERRIDE_BUILTINS
	#define roll(dice) rustg_roll_dice(dice)
#endif

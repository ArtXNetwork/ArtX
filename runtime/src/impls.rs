//! Some configurable implementations as associated type for the substrate runtime.

use sp_runtime::traits::{Convert};
use super::Balance;

/// Handles converting a scalar to convert balance
///
pub struct ConvertBalance;
impl Convert<Balance, Balance> for ConvertBalance {
	fn convert(x: Balance) -> Balance {
		x.into()
	}
}

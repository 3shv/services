//! Module emulating some of the functions in the Balancer WeightedMath.sol
//! smart contract. The original contract code can be found at:
//! https://github.com/balancer-labs/balancer-v2-monorepo/blob/6c9e24e22d0c46cca6dd15861d3d33da61a60b98/pkg/core/contracts/pools/weighted/WeightedMath.sol

#![allow(dead_code)]

use super::error::Error;
use super::fixed_point::Bfp;
use ethcontract::U256;
use lazy_static::lazy_static;

// https://github.com/balancer-labs/balancer-v2-monorepo/blob/6c9e24e22d0c46cca6dd15861d3d33da61a60b98/pkg/core/contracts/pools/weighted/WeightedMath.sol#L36-L37
lazy_static! {
    static ref MAX_IN_RATIO: Bfp =
        Bfp::from_wei(U256::exp10(17).checked_mul(3_u32.into()).unwrap());
    static ref MAX_OUT_RATIO: Bfp =
        Bfp::from_wei(U256::exp10(17).checked_mul(3_u32.into()).unwrap());
}

// https://github.com/balancer-labs/balancer-v2-monorepo/blob/6c9e24e22d0c46cca6dd15861d3d33da61a60b98/pkg/core/contracts/pools/weighted/WeightedMath.sol#L69-L100
pub fn calc_out_given_in(
    balance_in: Bfp,
    weight_in: Bfp,
    balance_out: Bfp,
    weight_out: Bfp,
    amount_in: Bfp,
) -> Result<Bfp, Error> {
    if amount_in > balance_in.mul_down(*MAX_IN_RATIO)? {
        return Err(Error::MaxInRatio);
    }

    let denominator = balance_in.add(amount_in)?;
    let base = balance_in.div_up(denominator)?;
    let exponent = weight_in.div_down(weight_out)?;
    let power = base.pow_up(exponent)?;

    balance_out.mul_down(power.complement())
}

// https://github.com/balancer-labs/balancer-v2-monorepo/blob/6c9e24e22d0c46cca6dd15861d3d33da61a60b98/pkg/core/contracts/pools/weighted/WeightedMath.sol#L104-L138
pub fn calc_in_given_out(
    balance_in: Bfp,
    weight_in: Bfp,
    balance_out: Bfp,
    weight_out: Bfp,
    amount_out: Bfp,
) -> Result<Bfp, Error> {
    if amount_out > balance_out.mul_down(*MAX_OUT_RATIO)? {
        return Err(Error::MaxOutRatio);
    }

    let base = balance_out.div_up(balance_out.sub(amount_out)?)?;
    let exponent = weight_out.div_up(weight_in)?;
    let power = base.pow_up(exponent)?;

    let ratio = power.sub(Bfp::one())?;
    balance_in.mul_up(ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The expected output for the tested functions was generated by running the
    // following instructions after cloning and installing the repo at
    // github.com/balancer-labs/balancer-v2-monorepo, commit
    // 6c9e24e22d0c46cca6dd15861d3d33da61a60b98:
    // ```
    // $ cd pkg/core/
    // $ cp contracts/pools/weighted/WeightedMath.sol contracts/pools/weighted/WeightedMathTest.sol
    // $ sed --in-place -E 's/contract WeightedMath/contract WeightedMathTest/' contracts/pools/weighted/WeightedMathTest.sol
    // $ sed --in-place -E 's/(private|internal)/public/' contracts/pools/weighted/WeightedMathTest.sol
    // $ yarn hardhat console
    // > const weightedMath = await (await ethers.getContractFactory("WeightedMathTest")).deploy()
    // ```
    // Every test specifies a command that should be pasted into the console to
    // obtain the expected output.

    #[test]
    fn calc_out_given_in_test() {
        assert_eq!(
            calc_out_given_in(
                Bfp::from_wei(100_000_000_000_000_000_000_000_u128.into()),
                Bfp::from_wei(300_000_000_000_000_u128.into()),
                Bfp::from_wei(10_000_000_000_000_000_000_u128.into()),
                Bfp::from_wei(700_000_000_000_000_u128.into()),
                Bfp::from_wei(10_000_000_000_000_000_u128.into()),
            )
            .unwrap(),
            // (await weightedMath["_calcOutGivenIn"]("100000000000000000000000", "300000000000000", "10000000000000000000", "700000000000000", "10000000000000000")).toString()
            Bfp::from_wei(428_571_297_950_u128.into()),
        );
    }

    #[test]
    fn calc_in_given_out_test() {
        assert_eq!(
            calc_in_given_out(
                Bfp::from_wei(100_000_000_000_000_000_000_000_u128.into()),
                Bfp::from_wei(300_000_000_000_000_u128.into()),
                Bfp::from_wei(10_000_000_000_000_000_000_u128.into()),
                Bfp::from_wei(700_000_000_000_000_u128.into()),
                Bfp::from_wei(10_000_000_000_000_000_u128.into()),
            )
            .unwrap(),
            // (await weightedMath["_calcInGivenOut"]("100000000000000000000000", "300000000000000", "10000000000000000000", "700000000000000", "10000000000000000")).toString()
            Bfp::from_wei(233_722_784_701_541_000_000_u128.into()),
        );
    }

    #[test]
    fn stops_large_trades() {
        let deposit_in = 20_000_000_000_000_000_000_000_u128;
        let deposit_out = 10_000_000_000_000_000_000_000_u128;
        macro_rules! calc_with_default_pool {
            ($fn_name:ident, $amount: expr) => {
                $fn_name(
                    Bfp::from_wei(deposit_in.into()),
                    Bfp::from_wei(500_000_000_000_000_u128.into()),
                    Bfp::from_wei(deposit_out.into()),
                    Bfp::from_wei(500_000_000_000_000_u128.into()),
                    Bfp::from_wei($amount),
                )
            };
        }
        // Test case output generating function:
        // > const calc_with_default_pool = async (fn_name, amount) => (await weightedMath[fn_name]("20000000000000000000000", "500000000000000", "10000000000000000000000", "500000000000000", amount)).toString()

        let largest_amount_in = deposit_in * 3 / 10;
        assert_eq!(
            calc_with_default_pool!(calc_out_given_in, largest_amount_in.into()).unwrap(),
            // > await calc_with_default_pool("_calcOutGivenIn", "6000000000000000000000")
            Bfp::from_wei(2_307_692_307_692_230_750_000_u128.into())
        );
        assert_eq!(
            calc_with_default_pool!(calc_out_given_in, (largest_amount_in + 1).into()).unwrap_err(),
            // > await calc_with_default_pool("_calcOutGivenIn", "6000000000000000000001")
            "304".into()
        );
        let largest_amount_out = deposit_out * 3 / 10;
        assert_eq!(
            calc_with_default_pool!(calc_in_given_out, largest_amount_out.into()).unwrap(),
            // > await calc_with_default_pool("_calcInGivenOut", "3000000000000000000000")
            Bfp::from_wei(8_571_428_571_428_857_160_000_u128.into())
        );
        assert_eq!(
            calc_with_default_pool!(calc_in_given_out, (largest_amount_out + 1).into())
                .unwrap_err(),
            // > await calc_with_default_pool("_calcInGivenOut", "3000000000000000000001")
            "305".into()
        );
    }
}

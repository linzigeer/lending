use crate::share_op::ShareOp;
use std::f64::consts::E;
use anchor_lang::prelude::{Clock, SolanaSysvar};
///保留n位小数点
pub fn round_to_n_decimal(value: f64, n_decimals: u8) -> f64 {
    let base = 10.0f64.powf(n_decimals as f64);
    (value * base).round() / base
}

///计算本次改变的份额计算
pub fn calc_change_shares(current_change_value: u64, total_value: u64, total_shares: f64, n_decimals: u8) -> f64 {
    round_to_n_decimal(
        (current_change_value as f64 / total_value as f64) * total_shares,
        n_decimals,
    )
}

///计算本次改变后的总份额
pub fn calc_new_total_shares(
    current_change_value: u64,
    total_value: u64,
    total_shares: f64,
    n_decimals: u8,
    share_op: ShareOp,
) -> f64 {
    let user_new_shares = calc_change_shares(current_change_value, total_value, total_shares, n_decimals);

    match share_op {
        ShareOp::Increase => total_shares + user_new_shares,
        ShareOp::Decrease => total_shares - user_new_shares,
    }
}

///利息应得计算
pub fn calc_accrued_interest(base: u64, interest: f64, last_update: i64, n_decimals: u8) -> f64 {
    let current_time = Clock::get().unwrap().unix_timestamp;
    let lasts_time = current_time - last_update;
    round_to_n_decimal(
        base as f64 * E.powf(interest * lasts_time as f64),
        n_decimals,
    )
}

///计算本金+利息
pub fn calc_base_sum_interest(base: u64, interest: f64, last_update: i64, n_decimals: u8) -> f64 {
    let interest = calc_accrued_interest(base, interest, last_update, n_decimals);

    base as f64 + interest
}

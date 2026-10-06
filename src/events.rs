use soroban_sdk::{symbol_short, Address, Env, String};

pub struct Events;

impl Events {
    pub fn emit_init(env: &Env, admin: &Address, treasury: &Address, operator: &Address) {
        let topics = (symbol_short!("init"), admin.clone());
        env.events().publish(topics, (treasury.clone(), operator.clone()));
    }

    pub fn emit_operator_updated(env: &Env, new_operator: &Address) {
        let topics = (symbol_short!("oper_upd"), new_operator.clone());
        env.events().publish(topics, ());
    }

    pub fn emit_treasury_updated(env: &Env, new_treasury: &Address) {
        let topics = (symbol_short!("tres_upd"), new_treasury.clone());
        env.events().publish(topics, ());
    }

    pub fn emit_pause_toggled(env: &Env, paused: bool) {
        let topics = (symbol_short!("pause"),);
        env.events().publish(topics, paused);
    }

    pub fn emit_plan_updated(
        env: &Env,
        plan_id: u32,
        duration_secs: u64,
        credits: u32,
        unlimited: bool,
        active: bool,
    ) {
        let topics = (symbol_short!("plan_upd"), plan_id);
        env.events().publish(topics, (duration_secs, credits, unlimited, active));
    }

    pub fn emit_plan_price_set(env: &Env, plan_id: u32, token: &Address, amount: i128) {
        let topics = (symbol_short!("price_set"), plan_id, token.clone());
        env.events().publish(topics, amount);
    }

    pub fn emit_credit_price_set(env: &Env, token: &Address, amount: i128) {
        let topics = (symbol_short!("c_price"), token.clone());
        env.events().publish(topics, amount);
    }

    pub fn emit_subscribed(
        env: &Env,
        user: &Address,
        plan_id: u32,
        token: &Address,
        amount: i128,
        expires_at: u64,
    ) {
        let topics = (symbol_short!("sub"), user.clone(), plan_id);
        env.events().publish(topics, (token.clone(), amount, expires_at));
    }

    pub fn emit_credits_purchased(
        env: &Env,
        user: &Address,
        token: &Address,
        count: u32,
        amount: i128,
    ) {
        let topics = (symbol_short!("buy_cred"), user.clone());
        env.events().publish(topics, (token.clone(), count, amount));
    }

    pub fn emit_credit_consumed(env: &Env, user: &Address, job_id: &String) {
        let topics = (symbol_short!("use_cred"), user.clone());
        env.events().publish(topics, job_id.clone());
    }

    pub fn emit_refunded(env: &Env, to: &Address, token: &Address, amount: i128) {
        let topics = (symbol_short!("refund"), to.clone(), token.clone());
        env.events().publish(topics, amount);
    }
}

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, Env, String,
};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Plan(u64),
    Subscription(Address),
    PlanCounter,
}

#[derive(Clone)]
#[contracttype]
pub struct SubscriptionPlan {
    pub id: u64,
    pub name: String,
    pub price: i128,
    pub duration: u64,
    pub active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct UserSubscription {
    pub user: Address,
    pub plan_id: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub auto_renew: bool,
    pub active: bool,
    pub paused: bool,
}

#[contract]
pub struct ChainSubscriptionHub;

#[contractimpl]
impl ChainSubscriptionHub {

    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::PlanCounter, &0u64);
    }

    fn require_admin(env: &Env) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap();

        admin.require_auth();
    }

    pub fn create_plan(
        env: Env,
        name: String,
        price: i128,
        duration: u64,
    ) -> u64 {
        Self::require_admin(&env);

        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PlanCounter)
            .unwrap_or(0);

        counter += 1;

        let plan = SubscriptionPlan {
            id: counter,
            name,
            price,
            duration,
            active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Plan(counter), &plan);

        env.storage()
            .instance()
            .set(&DataKey::PlanCounter, &counter);

        counter
    }

    pub fn subscribe(
        env: Env,
        user: Address,
        plan_id: u64,
        auto_renew: bool,
    ) {
        user.require_auth();

        let plan: SubscriptionPlan = env
            .storage()
            .persistent()
            .get(&DataKey::Plan(plan_id))
            .unwrap();

        if !plan.active {
            panic!("plan inactive");
        }

        let now = env.ledger().timestamp();

        let subscription = UserSubscription {
            user: user.clone(),
            plan_id,
            start_time: now,
            end_time: now + plan.duration,
            auto_renew,
            active: true,
            paused: false,
        };

        env.storage()
            .persistent()
            .set(
                &DataKey::Subscription(user),
                &subscription,
            );
    }

    pub fn cancel_subscription(
        env: Env,
        user: Address,
    ) {
        user.require_auth();

        let mut sub: UserSubscription = env
            .storage()
            .persistent()
            .get(
                &DataKey::Subscription(user.clone()),
            )
            .unwrap();

        sub.active = false;
        sub.auto_renew = false;

        env.storage()
            .persistent()
            .set(
                &DataKey::Subscription(user),
                &sub,
            );
    }

    pub fn get_subscription(
        env: Env,
        user: Address,
    ) -> UserSubscription {
        env.storage()
            .persistent()
            .get(
                &DataKey::Subscription(user),
            )
            .unwrap()
    }

    pub fn total_plans(
        env: Env,
    ) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::PlanCounter)
            .unwrap_or(0)
    }
}
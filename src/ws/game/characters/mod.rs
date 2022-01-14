use std::{collections::HashMap, fs::read_to_string};

use actix::Addr;
use rand::{prelude::SliceRandom, Rng};
use serde::{Deserialize, Serialize};

use crate::ws::ChatServer;

use self::player::Player;

pub mod bettrayer;
pub mod cupid;
pub mod fox;
pub mod guard;
pub mod lycan;
pub mod player;
pub mod seer;
pub mod superwolf;
pub mod villager;
pub mod werewolf;
pub mod witch;

pub mod roles {
    pub const VILLAGER: &'static str = "Villager";
    pub const WEREWOLF: &'static str = "Werewolf";
    pub const SUPERWOLF: &'static str = "Superwolf";
    pub const SEER: &'static str = "Seer";
    pub const GUARD: &'static str = "Guard";
    pub const LYCAN: &'static str = "Lycan";
    pub const FOX: &'static str = "Fox";
    pub const WITCH: &'static str = "Witch";
    pub const CUPID: &'static str = "Cupid";
    pub const BETRAYER: &'static str = "Betrayer";
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum FRR {
    Fixed(usize),
    Range(usize, usize),
    Rate(f32, usize),
}

pub type RoleConfig<'a> = HashMap<usize, HashMap<&'a str, FRR>>;

pub fn rand_roles(
    uids: &Vec<&i64>,
    addr: Addr<ChatServer>,
) -> Result<HashMap<i64, Box<dyn Player>>, String> {
    let json = read_to_string("./jsons/role-config.json")
        .map_err(|err| err.to_string())?;
    let config = serde_json::from_str::<RoleConfig>(&json).unwrap();

    let roles = config.get(&uids.len()).unwrap();

    let mut num = uids.len();
    let mut rls = HashMap::new();
    for (&role, frr) in roles {
        if let FRR::Fixed(n) = frr {
            rls.insert(role, *n);
            num -= n;
        }
    }
    for (&role, frr) in roles {
        if let FRR::Range(a, b) = frr {
            let r = rand::thread_rng().gen_range(*a..(*b + 1));
            if num < r {
                continue;
            }
            rls.insert(role, r);
            num -= r;
        }
    }
    'outer: loop {
        for (&role, frr) in roles {
            if num == 0 {
                break 'outer;
            }
            if let FRR::Rate(rate, max) = frr {
                if rand::thread_rng().gen::<f32>() < *rate {
                    let r = rls.entry(role).or_default();
                    if *r >= *max {
                        continue;
                    }
                    *r += 1;
                    num -= 1;
                }
            }
        }
    }

    let mut uids = uids.clone();
    uids.shuffle(&mut rand::thread_rng());

    let mut rs = HashMap::new();
    for (&role, &num) in rls.iter() {
        for _ in 0..num {
            let &id = uids.pop().ok_or("pop false".to_string())?;
            let role = new_role(role, id, addr.clone())?;
            rs.insert(id, role);
        }
    }

    Ok(rs)
}

fn new_role(
    role: &str,
    id: i64,
    addr: Addr<ChatServer>,
) -> Result<Box<dyn Player>, String> {
    match role {
        roles::VILLAGER => Ok(Box::new(villager::Villager::new(id, addr))),
        roles::WEREWOLF => Ok(Box::new(werewolf::Werewolf::new(id, addr))),
        roles::SUPERWOLF => Ok(Box::new(superwolf::Superwolf::new(id, addr))),
        roles::SEER => Ok(Box::new(seer::Seer::new(id, addr))),
        roles::GUARD => Ok(Box::new(guard::Guard::new(id, addr))),
        roles::LYCAN => Ok(Box::new(lycan::Lycan::new(id, addr))),
        roles::FOX => Ok(Box::new(fox::Fox::new(id, addr))),
        roles::WITCH => Ok(Box::new(witch::Witch::new(id, addr))),
        roles::CUPID => Ok(Box::new(cupid::Cupid::new(id, addr))),
        roles::BETRAYER => Ok(Box::new(bettrayer::Betrayer::new(id, addr))),
        _ => Err(format!("not found role {}", role)),
    }
}

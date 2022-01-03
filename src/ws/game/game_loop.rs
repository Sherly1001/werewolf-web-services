use std::sync::{Arc, Mutex};

use actix::Addr;

use crate::ws::ChatServer;

use super::cmds::BotMsg;
use super::game::{GameChannel, GameInfo};
use super::text_templates as ttp;

pub struct GameLoop {
    info: Arc<Mutex<GameInfo>>,
    addr: Addr<ChatServer>,
}

impl GameLoop {
    pub async fn new(info: Arc<Mutex<GameInfo>>, addr: Addr<ChatServer>) {
        let game = Self {
            info,
            addr,
        };
        game.run().await;
    }

    pub async fn run(&self) {
        for (_, player) in self.info.lock().unwrap().players.iter_mut() {
            player.on_start_game();
        }

        let next = self.info.lock().unwrap().next_flag.clone();

        while !self.info.lock().unwrap().is_ended {
            let is_day = self.info.lock().unwrap().is_day;
            let num_day = self.info.lock().unwrap().num_day;

            let gameplay = *self.info
                .lock()
                .unwrap()
                .channels
                .get(&GameChannel::GamePlay)
                .unwrap();

            println!("start");

            self.addr.do_send(BotMsg {
                channel_id: gameplay,
                msg: ttp::new_pharse(num_day, is_day),
                reply_to: None,
            });

            for (_, player) in self.info.lock().unwrap().players.iter_mut() {
                player.on_phase(is_day);
            }

            next.wait().await;

            println!("stop");

            if !is_day { self.info.lock().unwrap().num_day += 1; }
            self.info.lock().unwrap().is_day = !is_day;
            if self.info.lock().unwrap().num_day > 5 {
                self.info.lock().unwrap().is_ended = true;
            }
        }

        for (_, player) in self.info.lock().unwrap().players.iter_mut() {
            player.on_end_game();
        }

        self.info.lock().unwrap().is_ended = true;
    }
}

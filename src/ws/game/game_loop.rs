use std::sync::{Arc, Mutex};
use std::time::Duration;

use actix::{Addr, Arbiter};

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

            self.start_timmer();
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

    pub fn start_timmer(&self) {
        let addr = self.addr.clone();
        let info = self.info.clone();

        let is_day = info.lock().unwrap().is_day;
        let num_day = info.lock().unwrap().num_day;
        let (daytime, nighttime, preiod) = info.lock().unwrap().timmer;
        let next = info.lock().unwrap().next_flag.clone();

        let gameplay = *info
            .lock()
            .unwrap()
            .channels
            .get(&GameChannel::GamePlay)
            .unwrap();

        let timecount = if is_day { daytime } else { nighttime };

        let fut = async move {
            for count in (1..timecount + 1).rev() {
                {
                    let lock = info.lock().unwrap();
                    if lock.is_ended || lock.is_stopped ||
                        lock.is_day != is_day || lock.num_day != num_day {
                        return;
                    }
                }

                if count % preiod == 0 || count <= 5 {
                    addr.do_send(BotMsg {
                        channel_id: gameplay,
                        msg: ttp::timeout(count),
                        reply_to: None,
                    });
                }

                actix::clock::delay_for(Duration::from_secs(1)).await;
            }

            next.wake();
        };

        Arbiter::spawn(fut);
    }
}

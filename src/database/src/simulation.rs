use super::*;

/// For simulation
pub struct Simulation {
    history: Vec<data::SimulationExecution>,
    jpy: f64,
    btc: f64,
}

impl Simulation {
    pub fn new(jpy: f64, btc: f64) -> Self {
        Simulation {
            history: Vec::new(),
            jpy,
            btc,
        }
    }

    pub fn assets(&self) -> (f64, f64) {
        (self.jpy, self.btc)
    }

    pub fn deposit_jpy(&mut self, quantity: f64) -> Result<(), String> {
        if quantity > 1.0 {
            self.jpy += quantity;
            Ok(())
        } else {
            Err(error_message!(
                "quantity is too small! quantity : {}, jpy : {}, btc : {}",
                quantity,
                self.jpy,
                self.btc
            ))
        }
    }
    pub fn deposit_btc(&mut self, quantity: f64) -> Result<(), String> {
        if quantity > 0.0 && quantity < 25.0 {
            self.btc += quantity;
            Ok(())
        } else {
            Err(error_message!(
                "invalid quantity! quantity : {}, jpy : {}, btc : {}",
                quantity,
                self.jpy,
                self.btc
            ))
        }
    }
    pub fn withdraw_jpy(&mut self, quantity: f64) -> Result<(), String> {
        if quantity > 1.0 && quantity <= self.jpy {
            self.jpy -= quantity;
            Ok(())
        } else {
            Err(error_message!(
                "failed to withdraw jpy! quantity : {}, jpy : {}, btc : {}",
                quantity,
                self.jpy,
                self.btc
            ))
        }
    }

    pub fn withdraw_btc(&mut self, quantity: f64) -> Result<(), String> {
        if quantity > 0.0 && quantity < 25.0 && quantity <= self.btc {
            self.btc -= quantity;
            Ok(())
        } else {
            Err(error_message!(
                "failed to withdraw btc! quantity : {}, jpy : {}, btc : {}",
                quantity,
                self.jpy,
                self.btc
            ))
        }
    }

    pub fn buy_at(&mut self, timestamp: f64, price: i32, quantity: f64) -> Result<(), String> {
        if !(0.0001..=25.0).contains(&quantity) {
            self.history.push(data::SimulationExecution {
                timestamp,
                price,
                quantity,
                order: String::from("buy"),
                result: Err(error_message!("invalid quanity quntity : {}", quantity)),
            });
            return Err(error_message!("invalid quanity quntity : {}", quantity));
        }

        if let Err(result) = self.withdraw_jpy(price as f64 * quantity) {
            self.history.push(data::SimulationExecution {
                timestamp,
                price,
                quantity,
                order: String::from("buy"),
                result: Err(error_message!("failed to buy!\ndetails : {}", result)),
            });
            return Err(error_message!("failed to buy!\ndetails : {}", result));
        }
        if let Err(result) = self.deposit_btc(quantity) {
            self.history.push(data::SimulationExecution {
                timestamp,
                price,
                quantity,
                order: String::from("buy"),
                result: Err(error_message!("failed to buy!\ndetails : {}", result)),
            });
            return Err(error_message!("failed to buy!\ndetails : {}", result));
        }
        self.history.push(data::SimulationExecution {
            timestamp,
            price,
            quantity,
            order: String::from("buy"),
            result: Ok(()),
        });
        Ok(())
    }

    pub fn sell_at(&mut self, timestamp: f64, price: i32, quantity: f64) -> Result<(), String> {
        if !(0.0001..=25.0).contains(&quantity) {
            self.history.push(data::SimulationExecution {
                timestamp,
                price,
                quantity,
                order: String::from("sell"),
                result: Err(error_message!("invalid quanity quntity : {}", quantity)),
            });
            return Err(error_message!("invalid quanity quntity : {}", quantity));
        }
        if let Err(result) = self.deposit_jpy(price as f64 * quantity) {
            self.history.push(data::SimulationExecution {
                timestamp,
                price,
                quantity,
                order: String::from("sell"),
                result: Err(error_message!("failed to sell!\ndetails : {}", result)),
            });
            return Err(error_message!("failed to sell!\ndetails : {}", result));
        }
        if let Err(result) = self.withdraw_btc(quantity) {
            self.history.push(data::SimulationExecution {
                timestamp,
                price,
                quantity,
                order: String::from("sell"),
                result: Err(error_message!("failed to sell!\ndetails : {}", result)),
            });
            return Err(error_message!("failed to sell!\ndetails : {}", result));
        }
        self.history.push(data::SimulationExecution {
            timestamp,
            price,
            quantity,
            order: String::from("sell"),
            result: Ok(()),
        });
        Ok(())
    }

    pub fn sell_all(&mut self, timestamp: f64, price: i32) -> Result<(), String> {
        self.sell_at(timestamp, price, self.btc)
    }

    pub fn evaluate_at(&self, price: i32) -> f64 {
        self.jpy + self.btc * price as f64
    }

    pub fn get_history(&self) -> &Vec<data::SimulationExecution> {
        &self.history
    }

    pub fn get_traded_avaerage_price(&self, order: &str) -> f64 {
        let mut sum_price_quantity = 0.0;
        let mut sum_quantity = 0.0;
        for e in self.history.iter().filter(|e| e.order == order) {
            sum_price_quantity += e.price as f64 * e.quantity;
            sum_quantity += e.quantity;
        }
        sum_price_quantity / sum_quantity
    }

    pub fn estimate_profit(&self) -> f64 {
        let mut sum_quantity = 0.0;
        for e in self.history.iter().filter(|e| e.order == "sell") {
            sum_quantity += e.quantity;
        }

        (self.get_traded_avaerage_price("sell") - self.get_traded_avaerage_price("buy"))
            * sum_quantity
    }

    pub fn was_failed(&self) -> Result<(), String> {
        if let Some(e) = self.history.iter().filter(|e| e.result.is_err()).next() {
            return e.result.clone();
        }
        Ok(())
    }

    pub fn was_failed_by_rayon(&self) -> Result<(), String> {
        match self.history.par_iter().find_first(|e| e.result.is_err()) {
            Some(content) => content.result.clone(),
            None => Ok(()),
        }
    }
}

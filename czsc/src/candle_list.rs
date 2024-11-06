use crate::{Bar, Candle, FxType, Handle, KlineDir};

pub struct CandleList {
    #[allow(clippy::box_collection)]
    pub candle_list: Box<Vec<Candle>>,
}

impl CandleList {
    pub fn new() -> Self {
        CandleList {
            candle_list: Box::new(Vec::with_capacity(1_024_000)),
        }
    }

    pub fn update_candle(&mut self, bar: Handle<Bar>) -> bool {
        if self.candle_list.len() == 0 {
            let candle = Candle::new(&self.candle_list, bar, 0, KlineDir::Up);
            self.candle_list.push(candle);
        } else {
            let _dir = self.candle_list.last_mut().unwrap().try_add(bar);

            if _dir != KlineDir::Combine {
                let candle = Candle::new(&self.candle_list, bar, self.candle_list.len(), _dir);
                self.candle_list.push(candle);

                if self.candle_list.len() >= 3 {
                    let index = self.candle_list.len() - 2;
                    self.candle_list[index].fx_type = self.check_fx();
                }

                return true;
            }
        }
        false
    }

    fn check_fx(&mut self) -> FxType {
        let len = self.candle_list.len();
        let _cur = &self.candle_list[len - 2];
        let _pre = &self.candle_list[len - 3];
        let _next = &self.candle_list[len - 1];
        if _pre.high < _cur.high
            && _next.high < _cur.high
            && _pre.low < _cur.low
            && _next.low < _cur.low
        {
            return FxType::Top;
        }

        if _pre.high > _cur.high
            && _next.high > _cur.high
            && _pre.low > _cur.low
            && _next.low > _cur.low
        {
            return FxType::Bottom;
        }
        FxType::Unknown
    }
}

impl Default for CandleList {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for CandleList {
    type Target = Box<Vec<Candle>>;

    fn deref(&self) -> &Self::Target {
        &self.candle_list
    }
}

impl std::ops::DerefMut for CandleList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.candle_list
    }
}
